use super::{Error, Ratio};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    Red,
    Green,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HistoryResult {
    Supported([Ratio; 2]),
    Impossible { first_impossible_prefix: usize },
}

fn validate_inputs(prior: [Ratio; 2], likelihood: [Ratio; 2]) -> Result<(), Error> {
    let zero = Ratio::new(0, 1)?;
    let one = Ratio::new(1, 1)?;
    for weight in prior {
        if weight.checked_cmp(zero)? == Ordering::Less {
            return Err(Error::InvalidWeights);
        }
    }
    if prior[0].checked_add(prior[1])? != one {
        return Err(Error::InvalidWeights);
    }
    for probability in likelihood {
        if probability.checked_cmp(zero)? == Ordering::Less
            || probability.checked_cmp(one)? == Ordering::Greater
        {
            return Err(Error::InvalidProbability);
        }
    }
    Ok(())
}

/// Normalize the two subjective masses after one supplied observation.
/// Zero total mass is impossible; it never resets or revives a prior.
pub fn update_weights(prior: [Ratio; 2], likelihood: [Ratio; 2]) -> Result<[Ratio; 2], Error> {
    validate_inputs(prior, likelihood)?;
    let mass = [
        prior[0].checked_mul(likelihood[0])?,
        prior[1].checked_mul(likelihood[1])?,
    ];
    let total = mass[0].checked_add(mass[1])?;
    if total.num() == 0 {
        return Err(Error::ImpossibleObservation);
    }
    Ok([mass[0].checked_div(total)?, mass[1].checked_div(total)?])
}

/// Evaluate a supplied history of at most eight observations, retaining the
/// one-based first impossible prefix rather than evaluating a fresh suffix.
pub fn history_weights(
    prior: [Ratio; 2],
    red: [Ratio; 2],
    history: &[Observation],
) -> Result<HistoryResult, Error> {
    if history.len() > 8 {
        return Err(Error::HistoryTooLong);
    }
    validate_inputs(prior, red)?;
    let one = Ratio::new(1, 1)?;
    let green = [one.checked_sub(red[0])?, one.checked_sub(red[1])?];
    let mut weights = prior;
    for (index, observation) in history.iter().enumerate() {
        let likelihood = match observation {
            Observation::Red => red,
            Observation::Green => green,
        };
        match update_weights(weights, likelihood) {
            Ok(updated) => weights = updated,
            Err(Error::ImpossibleObservation) => {
                return Ok(HistoryResult::Impossible {
                    first_impossible_prefix: index + 1,
                })
            }
            Err(error) => return Err(error),
        }
    }
    Ok(HistoryResult::Supported(weights))
}
