use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt};

/// Errors at the exact-arithmetic and registered-record boundaries.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidDenominator,
    InvalidFraction(String),
    Overflow,
    InvalidProbability,
    InvalidWeights,
    HistoryTooLong,
    ImpossibleObservation,
    InvalidPolicy,
    InvalidFixture(String),
    RecordTooLarge,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDenominator => write!(
                f,
                "fraction denominator must be positive; division by zero is undefined"
            ),
            Self::InvalidFraction(detail) => write!(f, "invalid canonical fraction: {detail}"),
            Self::Overflow => write!(
                f,
                "exact fraction arithmetic exceeds the supported integer range"
            ),
            Self::InvalidProbability => write!(f, "observation likelihood must be in [0, 1]"),
            Self::InvalidWeights => write!(
                f,
                "prior weights must be nonnegative and sum exactly to one"
            ),
            Self::HistoryTooLong => {
                write!(f, "observation history exceeds the supported horizon of 8")
            }
            Self::ImpossibleObservation => {
                write!(f, "observation has zero total mass under the prior")
            }
            Self::InvalidPolicy => write!(
                f,
                "policy must name a registered action for each observation"
            ),
            Self::InvalidFixture(detail) => write!(f, "invalid reference fixture: {detail}"),
            Self::RecordTooLarge => write!(f, "record exceeds the declared output size bound"),
        }
    }
}

impl std::error::Error for Error {}

/// Reduced decimal-string fraction used in reference and output records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRatio {
    pub num: String,
    pub den: String,
}

