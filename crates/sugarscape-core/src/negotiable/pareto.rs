use super::cake::sort_ratios;
use super::{Error, Ratio, WireRatio};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub type Point = [Ratio; 2];

/// The paper forbids strict improvement of both coordinates; strong efficiency
/// also forbids an improvement of one coordinate while preserving the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParetoKind {
    PaperWeak,
    Strong,
}

/// An original input index and its probability in a dominating mixture.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessPart {
    pub policy_id: u8,
    pub weight: WireRatio,
}

/// Production witnesses contain at most two original indices, in index order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    pub parts: Vec<WitnessPart>,
    pub payoff: [WireRatio; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Classification {
    pub efficient: bool,
    pub witness: Option<Witness>,
}

fn dominates(point: Point, candidate: Point, kind: ParetoKind) -> Result<bool, Error> {
    let alice = point[0].checked_cmp(candidate[0])?;
    let bob = point[1].checked_cmp(candidate[1])?;
    Ok(match kind {
        ParetoKind::PaperWeak => alice == Ordering::Greater && bob == Ordering::Greater,
        ParetoKind::Strong => {
            alice != Ordering::Less
                && bob != Ordering::Less
                && (alice == Ordering::Greater || bob == Ordering::Greater)
        }
    })
}

fn segment_point(origin: Point, delta: Point, t: Ratio) -> Result<Point, Error> {
    Ok([
        origin[0].checked_add(t.checked_mul(delta[0])?)?,
        origin[1].checked_add(t.checked_mul(delta[1])?)?,
    ])
}

fn witness(i: usize, j: usize, t: Ratio, payoff: Point) -> Result<Witness, Error> {
    let zero = Ratio::new(0, 1)?;
    let one = Ratio::new(1, 1)?;
    let parts = if i == j || t == zero {
        vec![WitnessPart {
            policy_id: i as u8,
            weight: one.wire(),
        }]
    } else if t == one {
        vec![WitnessPart {
            policy_id: j as u8,
            weight: one.wire(),
        }]
    } else {
        vec![
            WitnessPart {
                policy_id: i as u8,
                weight: one.checked_sub(t)?.wire(),
            },
            WitnessPart {
                policy_id: j as u8,
                weight: t.wire(),
            },
        ]
    };
    Ok(Witness {
        parts,
        payoff: payoff.map(Ratio::wire),
    })
}

/// Exact stochastic dominance over the convex hull of 1 through 9 input points.
/// Input positions remain witness IDs, even when different positions coincide.
/// Arithmetic errors propagate; no floating grid or generic solver is involved.
///
/// Completeness: from any dominating hull point, move northeast along (1,1)
/// until reaching the compact hull's boundary. Domination is preserved. For a
/// two-dimensional hull that boundary point lies on an edge between two original
/// vertices; a lower-dimensional hull is already a segment or singleton. Thus
/// enumerating all original pairs also detects domination by interior mixtures
/// involving three or more vertices. On a segment, each coordinate's comparison
/// with the candidate changes only at its crossing. Every open interval between
/// crossings has constant comparison signs, so one exact midpoint suffices;
/// evaluating the crossings themselves preserves boundary-only improvements.
///
/// At most 45 segments (including singleton pairs), four breakpoints and three
/// midpoints per segment are visited, in stable (i,j,t) order. In the registered
/// cake, coordinate differences are integers of magnitude <= 30, so crossings
/// have numerator/denominator <= 30. Midpoint denominators are <= 1800; segment
/// payoff numerators are <= 54000. All checked intermediates/comparison products
/// are bounded by 10^8, far below i128::MAX. Other public inputs remain fallible.
pub fn classify(
    points: &[Point],
    candidate: usize,
    kind: ParetoKind,
) -> Result<Classification, Error> {
    if points.is_empty() || points.len() > 9 {
        return Err(Error::InvalidFixture(
            "Pareto classification requires 1 through 9 two-coordinate points".into(),
        ));
    }
    if candidate >= points.len() {
        return Err(Error::InvalidFixture(
            "Pareto candidate index is outside the supplied points".into(),
        ));
    }
    let zero = Ratio::new(0, 1)?;
    let one = Ratio::new(1, 1)?;
    let two = Ratio::new(2, 1)?;
    let target = points[candidate];
    for (i, &origin) in points.iter().enumerate() {
        for (j, &end) in points.iter().enumerate().skip(i) {
            let delta = [
                end[0].checked_sub(origin[0])?,
                end[1].checked_sub(origin[1])?,
            ];
            let mut breakpoints = vec![zero, one];
            for coordinate in 0..2 {
                if delta[coordinate] != zero {
                    let crossing = target[coordinate]
                        .checked_sub(origin[coordinate])?
                        .checked_div(delta[coordinate])?;
                    if crossing.checked_cmp(zero)? != Ordering::Less
                        && crossing.checked_cmp(one)? != Ordering::Greater
                    {
                        breakpoints.push(crossing);
                    }
                }
            }
            sort_ratios(&mut breakpoints)?;
            breakpoints.dedup();
            let mut samples = Vec::with_capacity(7);
            for (index, &t) in breakpoints.iter().enumerate() {
                samples.push(t);
                if let Some(&next) = breakpoints.get(index + 1) {
                    samples.push(t.checked_add(next)?.checked_div(two)?);
                }
            }
            for t in samples {
                let payoff = segment_point(origin, delta, t)?;
                if dominates(payoff, target, kind)? {
                    return Ok(Classification {
                        efficient: false,
                        witness: Some(witness(i, j, t, payoff)?),
                    });
                }
            }
        }
    }
    Ok(Classification {
        efficient: true,
        witness: None,
    })
}
