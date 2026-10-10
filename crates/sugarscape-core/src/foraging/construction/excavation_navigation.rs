//! Draw-free one-cell estimates from three reusable private distance fields.
use super::{
    knowledge::Knowledge, setup::neighbors, CellKnowledge, Checked, ExcavationAssessment,
    ExcavationCandidate, ExcavationCounts, Pos,
};
use crate::config::FieldError;
use std::collections::VecDeque;

pub(super) struct AssessmentPlan {
    pub(super) assessment: ExcavationAssessment,
    pub(super) tied_candidates: Vec<ExcavationCandidate>,
    pub(super) counts: ExcavationCounts,
}

fn error(message: &str) -> Vec<FieldError> {
    vec![FieldError::new("excavation.estimate", message)]
}
fn add(a: u64, b: u64) -> Checked<u64> {
    a.checked_add(b)
        .ok_or_else(|| error("cost or counter overflow"))
}
pub(super) fn estimated_cost(approach: u64, spoil_leg: u64, onward: u64) -> Checked<u64> {
    let round_trip = spoil_leg
        .checked_mul(2)
        .ok_or_else(|| error("spoil round-trip overflow"))?;
    add(add(add(approach, round_trip)?, onward)?, 3)
}
fn index(pos: Pos, width: u32) -> usize {
    (pos.y * width + pos.x) as usize
}
fn field(
    cells: &[CellKnowledge],
    width: u32,
    height: u32,
    seed: Pos,
    optimistic: bool,
    counts: &mut ExcavationCounts,
) -> Checked<Vec<Option<u32>>> {
    let traversable = |p| match cells[index(p, width)] {
        CellKnowledge::KnownOpen => true,
        CellKnowledge::Unknown => optimistic,
        CellKnowledge::KnownSolid { .. } => false,
    };
    let mut distances = vec![None; cells.len()];
    if !traversable(seed) {
        return Ok(distances);
    }
    counts.distance_calls = add(counts.distance_calls, 1)?;
    distances[index(seed, width)] = Some(0u32);
    let mut queue = VecDeque::from([seed]);
    counts.peak_queue = counts.peak_queue.max(1);
    while let Some(p) = queue.pop_front() {
        counts.distance_visits = add(counts.distance_visits, 1)?;
        let distance = distances[index(p, width)]
            .expect("queued distance")
            .checked_add(1)
            .ok_or_else(|| error("field distance overflow"))?;
        for next in neighbors(p, width, height) {
            if traversable(next) && distances[index(next, width)].is_none() {
                distances[index(next, width)] = Some(distance);
                queue.push_back(next);
            }
        }
        counts.peak_queue = counts.peak_queue.max(queue.len() as u64);
    }
    Ok(distances)
}

pub(super) fn assess(
    knowledge: &Knowledge,
    origin: Pos,
    site: Pos,
    outlet: Pos,
    tick: u32,
) -> Checked<AssessmentPlan> {
    let (width, height) = knowledge.dimensions();
    super::setup::dimensions(width, height)?;
    for p in [origin, site, outlet] {
        knowledge.kind(p)?;
    }
    if tick == 0 {
        return Err(error("assessment tick must be one-based"));
    }
    if knowledge.kind(origin)? != CellKnowledge::KnownOpen {
        return Err(error("origin must be KnownOpen"));
    }
    if matches!(knowledge.kind(outlet)?, CellKnowledge::KnownSolid { .. }) {
        return Err(error("outlet cannot be remembered solid"));
    }
    let known = knowledge.known();
    let mut cells = vec![CellKnowledge::Unknown; (width * height) as usize];
    for cell in &known {
        cells[index(cell.pos, width)] = cell.kind;
    }
    let mut counts = ExcavationCounts {
        assessments: 1,
        ..Default::default()
    };
    let known_origin = field(&cells, width, height, origin, false, &mut counts)?;
    let known_outlet = field(&cells, width, height, outlet, false, &mut counts)?;
    let optimistic_site = field(&cells, width, height, site, true, &mut counts)?;
    let reference_cost = known_origin[index(site, width)]
        .or(optimistic_site[index(origin, width)])
        .map(u64::from);
    let mut tied_candidates: Vec<ExcavationCandidate> = vec![];
    for cell in known {
        let CellKnowledge::KnownSolid { diggable } = cell.kind else {
            continue;
        };
        counts.faces_scanned = add(counts.faces_scanned, 1)?;
        if !diggable {
            continue;
        }
        let face = cell.pos;
        let onward = if face == site {
            Some(0)
        } else {
            neighbors(face, width, height)
                .into_iter()
                .filter_map(|p| optimistic_site[index(p, width)])
                .min()
                .map(|distance| add(u64::from(distance), 1))
                .transpose()?
        };
        for approach in neighbors(face, width, height) {
            let Some(approach_moves) = known_origin[index(approach, width)] else {
                continue;
            };
            counts.candidate_pairs = add(counts.candidate_pairs, 1)?;
            let (Some(spoil_leg), Some(onward)) = (known_outlet[index(approach, width)], onward)
            else {
                continue;
            };
            let pre_opening_distance = optimistic_site[index(approach, width)].map(u64::from);
            let opened_distance = add(onward, 1)?;
            if pre_opening_distance.is_some_and(|before| opened_distance >= before) {
                continue;
            }
            let candidate = ExcavationCandidate {
                face,
                approach,
                approach_moves: u64::from(approach_moves),
                spoil_leg_moves: u64::from(spoil_leg),
                continuation_moves: onward,
                pre_opening_distance,
                estimated_cost: estimated_cost(
                    u64::from(approach_moves),
                    u64::from(spoil_leg),
                    onward,
                )?,
            };
            match tied_candidates
                .first()
                .map(|best| candidate.estimated_cost.cmp(&best.estimated_cost))
            {
                None | Some(std::cmp::Ordering::Less) => {
                    tied_candidates.clear();
                    tied_candidates.push(candidate);
                }
                Some(std::cmp::Ordering::Equal) => tied_candidates.push(candidate),
                Some(std::cmp::Ordering::Greater) => {}
            }
        }
    }
    tied_candidates.sort_by_key(|candidate| (candidate.face, candidate.approach));
    let best_candidate = tied_candidates.first().cloned();
    if best_candidate
        .as_ref()
        .is_some_and(|best| reference_cost.is_some_and(|before| best.estimated_cost >= before))
    {
        tied_candidates.clear();
    }
    Ok(AssessmentPlan {
        assessment: ExcavationAssessment {
            tick,
            site,
            reference_cost,
            candidate_pairs: counts.candidate_pairs,
            best_candidate,
            selected_candidate: None,
        },
        tied_candidates,
        counts,
    })
}
