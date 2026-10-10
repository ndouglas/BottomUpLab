//! Supplemental engineering acceptance; all map preparation is paid and intent is controlled.
use super::super::{
    controller::{advance, check},
    decision::{decide_with_policies, Action, Decision, Policy, WaitReason},
    draws::PcgDraws,
    excavation_navigation::{assess, AssessmentPlan},
    excavation_state::check_counts,
};
use super::excavation_support::{informed_setup, prepare, tied_world};
use super::traffic_support::{fixture_tick, view};
use super::*;
use rand::Rng;

#[test]
fn explicit_legacy_excavation_preserves_both_old_world_apis_each_tick_and_rng() {
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let mut old = World::new_with_traffic(setup(), 7, traffic).unwrap();
        let mut explicit =
            World::new_with_policies(setup(), 7, traffic, ExcavationPolicy::ExplorationFirstV1)
                .unwrap();
        let mut ordinary = World::new(setup(), 7).unwrap();
        for _ in 0..40 {
            old.step().unwrap();
            explicit.step().unwrap();
            ordinary.step().unwrap();
            assert_eq!(old.state, explicit.state);
            assert_eq!(old.snapshot().unwrap(), explicit.snapshot().unwrap());
            assert_eq!(
                old.traffic_snapshot().unwrap(),
                explicit.traffic_snapshot().unwrap()
            );
            assert_eq!(
                explicit.excavation_summary().unwrap().counts,
                ExcavationCounts::default()
            );
            if traffic == TrafficPolicy::ShortestOnlyV1 {
                assert_eq!(ordinary.state, explicit.state);
                assert_eq!(ordinary.snapshot().unwrap(), explicit.snapshot().unwrap());
                assert_eq!(
                    ordinary.traffic_snapshot().unwrap(),
                    explicit.traffic_snapshot().unwrap()
                );
            }
        }
        let next = explicit.rng.gen::<u64>();
        assert_eq!(old.rng.gen::<u64>(), next);
        if traffic == TrafficPolicy::ShortestOnlyV1 {
            assert_eq!(ordinary.rng.gen::<u64>(), next);
        }
    }
}

fn two_worker_world(traffic: TrafficPolicy) -> World {
    let mut setup = informed_setup();
    setup.workers.push(pos(2, 2));
    let mut world =
        World::new_with_policies(setup, 7, traffic, ExcavationPolicy::InformedCostV1).unwrap();
    prepare(&mut world, false);
    assert_eq!(world.state.agents[1].cargo, None);
    check(&world.setup, &world.state).unwrap();
    world
}

#[test]
fn public_step_rolls_back_after_proactive_dig_and_real_pcg_draw_then_recovers() {
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let mut world = two_worker_world(traffic);
        let mut control = world.clone();
        let observations = world.state.agents[1].compute.observations;
        world.state.agents[1].compute.observations = u64::MAX;
        check(&world.setup, &world.state).unwrap();
        let before = world.state.clone();
        let mut expected_rng = world.rng.clone();
        let mut scratch = before.clone();
        let mut scratch_rng = world.rng.clone();
        let errors =
            advance(&world.setup, &mut scratch, &mut PcgDraws(&mut scratch_rng)).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.field.contains("agents[1]") && e.field.ends_with("observations")),
            "{errors:?}"
        );
        let a = &scratch.agents[0];
        assert_eq!(a.pos, before.agents[0].pos);
        assert!(scratch.terrain.is_open(pos(3, 1)).unwrap());
        assert_eq!(
            scratch.spoil.inventory().excavated,
            before.spoil.inventory().excavated + 1
        );
        assert!(matches!(a.cargo, Some(Cargo::Spoil(_))));
        assert_eq!(a.work.digs, before.agents[0].work.digs + 1);
        assert_eq!(
            a.work.opportunities,
            before.agents[0].work.opportunities + 1
        );
        assert_eq!(a.excavation.counts.assessments, 1);
        assert_eq!(a.excavation.counts.selected_opportunities, 1);
        assert_eq!(a.excavation.counts.initiated_digs, 1);
        let record = a.excavation.last_assessment.as_ref().unwrap();
        assert_eq!(record.tick, before.tick + 1);
        assert_eq!(record.selected_candidate.as_ref().unwrap().face, pos(3, 1));
        assert_eq!(record.selected_candidate, record.best_candidate);
        assert_eq!(scratch.food, before.food);
        assert_eq!(scratch.agents[1], before.agents[1]);
        assert_ne!(scratch_rng.gen::<u64>(), world.rng.clone().gen::<u64>());
        assert_eq!(world.step().unwrap_err(), errors);
        assert_eq!(world.state, before);
        assert_eq!(world.rng.clone().gen::<u64>(), expected_rng.gen::<u64>());
        world.state.agents[1].compute.observations = observations;
        world.step().unwrap();
        control.step().unwrap();
        assert_eq!(world.state, control.state);
        assert_eq!(world.rng.gen::<u64>(), control.rng.gen::<u64>());
        check(&world.setup, &world.state).unwrap();
    }
}