/// A reduced signed fraction with a strictly positive denominator.
///
/// Registered priority-domain bound (not a guarantee for arbitrary inputs):
/// write initial weights as w_j/2, w_j in {0,1,2}, with sum w_j = 2.
/// Every registered likelihood can be expressed as a_j/10, 0 <= a_j <= 10.
/// At prefix k <= 8, the unnormalized masses are w_j product(a_j)/(2*10^k).
/// Observation products have denominator at most 10^8. Integer masses M_j
/// have sum S <= 2*10^k; normalized weights M_j/S have numerators and
/// denominators <= 2*10^k, including after reduction.
///
/// In the actual sequential update at step k, prior denominators divide
/// S_(k-1) <= 2*10^(k-1). Multiplying a likelihood uses factors <= 10,
/// so intermediate products and mass components are <= 2*10^k <= 2*10^8.
/// Both mass denominators divide 10*S_(k-1), so their LCM is <= 2*10^k.
/// GCD-scaled addition products are at most this LCM; their sum is also at
/// most it because total observation mass is <= 1. Its reduced components
/// are <= 2*10^8. Normalization division multiplies components bounded by
/// 2*10^8, so even before cancellation its products are <= 4*10^16.
/// Comparisons with zero/one use products <= 2*10^8; comparing any two
/// registered components uses products <= 4*10^16. These bounds cover the
/// checked sequential intermediates, not merely the final wire values,
/// and are below i128::MAX. Public inputs outside this domain remain
/// fallible, including on intermediate overflow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    num: i128,
    den: i128,
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Ratio {
    pub fn new(num: i128, den: i128) -> Result<Self, Error> {
        if den <= 0 {
            return Err(Error::InvalidDenominator);
        }
        Self::from_magnitudes(num.unsigned_abs(), den as u128, num < 0)
    }

    fn from_magnitudes(num: u128, den: u128, negative: bool) -> Result<Self, Error> {
        if den == 0 {
            return Err(Error::InvalidDenominator);
        }
        let divisor = gcd(num, den);
        let num = num / divisor;
        let den = i128::try_from(den / divisor).map_err(|_| Error::Overflow)?;
        let num = if negative && num == (1u128 << 127) {
            i128::MIN
        } else {
            let magnitude = i128::try_from(num).map_err(|_| Error::Overflow)?;
            if negative {
                -magnitude
            } else {
                magnitude
            }
        };
        Ok(Self { num, den })
    }

    pub fn num(self) -> i128 {
        self.num
    }
    pub fn den(self) -> i128 {
        self.den
    }

    pub fn wire(self) -> WireRatio {
        WireRatio {
            num: self.num.to_string(),
            den: self.den.to_string(),
        }
    }

    /// Parse canonical, reduced fractions; never repair a malformed record.
    pub fn from_wire(wire: WireRatio) -> Result<Self, Error> {
        fn decimal(text: &str, signed: bool) -> Result<i128, Error> {
            if text.is_empty() || text.len() > 40 {
                return Err(Error::InvalidFraction(
                    "decimal must contain 1 through 40 characters".into(),
                ));
            }
            let digits = if signed {
                text.strip_prefix('-').unwrap_or(text)
            } else {
                text
            };
            if digits.is_empty()
                || !digits.bytes().all(|x| x.is_ascii_digit())
                || (digits.len() > 1 && digits.starts_with('0'))
                || text == "-0"
            {
                return Err(Error::InvalidFraction(
                    "decimal has a noncanonical sign, digit or leading zero".into(),
                ));
            }
            text.parse()
                .map_err(|_| Error::InvalidFraction("decimal exceeds i128 range".into()))
        }
        let num = decimal(&wire.num, true)?;
        let den = decimal(&wire.den, false)?;
        let value = Self::new(num, den)?;
        if value.num != num || value.den != den {
            return Err(Error::InvalidFraction("fraction is not reduced".into()));
        }
        Ok(value)
    }

    pub fn checked_add(self, other: Self) -> Result<Self, Error> {
        self.add_or_sub(other, false)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, Error> {
        self.add_or_sub(other, true)
    }

    fn add_or_sub(self, other: Self, subtract: bool) -> Result<Self, Error> {
        let common = gcd(self.den as u128, other.den as u128);
        let left_scale = other.den as u128 / common;
        let right_scale = self.den as u128 / common;
        let left = self
            .num
            .unsigned_abs()
            .checked_mul(left_scale)
            .ok_or(Error::Overflow)?;
        let right = other
            .num
            .unsigned_abs()
            .checked_mul(right_scale)
            .ok_or(Error::Overflow)?;
        let left_negative = self.num < 0;
        let right_negative = (other.num < 0) ^ subtract;
        let (num, negative) = if left_negative == right_negative {
            (
                left.checked_add(right).ok_or(Error::Overflow)?,
                left_negative,
            )
        } else if left >= right {
            (left - right, left_negative)
        } else {
            (right - left, right_negative)
        };
        // Reduce the numerator against the common factor before forming the LCM.
        let reduction = gcd(num, common);
        let den = right_scale
            .checked_mul(other.den as u128 / reduction)
            .ok_or(Error::Overflow)?;
        Self::from_magnitudes(num / reduction, den, negative)
    }

    pub fn checked_mul(self, other: Self) -> Result<Self, Error> {
        let left_cancel = gcd(self.num.unsigned_abs(), other.den as u128);
        let right_cancel = gcd(other.num.unsigned_abs(), self.den as u128);
        let num = (self.num.unsigned_abs() / left_cancel)
            .checked_mul(other.num.unsigned_abs() / right_cancel)
            .ok_or(Error::Overflow)?;
        let den = (self.den as u128 / right_cancel)
            .checked_mul(other.den as u128 / left_cancel)
            .ok_or(Error::Overflow)?;
        Self::from_magnitudes(num, den, (self.num < 0) ^ (other.num < 0))
    }

    pub fn checked_div(self, other: Self) -> Result<Self, Error> {
        if other.num == 0 {
            return Err(Error::InvalidDenominator);
        }
        let num_cancel = gcd(self.num.unsigned_abs(), other.num.unsigned_abs());
        let den_cancel = gcd(self.den as u128, other.den as u128);
        let num = (self.num.unsigned_abs() / num_cancel)
            .checked_mul(other.den as u128 / den_cancel)
            .ok_or(Error::Overflow)?;
        let den = (self.den as u128 / den_cancel)
            .checked_mul(other.num.unsigned_abs() / num_cancel)
            .ok_or(Error::Overflow)?;
        Self::from_magnitudes(num, den, (self.num < 0) ^ (other.num < 0))
    }

    pub fn checked_cmp(self, other: Self) -> Result<Ordering, Error> {
        let left = self.num.checked_mul(other.den).ok_or(Error::Overflow)?;
        let right = other.num.checked_mul(self.den).ok_or(Error::Overflow)?;
        Ok(left.cmp(&right))
    }
}
