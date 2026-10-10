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

#[test]
fn peer_opened_selected_face_is_retired_during_paid_hold_then_public_steps_continue() {
    use super::decision::{decide_with_policies, Action, Decision, Policy, WaitReason};
    use super::excavation_state::ExcavationEffect;
    use super::traffic::TrafficEffect;
    use traffic_support::{controlled_tick, decision_tick, fixture_tick, view};
    let mut open: Vec<_> = (3..=8).map(|y| pos(2, y)).collect();
    open.extend((4..=8).map(|y| pos(4, y)));
    open.extend([pos(3, 8), pos(3, 3), pos(1, 4)]);
    let setup = Setup {
        width: 7,
        height: 10,
        open,
        diggable: vec![pos(3, 4)],
        nest: vec![pos(2, 3), pos(2, 4), pos(2, 5)],
        waste: pos(1, 4),
        workers: vec![pos(2, 3), pos(2, 5), pos(2, 5)],
        food: vec![Resource {
            id: 42,
            pos: pos(4, 4),
        }],
        parameters: Parameters {
            p_search: 0.,
            p_return: 0.,
            lambda_fidelity: 0.,
            lambda_publish: 0.,
            lambda_waypoint: 0.,
        },
    };
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    // Engineering fixture controls: only trip intent is assigned below.
    // Every position, map, streak, hold and material effect comes from paid actions.
    // Move peers away temporarily, leaving capacity for paid exploration.
    fixture_tick(&mut w, 1, Action::Move(pos(2, 6)));
    fixture_tick(&mut w, 2, Action::Move(pos(2, 6)));
    // Subject cannot pass that occupied cell; move peers into the separate dead-end.
    for i in [1, 2] {
        for p in [pos(2, 5), pos(2, 4), pos(2, 3), pos(3, 3)] {
            fixture_tick(&mut w, i, Action::Move(p));
        }
    }
    for y in 4..=8 {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, 8)));
    for y in (4..=8).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    for y in 5..=8 {
        fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, 8)));
    for y in (4..=8).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(1, 4)));
    fixture_tick(&mut w, 0, Action::Move(pos(2, 4)));
    fixture_tick(&mut w, 0, Action::Move(pos(2, 3)));
    // Park peers at the approach, then subject learns the yielding pocket and returns.
    for i in [1, 2] {
        for p in [pos(2, 3), pos(2, 4)] {
            fixture_tick(&mut w, i, Action::Move(p));
        }
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, 3)));
    fixture_tick(&mut w, 0, Action::Move(pos(2, 3)));
    controlled_tick(&mut w, 0, Action::Wait(WaitReason::NoNeighbor), |a| {
        a.site = Some(pos(4, 4));
        a.frontier = None;
        a.face = None;
    });
    let choose = |w: &World, values: &[f64]| {
        let mut draws = Scripted::new(values);
        let d = decide_with_policies(
            &Policy::from(&w.setup),
            &w.state.agents[0],
            &view(w, 0),
            w.state.traffic_policy,
            w.state.excavation_policy,
            w.state.tick + 1,
            &mut draws,
        )
        .unwrap();
        (d, draws)
    };
    for _ in 0..2 {
        let (d, mut draws) = choose(&w, &[0.]);
        assert_eq!(d.action, Action::Wait(WaitReason::Congestion));
        decision_tick(&mut w, 0, d, &mut draws);
    }
    let (d, mut draws) = choose(&w, &[0., 0., 0.]);
    assert_eq!(d.action, Action::Move(pos(3, 3)));
    assert_eq!(d.traffic_effect, TrafficEffect::YieldMove);
    assert_eq!(d.excavation_effect, ExcavationEffect::SelectedTravel);
    let record = d.agent.excavation.last_assessment.as_ref().unwrap();
    assert_eq!(record.tick, 44);
    assert_eq!(record.reference_cost, Some(11));
    assert_eq!(record.candidate_pairs, 3);
    assert_eq!(
        record.selected_candidate,
        Some(ExcavationCandidate {
            face: pos(3, 4),
            approach: pos(2, 4),
            approach_moves: 1,
            spoil_leg_moves: 1,
            continuation_moves: 1,
            pre_opening_distance: Some(10),
            estimated_cost: 7,
        })
    );
    let o = view(&w, 0);
    super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut draws).unwrap();
    // Supplemental check: a still-valid face survives a draw-free hold decision.
    let (valid_hold, valid_draws) = choose(&w, &[]);
    assert_eq!(valid_hold.action, Action::Wait(WaitReason::TrafficHold));
    assert_eq!(valid_hold.agent.face, Some(pos(3, 4)));
    assert_eq!(valid_hold.agent.excavation, w.state.agents[0].excavation);
    assert_eq!(valid_draws.next, 0);
    // A later worker legally opens the retained face in the same ascending-ID tick.
    for i in 1..3 {
        let o = view(&w, i);
        let mut a = w.state.agents[i].clone();
        let delta = a.map.learn(&o).unwrap();
        a.compute
            .checked_include(&ComputeCounts {
                observations: 1,
                cells_inspected: o.cells.len() as u64,
                cells_learned: delta.first,
                observed_revisions: delta.observed_revisions,
                dig_confirmations: delta.dig_confirmations,
                ..Default::default()
            })
            .unwrap();
        let d = Decision {
            agent: a,
            action: if i == 1 {
                Action::Dig(pos(3, 4))
            } else {
                Action::Wait(WaitReason::NoNeighbor)
            },
            traffic_effect: TrafficEffect::Normal,
            excavation_effect: ExcavationEffect::Ordinary,
        };
        super::actions::apply(&w.setup, &mut w.state, i, d, &o, &mut Scripted::new(&[])).unwrap();
    }
    w.state.tick += 1;
    check(&w.setup, &w.state).unwrap();
    assert!(w.state.agents[0].traffic.hold_pending);
    assert_eq!(w.state.agents[0].face, Some(pos(3, 4)));
    assert!(w.state.terrain.is_open(pos(3, 4)).unwrap());

    let before = w.clone();
    // The public transaction must commit the paid hold after fresh peer-open learning.
    w.step().unwrap();
    check(&w.setup, &w.state).unwrap();
    assert_eq!(w.state.tick, before.state.tick + 1);
    let old = &before.state.agents[0];
    assert_eq!(
        old.map.kind(pos(3, 4)).unwrap(),
        CellKnowledge::KnownSolid { diggable: true }
    );
    let held = &w.state.agents[0];
    assert_eq!(held.face, None);
    assert_eq!(held.map.kind(pos(3, 4)).unwrap(), CellKnowledge::KnownOpen);
    assert_eq!(
        (
            held.pos,
            held.site,
            held.phase,
            held.cargo,
            held.find,
            held.frontier
        ),
        (
            old.pos,
            old.site,
            old.phase,
            old.cargo,
            old.find,
            old.frontier
        )
    );
    assert_eq!(held.excavation, old.excavation);
    assert!(!held.traffic.hold_pending);
    let mut expected_work = old.work.clone();
    expected_work.opportunities += 1;
    expected_work.waits += 1;
    expected_work.empty_congestion_waits += 1;
    assert_eq!(held.work, expected_work);
    let mut expected_traffic = old.traffic.clone();
    expected_traffic.hold_pending = false;
    expected_traffic.counts.hold_waits += 1;
    assert_eq!(held.traffic, expected_traffic);
    let mut expected_compute = old.compute;
    expected_compute.observations += 1;
    expected_compute.cells_inspected += 5;
    expected_compute.observed_revisions += 1;
    assert_eq!(held.compute, expected_compute);
    // Isolate this worker's decision draw stream: peers in World::step may draw.
    let (hold, draws) = choose(&before, &[]);
    assert_eq!(draws.next, 0);
    assert_eq!(hold.action, Action::Wait(WaitReason::TrafficHold));
    assert_eq!(hold.traffic_effect, TrafficEffect::Hold);
    assert_eq!(hold.excavation_effect, ExcavationEffect::Ordinary);
    let mut expected_agent = old.clone();
    expected_agent.map = held.map.clone();
    expected_agent.compute = expected_compute;
    expected_agent.face = None;
    assert_eq!(hold.agent, expected_agent);
    for _ in 0..20 {
        w.step().unwrap();
        check(&w.setup, &w.state).unwrap();
    }
}