#[test]
fn malformed_selection_draw_never_commits_source_state() {
    for value in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
        let world = two_worker_world(TrafficPolicy::LocalYieldingV1);
        let before = world.state.clone();
        let mut scratch = before.clone();
        let errors = advance(&world.setup, &mut scratch, &mut Scripted::new(&[value])).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.field.contains("agents[0]") && e.field.ends_with("draw")),
            "{errors:?}"
        );
        assert_eq!(scratch, before);
        assert_eq!(world.state, before);
    }
}

fn remote_world(change: usize) -> World {
    let mut setup = informed_setup();
    setup.workers.push(pos(2, 2));
    setup.diggable.extend([pos(6, 0), pos(6, 6)]);
    setup.food.push(Resource {
        id: 43,
        pos: if change == 2 { pos(6, 0) } else { pos(6, 6) },
    });
    if change == 1 {
        setup.diggable.push(pos(6, 3));
    }
    let mut world = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    fixture_tick(&mut world, 1, Action::Move(pos(2, 3)));
    fixture_tick(&mut world, 1, Action::Move(pos(2, 4)));
    prepare(&mut world, false);
    let actions = match change {
        3 => [
            Action::Move(pos(2, 5)),
            Action::Wait(WaitReason::NoNeighbor),
            Action::Wait(WaitReason::NoNeighbor),
            Action::Wait(WaitReason::NoNeighbor),
        ],
        4 => [
            Action::Move(pos(2, 5)),
            Action::Move(pos(3, 5)),
            Action::Move(pos(2, 5)),
            Action::Move(pos(2, 4)),
        ],
        _ => [Action::Wait(WaitReason::NoNeighbor); 4],
    };
    for action in actions {
        fixture_tick(&mut world, 1, action);
    }
    world
}

fn assessment(world: &World) -> AssessmentPlan {
    let a = &world.state.agents[0];
    assess(
        &a.map,
        a.pos,
        a.site.unwrap(),
        world.setup.waste,
        world.state.tick + 1,
    )
    .unwrap()
}
fn decision(world: &World) -> (Decision, usize) {
    let mut draws = Scripted::new(&[0.0]);
    let d = decide_with_policies(
        &Policy::from(&world.setup),
        &world.state.agents[0],
        &view(world, 0),
        world.state.traffic_policy,
        world.state.excavation_policy,
        world.state.tick + 1,
        &mut draws,
    )
    .unwrap();
    (d, draws.next)
}
fn assert_remote_private_equivalence(change: usize) {
    let base = remote_world(0);
    let other = remote_world(change);
    check(&base.setup, &base.state).unwrap();
    check(&other.setup, &other.state).unwrap();
    assert_ne!(base.state, other.state);
    assert_eq!(base.state.agents[0], other.state.agents[0]);
    assert_eq!(view(&base, 0), view(&other, 0));
    match change {
        1 => {
            assert_ne!(base.state.terrain, other.state.terrain);
            assert_eq!(
                base.state.agents[0].map.kind(pos(6, 3)).unwrap(),
                CellKnowledge::Unknown
            );
        }
        2 => assert_ne!(base.state.food, other.state.food),
        3 => assert_ne!(base.state.agents[1].pos, other.state.agents[1].pos),
        4 => {
            assert_eq!(base.state.agents[1].pos, other.state.agents[1].pos);
            assert_ne!(base.state.agents[1].map, other.state.agents[1].map);
        }
        _ => unreachable!(),
    }
    let a = assessment(&base);
    let b = assessment(&other);
    assert_eq!(a.assessment, b.assessment);
    assert_eq!(a.tied_candidates, b.tied_candidates);
    assert_eq!(a.counts, b.counts);
    let (a, draws_a) = decision(&base);
    let (b, draws_b) = decision(&other);
    assert_eq!(a.agent.excavation, b.agent.excavation);
    assert_eq!(a.agent.traffic, b.agent.traffic);
    assert_eq!(a, b);
    assert_eq!((draws_a, draws_b), (1, 1));
    assert_eq!(a.action, Action::Dig(pos(3, 1)));
}
#[test]
fn unseen_terrain_cannot_change_assessment_decision_diagnostics_or_draws() {
    assert_remote_private_equivalence(1);
}
#[test]
fn hidden_food_cannot_change_assessment_decision_diagnostics_or_draws() {
    assert_remote_private_equivalence(2);
}
#[test]
fn remote_occupancy_cannot_change_assessment_decision_diagnostics_or_draws() {
    assert_remote_private_equivalence(3);
}
#[test]
fn another_workers_map_cannot_change_assessment_decision_diagnostics_or_draws() {
    assert_remote_private_equivalence(4);
}

