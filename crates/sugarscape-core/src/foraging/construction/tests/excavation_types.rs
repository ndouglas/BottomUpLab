use super::super::{
    excavation_navigation::assess,
    excavation_state::{include_counts, ExcavationEffect, ExcavationState},
};
use super::excavation_navigation::corridor;
use super::*;
#[test]
fn excavation_identity_is_explicit() {
    for (policy, wire) in [
        (ExcavationPolicy::ExplorationFirstV1, "exploration_first_v1"),
        (ExcavationPolicy::InformedCostV1, "informed_cost_v1"),
    ] {
        let json = format!("\"{wire}\"");
        assert_eq!(serde_json::to_string(&policy).unwrap(), json);
        assert_eq!(
            serde_json::from_str::<ExcavationPolicy>(&json).unwrap(),
            policy
        );
    }
    assert!(serde_json::from_str::<ExcavationPolicy>("\"informed_cost_v2\"").is_err());
}
fn accepted_state() -> ExcavationState {
    let plan = assess(&corridor(5), pos(2, 1), pos(4, 1), pos(1, 1), 1).unwrap();
    let mut state = ExcavationState::default();
    state
        .include_assessment(&plan, Some(plan.tied_candidates[0].clone()))
        .unwrap();
    state
}
fn work() -> WorkCounts {
    WorkCounts {
        opportunities: 2,
        digs: 1,
        ..Default::default()
    }
}
#[test]
fn assessment_storage_and_paid_effects_have_separate_counts() {
    let mut state = accepted_state();
    assert_eq!(
        (
            state.counts.assessments,
            state.counts.selected_opportunities,
            state.counts.initiated_digs
        ),
        (1, 0, 0)
    );
    state.commit(ExcavationEffect::Ordinary).unwrap();
    assert_eq!(state.counts.selected_opportunities, 0);
    state.commit(ExcavationEffect::SelectedTravel).unwrap();
    assert_eq!(
        (
            state.counts.selected_opportunities,
            state.counts.initiated_digs
        ),
        (1, 0)
    );
    state
        .check(ExcavationPolicy::InformedCostV1, &work(), 7, 7, 1)
        .unwrap();
    let plan = assess(&corridor(5), pos(2, 1), pos(4, 1), pos(1, 1), 2).unwrap();
    state
        .include_assessment(&plan, Some(plan.tied_candidates[0].clone()))
        .unwrap();
    state.commit(ExcavationEffect::SelectedDig).unwrap();
    assert_eq!(
        (
            state.counts.selected_opportunities,
            state.counts.initiated_digs
        ),
        (2, 1)
    );
    state
        .check(ExcavationPolicy::InformedCostV1, &work(), 7, 7, 2)
        .unwrap();
}
#[test]
fn count_sums_and_paid_effects_roll_back_on_late_overflow() {
    let mut total = ExcavationCounts {
        assessments: 1,
        distance_visits: u64::MAX,
        peak_queue: 7,
        ..Default::default()
    };
    let before = total;
    assert!(include_counts(
        &mut total,
        &ExcavationCounts {
            assessments: 1,
            distance_visits: 1,
            peak_queue: 9,
            ..Default::default()
        }
    )
    .is_err());
    assert_eq!(total, before);
    include_counts(
        &mut total,
        &ExcavationCounts {
            assessments: 1,
            peak_queue: 5,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((total.assessments, total.peak_queue), (2, 7));
    let mut state = accepted_state();
    state.counts.initiated_digs = u64::MAX;
    let before = state.clone();
    assert!(state.commit(ExcavationEffect::SelectedDig).is_err());
    assert_eq!(state, before);
    state.counts.distance_visits = u64::MAX;
    let before = state.clone();
    let plan = assess(&corridor(5), pos(2, 1), pos(4, 1), pos(1, 1), 2).unwrap();
    assert!(state.include_assessment(&plan, None).is_err());
    assert_eq!(state, before);
}
#[test]
fn legacy_diagnostics_must_be_zero_and_empty() {
    ExcavationState::default()
        .check(ExcavationPolicy::ExplorationFirstV1, &work(), 7, 7, 1)
        .unwrap();
    assert!(accepted_state()
        .check(ExcavationPolicy::ExplorationFirstV1, &work(), 7, 7, 1)
        .is_err());
    let mut state = ExcavationState::default();
    state.counts.faces_scanned = 1;
    assert!(state
        .check(ExcavationPolicy::ExplorationFirstV1, &work(), 7, 7, 1)
        .is_err());
}
#[test]
fn historical_record_validates_geometry_arithmetic_ticks_and_comparison() {
    let mut state = accepted_state();
    state.commit(ExcavationEffect::SelectedDig).unwrap();
    state
        .check(ExcavationPolicy::InformedCostV1, &work(), 7, 7, 40)
        .unwrap();
    for mutate in [
        |s: &mut ExcavationState| s.last_assessment.as_mut().unwrap().tick = 0,
        |s: &mut ExcavationState| s.last_assessment.as_mut().unwrap().tick = 41,
        |s: &mut ExcavationState| s.last_assessment.as_mut().unwrap().site = pos(7, 0),
        |s: &mut ExcavationState| {
            s.last_assessment
                .as_mut()
                .unwrap()
                .best_candidate
                .as_mut()
                .unwrap()
                .estimated_cost = 7
        },
        |s: &mut ExcavationState| {
            s.last_assessment
                .as_mut()
                .unwrap()
                .best_candidate
                .as_mut()
                .unwrap()
                .pre_opening_distance = Some(2)
        },
        |s: &mut ExcavationState| {
            s.last_assessment
                .as_mut()
                .unwrap()
                .selected_candidate
                .as_mut()
                .unwrap()
                .face = pos(7, 0)
        },
        |s: &mut ExcavationState| s.last_assessment.as_mut().unwrap().reference_cost = Some(6),
        |s: &mut ExcavationState| s.last_assessment.as_mut().unwrap().best_candidate = None,
        |s: &mut ExcavationState| s.last_assessment.as_mut().unwrap().selected_candidate = None,
        |s: &mut ExcavationState| s.counts.selected_opportunities = 2,
        |s: &mut ExcavationState| s.counts.initiated_digs = 2,
        |s: &mut ExcavationState| s.counts.assessments = 3,
    ] {
        let mut invalid = state.clone();
        mutate(&mut invalid);
        assert!(invalid
            .check(ExcavationPolicy::InformedCostV1, &work(), 7, 7, 40)
            .is_err());
    }
    let plan = assess(&corridor(3), pos(2, 1), pos(4, 1), pos(1, 1), 1).unwrap();
    let mut rejected = ExcavationState::default();
    rejected.include_assessment(&plan, None).unwrap();
    rejected
        .check(ExcavationPolicy::InformedCostV1, &work(), 7, 7, 40)
        .unwrap();
}

#[test]
fn selection_must_be_a_tied_candidate_and_invalid_selection_is_atomic() {
    let plan = assess(&corridor(5), pos(2, 1), pos(4, 1), pos(1, 1), 1).unwrap();
    let mut state = ExcavationState::default();
    let before = state.clone();
    let mut invalid = plan.tied_candidates[0].clone();
    invalid.approach = pos(4, 1);
    assert!(state.include_assessment(&plan, Some(invalid)).is_err());
    assert_eq!(state, before);
}
#[test]
fn historical_selected_geometry_and_equal_best_cost_are_required() {
    let mut state = accepted_state();
    state.commit(ExcavationEffect::SelectedDig).unwrap();
    for mutate in [
        |s: &mut ExcavationState| {
            s.last_assessment
                .as_mut()
                .unwrap()
                .selected_candidate
                .as_mut()
                .unwrap()
                .pre_opening_distance = Some(2)
        },
        |s: &mut ExcavationState| {
            let selected = s
                .last_assessment
                .as_mut()
                .unwrap()
                .selected_candidate
                .as_mut()
                .unwrap();
            selected.approach_moves = 1;
            selected.estimated_cost = 7;
        },
        |s: &mut ExcavationState| {
            s.last_assessment
                .as_mut()
                .unwrap()
                .selected_candidate
                .as_mut()
                .unwrap()
                .approach = pos(1, 1)
        },
    ] {
        let mut invalid = state.clone();
        mutate(&mut invalid);
        assert!(invalid
            .check(ExcavationPolicy::InformedCostV1, &work(), 7, 7, 1)
            .is_err());
    }
    let no_digs = WorkCounts {
        opportunities: 2,
        ..Default::default()
    };
    assert!(state
        .check(ExcavationPolicy::InformedCostV1, &no_digs, 7, 7, 1)
        .is_err());
}
