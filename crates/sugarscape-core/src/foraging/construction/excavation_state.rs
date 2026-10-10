//! Checked assessment diagnostics and successful paid-action effects.
use super::{
    excavation_navigation::{estimated_cost, AssessmentPlan},
    Checked, ExcavationAssessment, ExcavationCandidate, ExcavationCounts, ExcavationPolicy, Pos,
    WorkCounts,
};
use crate::config::FieldError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExcavationState {
    pub(super) counts: ExcavationCounts,
    pub(super) last_assessment: Option<ExcavationAssessment>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExcavationEffect {
    Ordinary,
    SelectedTravel,
    SelectedDig,
}
fn error(field: &str, message: &str) -> Vec<FieldError> {
    vec![FieldError::new(format!("excavation.{field}"), message)]
}
fn add(a: u64, b: u64, field: &str) -> Checked<u64> {
    a.checked_add(b)
        .ok_or_else(|| error(field, "counter overflow"))
}
pub(super) fn include_counts(
    total: &mut ExcavationCounts,
    delta: &ExcavationCounts,
) -> Checked<()> {
    let candidate = ExcavationCounts {
        assessments: add(total.assessments, delta.assessments, "assessments")?,
        faces_scanned: add(total.faces_scanned, delta.faces_scanned, "faces_scanned")?,
        candidate_pairs: add(
            total.candidate_pairs,
            delta.candidate_pairs,
            "candidate_pairs",
        )?,
        selected_opportunities: add(
            total.selected_opportunities,
            delta.selected_opportunities,
            "selected_opportunities",
        )?,
        initiated_digs: add(total.initiated_digs, delta.initiated_digs, "initiated_digs")?,
        distance_calls: add(total.distance_calls, delta.distance_calls, "distance_calls")?,
        distance_visits: add(
            total.distance_visits,
            delta.distance_visits,
            "distance_visits",
        )?,
        peak_queue: total.peak_queue.max(delta.peak_queue),
    };
    *total = candidate;
    Ok(())
}
impl ExcavationState {
    pub(super) fn include_assessment(
        &mut self,
        plan: &AssessmentPlan,
        selected: Option<ExcavationCandidate>,
    ) -> Checked<()> {
        if selected
            .as_ref()
            .is_some_and(|candidate| !plan.tied_candidates.contains(candidate))
        {
            return Err(error("selection", "must be an accepted tied minimum"));
        }
        let mut candidate = self.clone();
        include_counts(
            &mut candidate.counts,
            &ExcavationCounts {
                selected_opportunities: 0,
                initiated_digs: 0,
                ..plan.counts
            },
        )?;
        let mut assessment = plan.assessment.clone();
        assessment.selected_candidate = selected;
        candidate.last_assessment = Some(assessment);
        *self = candidate;
        Ok(())
    }
    pub(super) fn commit(&mut self, effect: ExcavationEffect) -> Checked<()> {
        let delta = match effect {
            ExcavationEffect::Ordinary => ExcavationCounts::default(),
            ExcavationEffect::SelectedTravel => ExcavationCounts {
                selected_opportunities: 1,
                ..Default::default()
            },
            ExcavationEffect::SelectedDig => ExcavationCounts {
                selected_opportunities: 1,
                initiated_digs: 1,
                ..Default::default()
            },
        };
        include_counts(&mut self.counts, &delta)
    }
    pub(super) fn check(
        &self,
        policy: ExcavationPolicy,
        work: &WorkCounts,
        width: u32,
        height: u32,
        completed_ticks: u32,
    ) -> Checked<()> {
        super::setup::dimensions(width, height)?;
        if policy == ExcavationPolicy::ExplorationFirstV1
            && (self.counts != ExcavationCounts::default() || self.last_assessment.is_some())
        {
            return Err(error(
                "policy",
                "exploration_first_v1 requires zero counters and no assessment",
            ));
        }
        let c = self.counts;
        if c.initiated_digs > c.selected_opportunities
            || c.selected_opportunities > c.assessments
            || c.assessments > work.opportunities
            || c.initiated_digs > work.digs
        {
            return Err(error("counts", "exceeds assessment or paid-work bound"));
        }
        if (c.assessments == 0) != self.last_assessment.is_none() {
            return Err(error(
                "last_assessment",
                "must exist exactly when assessments are nonzero",
            ));
        }
        let Some(record) = &self.last_assessment else {
            return Ok(());
        };
        if record.tick == 0 || record.tick > completed_ticks {
            return Err(error("tick", "must be in 1..=completed_ticks"));
        }
        coordinate(record.site, width, height)?;
        if record.candidate_pairs > c.candidate_pairs {
            return Err(error("candidate_pairs", "record exceeds cumulative count"));
        }
        for candidate in [&record.best_candidate, &record.selected_candidate]
            .into_iter()
            .flatten()
        {
            check_candidate(candidate, width, height)?;
            if record.candidate_pairs == 0 {
                return Err(error(
                    "candidate_pairs",
                    "candidate requires an examined pair",
                ));
            }
        }
        match (&record.best_candidate, &record.selected_candidate) {
            (None, Some(_)) => {
                return Err(error(
                    "selection",
                    "selected candidate requires a best candidate",
                ))
            }
            (Some(best), Some(selected)) => {
                if best.estimated_cost != selected.estimated_cost {
                    return Err(error("selection", "selected and best costs must match"));
                }
                if record
                    .reference_cost
                    .is_some_and(|before| selected.estimated_cost >= before)
                {
                    return Err(error("selection", "must strictly improve reference cost"));
                }
            }
            (Some(best), None)
                if record
                    .reference_cost
                    .is_none_or(|before| best.estimated_cost < before) =>
            {
                return Err(error(
                    "selection",
                    "improving best candidate must be selected",
                ));
            }
            _ => {}
        }
        Ok(())
    }
}
fn coordinate(p: Pos, width: u32, height: u32) -> Checked<()> {
    if p.x >= width || p.y >= height {
        return Err(error("candidate.pos", "must be inside the grid"));
    }
    Ok(())
}
fn check_candidate(candidate: &ExcavationCandidate, width: u32, height: u32) -> Checked<()> {
    coordinate(candidate.face, width, height)?;
    coordinate(candidate.approach, width, height)?;
    if !super::setup::neighbors(candidate.face, width, height).contains(&candidate.approach) {
        return Err(error(
            "candidate.approach",
            "must be cardinally adjacent to face",
        ));
    }
    if estimated_cost(
        candidate.approach_moves,
        candidate.spoil_leg_moves,
        candidate.continuation_moves,
    )? != candidate.estimated_cost
    {
        return Err(error("candidate.cost", "does not match paid-cost formula"));
    }
    let opened_distance = add(candidate.continuation_moves, 1, "candidate.geometry")?;
    if candidate
        .pre_opening_distance
        .is_some_and(|before| opened_distance >= before)
    {
        return Err(error(
            "candidate.geometry",
            "must strictly improve pre-opening distance",
        ));
    }
    Ok(())
}
