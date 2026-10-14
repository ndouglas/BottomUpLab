use super::types::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeanCosts {
    pub acquisition: [Ratio; 2],
    pub writing: [Ratio; 2],
    pub reading: [Ratio; 2],
    pub inspection: [Ratio; 2],
    pub movement: [Ratio; 2],
    pub handoff: [Ratio; 2],
    pub pickup: [Ratio; 2],
    pub delivery: [Ratio; 2],
    pub wait: [Ratio; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeanMetrics {
    pub correctness: Ratio,
    pub completion: Ratio,
    pub deadline: Ratio,
    pub spent: [Ratio; 2],
    pub reward: [Ratio; 2],
    pub net: [Ratio; 2],
    pub group_net: Ratio,
    pub reader_actions: Ratio,
    pub nonblank_reads: Ratio,
    pub costs: MeanCosts,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeanRow {
    pub key: (u64, u64, u64, u64, u64, Arm, ReaderRoutine),
    pub metrics: MeanMetrics,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestContrast {
    pub key: (u64, u64, u64, u64, u64, ReaderRoutine, Good, Contrast),
    pub metrics: MeanMetrics,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeanContrast {
    pub key: (u64, u64, u64, u64, u64, ReaderRoutine, Contrast),
    pub metrics: MeanMetrics,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub key: (u64, u64, u64, u64, u64, Arm),
    pub reader_net: Ratio,
    pub ties: Vec<ReaderRoutine>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Contrast {
    #[serde(rename = "Retained-Erased")]
    RetainedErased,
    #[serde(rename = "Retained-NoNote")]
    RetainedNoNote,
    #[serde(rename = "Erased-NoNote")]
    ErasedNoNote,
    #[serde(rename = "Retained-NoAcquireNoNote")]
    RetainedNoAcquireNoNote,
}
pub(crate) fn outcome_atoms(o: &Outcome) -> Result<[i64; 30], Error> {
    let mut atoms = [0; 30];
    atoms[0] = i64::from(o.correct);
    atoms[1] = i64::from(o.completed);
    atoms[2] = i64::from(o.deadline);
    let unsigned = |u: u64| i64::try_from(u).map_err(|_| Error::ArithmeticOverflow);
    for j in 0..2 {
        atoms[3 + j] = unsigned(o.spent[j])?;
        atoms[5 + j] = unsigned(o.reward[j])?;
        atoms[7 + j] = o.net[j];
    }
    atoms[9] = o.group_net;
    atoms[10] = unsigned(o.counts.attempted[1])?;
    atoms[11] = unsigned(o.counts.nonblank_reads)?;
    for j in 0..2 {
        atoms[12 + j] = unsigned(o.costs.acquisition[j])?;
    }
    for j in 0..2 {
        atoms[14 + j] = unsigned(o.costs.writing[j])?;
    }
    for j in 0..2 {
        atoms[16 + j] = unsigned(o.costs.reading[j])?;
    }
    for j in 0..2 {
        atoms[18 + j] = unsigned(o.costs.inspection[j])?;
    }
    for j in 0..2 {
        atoms[20 + j] = unsigned(o.costs.movement[j])?;
    }
    for j in 0..2 {
        atoms[22 + j] = unsigned(o.costs.handoff[j])?;
    }
    for j in 0..2 {
        atoms[24 + j] = unsigned(o.costs.pickup[j])?;
    }
    for j in 0..2 {
        atoms[26 + j] = unsigned(o.costs.delivery[j])?;
    }
    for j in 0..2 {
        atoms[28 + j] = unsigned(o.costs.wait[j])?;
    }
    Ok(atoms)
}
fn mean_pair(left: i64, right: i64) -> Result<Ratio, Error> {
    let sum = left.checked_add(right).ok_or(Error::ArithmeticOverflow)?;
    Ok(if sum % 2 == 0 {
        Ratio {
            numerator: sum / 2,
            denominator: 1,
        }
    } else {
        Ratio {
            numerator: sum,
            denominator: 2,
        }
    })
}
pub(crate) fn combine(left: &Outcome, right: &Outcome, mean: bool) -> Result<MeanMetrics, Error> {
    let left = outcome_atoms(left)?;
    let right = outcome_atoms(right)?;
    let mut atoms = std::array::from_fn(|_| Ratio::ZERO);
    for k in 0..30 {
        atoms[k] = if mean {
            mean_pair(left[k], right[k])?
        } else {
            Ratio {
                numerator: left[k]
                    .checked_sub(right[k])
                    .ok_or(Error::ArithmeticOverflow)?,
                denominator: 1,
            }
        };
    }
    Ok(from_atoms(atoms))
}
fn from_atoms(a: [Ratio; 30]) -> MeanMetrics {
    let pair = |k: usize| [a[k].clone(), a[k + 1].clone()];
    MeanMetrics {
        correctness: a[0].clone(),
        completion: a[1].clone(),
        deadline: a[2].clone(),
        spent: pair(3),
        reward: pair(5),
        net: pair(7),
        group_net: a[9].clone(),
        reader_actions: a[10].clone(),
        nonblank_reads: a[11].clone(),
        costs: MeanCosts {
            acquisition: pair(12),
            writing: pair(14),
            reading: pair(16),
            inspection: pair(18),
            movement: pair(20),
            handoff: pair(22),
            pickup: pair(24),
            delivery: pair(26),
            wait: pair(28),
        },
    }
}
fn atoms(m: &MeanMetrics) -> [Ratio; 30] {
    let mut a = std::array::from_fn(|_| Ratio::ZERO);
    a[0] = m.correctness.clone();
    a[1] = m.completion.clone();
    a[2] = m.deadline.clone();
    a[3..5].clone_from_slice(&m.spent);
    a[5..7].clone_from_slice(&m.reward);
    a[7..9].clone_from_slice(&m.net);
    a[9] = m.group_net.clone();
    a[10] = m.reader_actions.clone();
    a[11] = m.nonblank_reads.clone();
    a[12..14].clone_from_slice(&m.costs.acquisition);
    a[14..16].clone_from_slice(&m.costs.writing);
    a[16..18].clone_from_slice(&m.costs.reading);
    a[18..20].clone_from_slice(&m.costs.inspection);
    a[20..22].clone_from_slice(&m.costs.movement);
    a[22..24].clone_from_slice(&m.costs.handoff);
    a[24..26].clone_from_slice(&m.costs.pickup);
    a[26..28].clone_from_slice(&m.costs.delivery);
    a[28..30].clone_from_slice(&m.costs.wait);
    a
}
pub(crate) fn difference(left: &MeanMetrics, right: &MeanMetrics) -> Result<MeanMetrics, Error> {
    let l = atoms(left);
    let r = atoms(right);
    let mut a = std::array::from_fn(|_| Ratio::ZERO);
    for k in 0..30 {
        a[k] = subtract(&l[k], &r[k])?;
    }
    Ok(from_atoms(a))
}
fn subtract(left: &Ratio, right: &Ratio) -> Result<Ratio, Error> {
    let lden = i64::try_from(left.denominator).map_err(|_| Error::ArithmeticOverflow)?;
    let rden = i64::try_from(right.denominator).map_err(|_| Error::ArithmeticOverflow)?;
    let n = left
        .numerator
        .checked_mul(rden)
        .and_then(|l| {
            right
                .numerator
                .checked_mul(lden)
                .and_then(|r| l.checked_sub(r))
        })
        .ok_or(Error::ArithmeticOverflow)?;
    let d = left
        .denominator
        .checked_mul(right.denominator)
        .ok_or(Error::ArithmeticOverflow)?;
    let mut x = n.unsigned_abs();
    let mut y = d;
    while y != 0 {
        let t = x % y;
        x = y;
        y = t;
    }
    Ok(Ratio {
        numerator: n / i64::try_from(x).map_err(|_| Error::ArithmeticOverflow)?,
        denominator: d / x,
    })
}
pub(crate) fn compare(left: &Ratio, right: &Ratio) -> Result<std::cmp::Ordering, Error> {
    let ld = i64::try_from(left.denominator).map_err(|_| Error::ArithmeticOverflow)?;
    let rd = i64::try_from(right.denominator).map_err(|_| Error::ArithmeticOverflow)?;
    Ok(left
        .numerator
        .checked_mul(rd)
        .ok_or(Error::ArithmeticOverflow)?
        .cmp(
            &right
                .numerator
                .checked_mul(ld)
                .ok_or(Error::ArithmeticOverflow)?,
        ))
}
