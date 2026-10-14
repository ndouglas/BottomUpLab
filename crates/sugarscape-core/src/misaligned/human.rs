//! The human's proxy choice. `top` is Proposition 4's (the J most sensitive attributes),
//! `extremes` Proposition 5's (the most and the least sensitive); ties go to the lower index,
//! the "arbitrary" tie-break of the supplemental made deterministic.
use super::worlds::{Set, State, L};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Human {
    Fixed,
    Top,
    Extremes,
}

/// Indices by descending sensitivity, ties to the lower index.
fn ranked(sensitivities: &State) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..L).collect();
    idx.sort_by(|&a, &b| {
        sensitivities[b]
            .partial_cmp(&sensitivities[a])
            .unwrap_or(Ordering::Equal)
            .then(a.cmp(&b))
    });
    idx
}

/// The `size` attributes with the largest sensitivities.
pub fn top(sensitivities: &State, size: usize) -> Set {
    let mut set = [false; L];
    for &i in ranked(sensitivities).iter().take(size.clamp(1, L - 1)) {
        set[i] = true;
    }
    set
}

/// The most and the least sensitive attribute (ties to the lower index); when all are equal,
/// the first two.
pub fn extremes(sensitivities: &State) -> Set {
    let hi = ranked(sensitivities)[0];
    let lo = (0..L).fold(0, |best, i| {
        if sensitivities[i] < sensitivities[best] {
            i
        } else {
            best
        }
    });
    let mut set = [false; L];
    if sensitivities[hi] == sensitivities[lo] {
        set[0] = true;
        set[1] = true;
    } else {
        set[hi] = true;
        set[lo] = true;
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_takes_the_largest_sensitivities_with_ties_to_the_lower_index() {
        assert_eq!(top(&[0.5, 0.5, 0.07, 0.07], 2), [true, true, false, false]);
        assert_eq!(top(&[0.1, 0.5, 0.07, 0.5], 2), [false, true, false, true]);
        assert_eq!(top(&[0.1; 4], 2), [true, true, false, false]);
        assert_eq!(top(&[0.1, 0.2, 0.3, 0.4], 1), [false, false, false, true]);
        assert_eq!(top(&[0.1, 0.2, 0.3, 0.4], 3), [false, true, true, true]);
    }

    #[test]
    fn extremes_takes_the_most_and_least_sensitive() {
        assert_eq!(extremes(&[0.5, 0.5, 0.07, 0.07]), [true, false, true, false]);
        assert_eq!(extremes(&[0.1, 0.5, 0.07, 0.5]), [false, true, true, false]);
        assert_eq!(extremes(&[0.1; 4]), [true, true, false, false], "all equal: the first two");
    }
}
