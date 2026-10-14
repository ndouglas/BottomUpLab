use super::{Error, Ratio, WireRatio};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Stable registered policy ID: three times the red action plus the green action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy(u8);

impl Policy {
    pub fn from_id(id: u8) -> Result<Self, Error> {
        if id < 9 {
            Ok(Self(id))
        } else {
            Err(Error::InvalidPolicy)
        }
    }
    pub fn id(self) -> u8 {
        self.0
    }
    pub fn actions(self) -> [u8; 2] {
        [self.0 / 3, self.0 % 3]
    }
}

/// One exact registered policy/objective cell, including every maximizing tie.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CakeRow {
    pub weight: WireRatio,
    pub policy_id: u8,
    pub expected: [WireRatio; 2],
    pub objective: WireRatio,
    pub global_maximizers: Vec<u8>,
    pub local_maximizers: Vec<u8>,
}

const UTILITIES: [[i128; 2]; 3] = [[30, 0], [20, 20], [0, 30]];

/// Evaluate each principal using that principal's own observation distribution.
pub fn cake_payoff(policy: Policy) -> Result<[Ratio; 2], Error> {
    let [red, green] = policy.actions().map(usize::from);
    let expectation = |principal, red_mass, green_mass| {
        Ratio::new(UTILITIES[red][principal], 1)?
            .checked_mul(Ratio::new(red_mass, 10)?)?
            .checked_add(
                Ratio::new(UTILITIES[green][principal], 1)?
                    .checked_mul(Ratio::new(green_mass, 10)?)?,
            )
    };
    Ok([expectation(0, 9, 1)?, expectation(1, 1, 9)?])
}

/// Fallible insertion sort: Ratio comparison can overflow for public inputs.
pub(super) fn sort_ratios(values: &mut [Ratio]) -> Result<(), Error> {
    for index in 1..values.len() {
        let mut cursor = index;
        while cursor > 0 && values[cursor].checked_cmp(values[cursor - 1])? == Ordering::Less {
            values.swap(cursor, cursor - 1);
            cursor -= 1;
        }
    }
    Ok(())
}

/// The registered twentieths grid and four exact action boundaries.
pub fn registered_weights() -> Result<Vec<Ratio>, Error> {
    let mut weights = (0..=20)
        .map(|k| Ratio::new(k, 20))
        .collect::<Result<Vec<_>, _>>()?;
    for (num, den) in [(1, 19), (2, 11), (9, 11), (18, 19)] {
        weights.push(Ratio::new(num, den)?);
    }
    sort_ratios(&mut weights)?;
    weights.dedup();
    Ok(weights)
}

fn maximizers(scores: &[Ratio]) -> Result<Vec<u8>, Error> {
    let mut best = scores[0];
    let mut ids = vec![0];
    for (index, &score) in scores.iter().enumerate().skip(1) {
        match score.checked_cmp(best)? {
            Ordering::Greater => {
                best = score;
                ids.clear();
                ids.push(index as u8);
            }
            Ordering::Equal => ids.push(index as u8),
            Ordering::Less => {}
        }
    }
    Ok(ids)
}

fn local_maximizers(weight: Ratio, complement: Ratio) -> Result<Vec<u8>, Error> {
    // Score unnormalized likelihood masses separately at each observation.
    // No normalized-priority thresholds enter these decisions.
    let actions = |alice_likelihood, bob_likelihood| -> Result<Vec<u8>, Error> {
        let alice_mass = weight.checked_mul(Ratio::new(alice_likelihood, 10)?)?;
        let bob_mass = complement.checked_mul(Ratio::new(bob_likelihood, 10)?)?;
        let scores = UTILITIES
            .iter()
            .map(|&[alice, bob]| {
                alice_mass
                    .checked_mul(Ratio::new(alice, 1)?)?
                    .checked_add(bob_mass.checked_mul(Ratio::new(bob, 1)?)?)
            })
            .collect::<Result<Vec<_>, _>>()?;
        maximizers(&scores)
    };
    let red = actions(9, 1)?;
    let green = actions(1, 9)?;
    Ok(red
        .into_iter()
        .flat_map(|r| green.iter().map(move |&g| 3 * r + g))
        .collect())
}

/// All 225 exact objective cells and complete whole/local maximizer sets.
///
/// Registered-domain arithmetic is bounded: payoffs are integers in [0,30];
/// weights have denominator <= 20 and numerator <= denominator. Objectives
/// have denominator dividing the weight denominator and numerator <= 600.
/// Local likelihood masses have denominator dividing 10 times that denominator
/// (<= 200), and action-score numerators <= 6000. Checked intermediate sums,
/// products and comparisons are therefore comfortably below i128::MAX.
pub fn cake_rows() -> Result<Vec<CakeRow>, Error> {
    let payoffs = (0..9)
        .map(|id| cake_payoff(Policy::from_id(id)?))
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::with_capacity(225);
    for weight in registered_weights()? {
        let complement = Ratio::new(1, 1)?.checked_sub(weight)?;
        let objectives = payoffs
            .iter()
            .map(|&[alice, bob]| {
                weight
                    .checked_mul(alice)?
                    .checked_add(complement.checked_mul(bob)?)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let global = maximizers(&objectives)?;
        let local = local_maximizers(weight, complement)?;
        if global != local {
            return Err(Error::InvalidFixture(
                "cake whole-policy and observation-local maximizing sets disagree".into(),
            ));
        }
        for (id, (&expected, objective)) in payoffs.iter().zip(objectives).enumerate() {
            rows.push(CakeRow {
                weight: weight.wire(),
                policy_id: id as u8,
                expected: expected.map(Ratio::wire),
                objective: objective.wire(),
                global_maximizers: global.clone(),
                local_maximizers: local.clone(),
            });
        }
    }
    Ok(rows)
}