#[test]
fn optimistic_unknown_assessment_never_learns_assumed_cells() {
    let world = remote_world(0);
    let a = &world.state.agents[0];
    assert_eq!(a.map.kind(pos(6, 6)).unwrap(), CellKnowledge::Unknown);
    let before = a.map.clone();
    let plan = assess(
        &a.map,
        a.pos,
        pos(6, 6),
        world.setup.waste,
        world.state.tick + 1,
    )
    .unwrap();
    assert_eq!(plan.counts.distance_calls, 3);
    assert!(plan.counts.distance_visits > 0);
    assert_eq!(a.map, before);
}

#[test]
fn fresh_local_capacity_changes_route_step_but_not_costs_or_tied_candidates() {
    let blocked = tied_world(TrafficPolicy::LocalYieldingV1, true);
    let mut free = blocked.clone();
    fixture_tick(&mut free, 1, Action::Move(pos(2, 3)));
    fixture_tick(&mut free, 2, Action::Move(pos(2, 3)));
    let a = &blocked.state.agents[0];
    let plan = assessment(&blocked);
    let other = assess(
        &a.map,
        a.pos,
        a.site.unwrap(),
        free.setup.waste,
        blocked.state.tick + 1,
    )
    .unwrap();
    assert_eq!(plan.assessment, other.assessment);
    assert_eq!(plan.tied_candidates, other.tied_candidates);
    assert_eq!(plan.counts, other.counts);
    let choose = |world: &World, values: &[f64]| {
        let mut draws = Scripted::new(values);
        let d = decide_with_policies(
            &Policy::from(&world.setup),
            a,
            &view(world, 0),
            world.state.traffic_policy,
            world.state.excavation_policy,
            blocked.state.tick + 1,
            &mut draws,
        )
        .unwrap();
        (d, draws.next)
    };
    let (wait, blocked_draws) = choose(&blocked, &[0.0]);
    let (step, free_draws) = choose(&free, &[0.0, 0.0]);
    assert_eq!(wait.action, Action::Wait(WaitReason::Congestion));
    assert_eq!(step.action, Action::Move(pos(2, 4)));
    assert_eq!(wait.agent.excavation, step.agent.excavation);
    assert_eq!((blocked_draws, free_draws), (1, 2));
}

fn isolated_bound(case: usize) {
    let work = WorkCounts {
        opportunities: 2,
        digs: 1,
        ..Default::default()
    };
    let mut counts = ExcavationCounts {
        assessments: 2,
        selected_opportunities: 1,
        initiated_digs: 1,
        ..Default::default()
    };
    match case {
        0 => counts.selected_opportunities = 0,
        1 => counts.assessments = 0,
        2 => counts.assessments = 3,
        3 => {
            counts.initiated_digs = 2;
            counts.selected_opportunities = 2;
        }
        _ => unreachable!(),
    }
    let violations = [
        counts.initiated_digs > counts.selected_opportunities,
        counts.selected_opportunities > counts.assessments,
        counts.assessments > work.opportunities,
        counts.initiated_digs > work.digs,
    ];
    assert_eq!(violations.iter().filter(|v| **v).count(), 1);
    assert!(violations[case]);
    let errors = check_counts(&counts, &work).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "excavation.counts");
}
#[test]
fn initiated_digs_cannot_exceed_selected_opportunities_in_isolation() {
    isolated_bound(0);
}
#[test]
fn selected_opportunities_cannot_exceed_assessments_in_isolation() {
    isolated_bound(1);
}
#[test]
fn assessments_cannot_exceed_paid_opportunities_in_isolation() {
    isolated_bound(2);
}
#[test]
fn initiated_digs_cannot_exceed_paid_digs_in_isolation() {
    isolated_bound(3);
}
