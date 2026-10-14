use super::{r, Error, Ratio, WireRatio};
use std::cmp::Ordering;

#[test]
fn rejected_fraction_does_not_panic() {
    assert_eq!(Ratio::new(1, 0), Err(Error::InvalidDenominator));
}

#[test]
fn negative_denominator_is_rejected() {
    assert_eq!(Ratio::new(1, -2), Err(Error::InvalidDenominator));
}

#[test]
fn constructor_reduces_signed_and_zero_fractions() {
    assert_eq!(
        r(-6, 8).wire(),
        WireRatio {
            num: "-3".into(),
            den: "4".into()
        }
    );
    assert_eq!(r(0, i128::MAX), r(0, 1));
}

#[test]
fn minimum_numerator_reduces_without_negation() {
    let reduced = r(i128::MIN, 2);
    assert_eq!((reduced.num(), reduced.den()), (i128::MIN / 2, 1));
    assert_eq!(r(i128::MIN, 1).checked_mul(r(1, 1)), Ok(r(i128::MIN, 1)));
}

#[test]
fn multiplication_cross_cancels_before_products() {
    assert_eq!(r(i128::MAX, 2).checked_mul(r(2, i128::MAX)), Ok(r(1, 1)));
}

#[test]
fn division_cross_cancels_minimum_numerators() {
    assert_eq!(r(i128::MIN, 1).checked_div(r(i128::MIN, 1)), Ok(r(1, 1)));
}

#[test]
fn division_cancels_signs() {
    assert_eq!(r(-3, 4).checked_div(r(-9, 2)), Ok(r(1, 6)));
    assert_eq!(r(3, 4).checked_div(r(-9, 2)), Ok(r(-1, 6)));
}

#[test]
fn zero_division_is_an_error() {
    assert_eq!(r(1, 2).checked_div(r(0, 1)), Err(Error::InvalidDenominator));
}

#[test]
fn addition_and_subtraction_use_common_denominator_factors() {
    assert_eq!(r(1, 6).checked_add(r(1, 3)), Ok(r(1, 2)));
    assert_eq!(r(1, 6).checked_sub(r(1, 3)), Ok(r(-1, 6)));
}

#[test]
fn addition_reduces_before_rejecting_representable_result() {
    assert_eq!(
        r(i128::MAX, 2).checked_add(r(i128::MAX, 2)),
        Ok(r(i128::MAX, 1))
    );
}

#[test]
fn subtraction_does_not_negate_minimum_integer() {
    assert_eq!(r(i128::MIN, 1).checked_sub(r(i128::MIN, 1)), Ok(r(0, 1)));
}

#[test]
fn maximum_integer_operation_failures_are_explicit() {
    assert_eq!(r(i128::MAX, 1).checked_add(r(1, 1)), Err(Error::Overflow));
    assert_eq!(r(i128::MIN, 1).checked_sub(r(1, 1)), Err(Error::Overflow));
    assert_eq!(r(i128::MIN, 1).checked_mul(r(-1, 1)), Err(Error::Overflow));
    assert_eq!(r(1, i128::MAX).checked_mul(r(1, 2)), Err(Error::Overflow));
    assert_eq!(r(i128::MIN, 1).checked_div(r(-1, 1)), Err(Error::Overflow));
    assert_eq!(r(1, 1).checked_div(r(i128::MIN, 1)), Err(Error::Overflow));
}

#[test]
fn checked_comparison_orders_negative_and_reduced_fractions() {
    assert_eq!(r(-1, 2).checked_cmp(r(-1, 3)), Ok(Ordering::Less));
    assert_eq!(r(2, 4).checked_cmp(r(1, 2)), Ok(Ordering::Equal));
    assert_eq!(r(2, 3).checked_cmp(r(1, 2)), Ok(Ordering::Greater));
}

#[test]
fn comparison_overflow_is_explicit() {
    assert_eq!(r(i128::MAX, 1).checked_cmp(r(1, 2)), Err(Error::Overflow));
}

#[test]
fn canonical_wire_fractions_round_trip_including_minimum_integer() {
    for ratio in [
        r(0, 1),
        r(-3, 7),
        r(i128::MIN, 1),
        r(i128::MAX, i128::MAX - 1),
    ] {
        let encoded = serde_json::to_string(&ratio.wire()).unwrap();
        let wire = serde_json::from_str(&encoded).unwrap();
        assert_eq!(Ratio::from_wire(wire), Ok(ratio));
    }
}

#[test]
fn malformed_and_unreduced_wire_fractions_are_rejected() {
    for (num, den) in [
        ("", "1"),
        ("+1", "2"),
        ("01", "2"),
        ("-0", "1"),
        ("1 ", "2"),
        ("1", "02"),
        ("1", "-2"),
        ("1", "0"),
        ("2", "4"),
        ("0", "2"),
        ("170141183460469231731687303715884105728", "1"),
    ] {
        assert!(
            Ratio::from_wire(WireRatio {
                num: num.into(),
                den: den.into()
            })
            .is_err(),
            "{num}/{den}"
        );
    }
    assert!(Ratio::from_wire(WireRatio {
        num: "1".repeat(41),
        den: "1".into()
    })
    .is_err());
}

#[test]
fn wire_fraction_rejects_unknown_fields() {
    assert!(serde_json::from_str::<WireRatio>(r#"{"num":"1","den":"2","extra":0}"#).is_err());
}

#[test]
fn errors_describe_the_rejected_operation() {
    let errors = [
        Error::InvalidDenominator,
        Error::InvalidFraction("numerator is noncanonical".into()),
        Error::Overflow,
        Error::InvalidProbability,
        Error::InvalidWeights,
        Error::HistoryTooLong,
        Error::ImpossibleObservation,
        Error::InvalidPolicy,
        Error::InvalidFixture("row 4".into()),
        Error::RecordTooLarge,
    ];
    for error in errors {
        assert!(!error.to_string().is_empty());
    }
}
