use super::super::{
    decision::{decide_with_policies, Action, Policy},
    excavation_state::ExcavationEffect,
};
use super::excavation_support::*;
use super::traffic_support;
use super::*;
#[test]
fn informed_known_site_selects_early_dig_before_payment() {
    let w = informed_world(
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    );
    let o = traffic_support::view(&w, 0);
    let before = w.state.agents[0].work.digs;
    let mut draws = Scripted::new(&[0.0]);
    let d = decide_with_policies(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &o,
        w.state.traffic_policy,
        w.state.excavation_policy,
        w.state.tick + 1,
        &mut draws,
    )
    .unwrap();
    assert_eq!(d.action, Action::Dig(pos(3, 1)));
    assert_eq!(d.excavation_effect, ExcavationEffect::SelectedDig);
    assert_eq!(draws.next, 1);
    assert_eq!(w.state.agents[0].work.digs, before);
}
#[test]
fn early_dig_wins_with_a_reachable_unknown_frontier() {
    let w = frontier_world();
    assert_eq!(
        w.state.agents[0].map.kind(pos(0, 3)).unwrap(),
        CellKnowledge::KnownOpen
    );
    assert_eq!(
        w.state.agents[0].map.kind(pos(0, 2)).unwrap(),
        CellKnowledge::Unknown
    );
    assert_eq!(
        w.state.agents[0].map.kind(pos(0, 4)).unwrap(),
        CellKnowledge::Unknown
    );
    let (frontiers, _) =
        super::super::navigation::frontiers(&w.state.agents[0].map, w.state.agents[0].pos).unwrap();
    assert!(frontiers.iter().any(|f| f.pos == pos(0, 3)));
    let mut draws = Scripted::new(&[0.0]);
    let d = decide_with_policies(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &traffic_support::view(&w, 0),
        w.state.traffic_policy,
        w.state.excavation_policy,
        w.state.tick + 1,
        &mut draws,
    )
    .unwrap();
    assert_eq!(d.action, Action::Dig(pos(3, 1)));
}
#[test]
fn legacy_prepared_world_travels_the_existing_route() {
    let w = informed_world(
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::ExplorationFirstV1,
    );
    let mut draws = Scripted::new(&[0.0]);
    let d = decide_with_policies(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &traffic_support::view(&w, 0),
        w.state.traffic_policy,
        w.state.excavation_policy,
        w.state.tick + 1,
        &mut draws,
    )
    .unwrap();
    assert_eq!(d.action, Action::Move(pos(2, 2)));
    assert_eq!(draws.next, 1);
    assert_eq!(d.agent.excavation.counts, ExcavationCounts::default());
    assert!(d.agent.excavation.last_assessment.is_none());
}
fn choose(w: &World, values: &[f64]) -> (super::super::decision::Decision, Scripted) {
    let mut draws = Scripted::new(values);
    let d = decide_with_policies(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &traffic_support::view(w, 0),
        w.state.traffic_policy,
        w.state.excavation_policy,
        w.state.tick + 1,
        &mut draws,
    )
    .unwrap();
    (d, draws)
}
#[test]
fn paid_dig_disposal_and_resumed_pursuit_preserve_food_intent() {
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let mut w = informed_world(traffic, ExcavationPolicy::InformedCostV1);
        let before = w.state.agents[0].work.clone();
        let tick = w.state.tick;
        traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
        let a = &w.state.agents[0];
        assert_eq!(a.excavation.counts.selected_opportunities, 1);
        assert_eq!(a.excavation.counts.initiated_digs, 1);
        assert_eq!(
            a.excavation.last_assessment.as_ref().unwrap().tick,
            tick + 1
        );
        assert_eq!(w.state.spoil.inventory().excavated, 1);
        assert_eq!(w.state.spoil.views()[0].born_tick, tick);
        assert!(matches!(a.cargo, Some(Cargo::Spoil(_))));
        assert_eq!(a.pos, pos(2, 1));
        traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
        assert_eq!(w.state.agents[0].pos, pos(1, 1));
        let retained = w.state.agents[0].excavation.clone();
        traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[])).unwrap();
        assert_eq!(w.state.agents[0].excavation, retained);
        assert_eq!(w.state.spoil.inventory().disposed, 1);
        assert_eq!(w.state.agents[0].cargo, None);
        traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
        assert_eq!(w.state.agents[0].pos, pos(2, 1));
        traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
        let a = &w.state.agents[0];
        assert_eq!(a.pos, pos(3, 1));
        assert_eq!(
            (
                a.work.digs - before.digs,
                a.work.spoil_moves - before.spoil_moves,
                a.work.disposals - before.disposals
            ),
            (1, 1, 1)
        );
        assert_eq!(
            (a.site, a.phase, a.find),
            (Some(pos(4, 1)), FoodPhase::Departing, None)
        );
        assert_eq!(
            (
                a.work.pickups,
                a.work.deposits,
                a.work.publications,
                a.work.uninformed_departures,
                a.work.recruited_departures,
                a.work.fidelity_departures
            ),
            (
                before.pickups,
                before.deposits,
                before.publications,
                before.uninformed_departures,
                before.recruited_departures,
                before.fidelity_departures
            )
        );
        assert_eq!(w.state.food.inventory().available, 1);
    }
}
#[test]
fn selected_effects_cannot_be_silently_relabelled_or_attached_to_wrong_actions() {
    for change in 0..7 {
        let mut w = informed_world(
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1,
        );
        let (mut d, mut draws) = choose(&w, &[0.0]);
        match change {
            0 => d.excavation_effect = ExcavationEffect::Ordinary,
            1 => d.action = Action::Move(pos(2, 2)),
            2 => d.excavation_effect = ExcavationEffect::SelectedTravel,
            3 => d.agent.excavation.last_assessment.as_mut().unwrap().tick -= 1,
            4 => d.agent.face = None,
            5 => d.agent.excavation.counts.selected_opportunities += 1,
            6 => w.state.excavation_policy = ExcavationPolicy::ExplorationFirstV1,
            _ => unreachable!(),
        }
        let o = traffic_support::view(&w, 0);
        let err = super::super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut draws);
        assert!(err.is_err(), "case {change} accepted");
    }
}
#[test]
fn controller_rejects_each_excavation_paid_bound_with_valid_physical_work() {
    for bound in 0..4 {
        let mut w = informed_world(
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1,
        );
        let (d, _) = choose(&w, &[0.0]);
        let mut excavation = d.agent.excavation;
        excavation.last_assessment.as_mut().unwrap().tick = w.state.tick;
        match bound {
            0 => excavation.counts.initiated_digs = 1,
            1 => excavation.counts.selected_opportunities = 2,
            2 => excavation.counts.assessments = w.state.agents[0].work.opportunities + 1,
            3 => {
                excavation.counts.selected_opportunities = 1;
                excavation.counts.initiated_digs = 1;
            }
            _ => unreachable!(),
        }
        w.state.agents[0].excavation = excavation;
        let errors = super::super::controller::check(&w.setup, &w.state).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.field == "agents[0].excavation.counts"),
            "{errors:?}"
        );
    }
}
#[test]
fn paid_selected_counts_commit_overflow_fails_after_physical_dig() {
    let mut w = informed_world(
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    );
    w.state.agents[0].excavation.counts.selected_opportunities = u64::MAX;
    let (d, mut draws) = choose(&w, &[0.0]);
    let o = traffic_support::view(&w, 0);
    let errors =
        super::super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut draws).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.field == "dig.spoil.food.excavation.selected_opportunities"),
        "{errors:?}"
    );
    // apply is scratch-only; the public controller owns rollback of this actual physical work.
    assert_eq!(w.state.spoil.inventory().excavated, 1);
}
#[test]
fn equal_and_higher_cost_reject_without_selection_draws() {
    for (depth, reference) in [(3, 6), (2, 4)] {
        let w = short_world(depth);
        let (d, draws) = choose(&w, &[0.0]);
        let assessment = d.agent.excavation.last_assessment.unwrap();
        assert_eq!(assessment.reference_cost, Some(reference));
        assert_eq!(assessment.best_candidate.unwrap().estimated_cost, 6);
        assert_eq!(assessment.selected_candidate, None);
        assert_eq!(d.action, Action::Move(pos(2, 2)));
        assert_eq!(d.excavation_effect, ExcavationEffect::Ordinary);
        assert_eq!(draws.next, 1); // existing singleton route draw only
    }
}
#[test]
fn unavailable_known_outlet_rejects_without_selection_draws() {
    let mut setup = informed_setup();
    setup.open.extend([pos(1, 5), pos(0, 5)]);
    setup.waste = pos(0, 5);
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    prepare(&mut w, false);
    assert_eq!(
        w.state.agents[0].map.kind(pos(0, 5)).unwrap(),
        CellKnowledge::Unknown
    );
    let (d, draws) = choose(&w, &[0.0]);
    assert_eq!(d.action, Action::Move(pos(2, 2)));
    assert_eq!(
        d.agent.excavation.last_assessment.unwrap().best_candidate,
        None
    );
    assert_eq!(draws.next, 1);
}
#[test]
fn ordered_pair_ties_select_then_draw_only_the_route_to_that_approach() {
    for (traffic, draw, face, approach) in [
        (TrafficPolicy::ShortestOnlyV1, 0.0, pos(3, 4), pos(2, 4)),
        (TrafficPolicy::ShortestOnlyV1, 0.9, pos(3, 6), pos(2, 6)),
        (TrafficPolicy::LocalYieldingV1, 0.0, pos(3, 4), pos(2, 4)),
        (TrafficPolicy::LocalYieldingV1, 0.9, pos(3, 6), pos(2, 6)),
    ] {
        let mut w = tied_world(traffic, false);
        let (d, mut draws) = choose(&w, &[draw, 0.0]);
        let assessment = d.agent.excavation.last_assessment.as_ref().unwrap();
        assert_eq!(assessment.reference_cost, Some(12));
        assert_eq!(assessment.best_candidate.as_ref().unwrap().face, pos(3, 4));
        assert_eq!(assessment.selected_candidate.as_ref().unwrap().face, face);
        assert_eq!(
            assessment.selected_candidate.as_ref().unwrap().approach,
            approach
        );
        assert_eq!(
            assessment
                .selected_candidate
                .as_ref()
                .unwrap()
                .estimated_cost,
            10
        );
        assert_eq!(
            (d.action, d.excavation_effect, draws.next),
            (Action::Move(approach), ExcavationEffect::SelectedTravel, 2)
        );
        traffic_support::decision_tick(&mut w, 0, d, &mut draws);
        assert_eq!(
            w.state.agents[0].excavation.counts.selected_opportunities,
            1
        );
        assert_eq!(w.state.agents[0].excavation.counts.initiated_digs, 0);
        let (d, draws) = choose(&w, &[0.0]);
        assert_eq!(
            (d.action, d.excavation_effect, draws.next),
            (Action::Dig(face), ExcavationEffect::SelectedDig, 1)
        );
    }
}
#[test]
fn selected_approach_congestion_uses_each_traffic_policy_and_preserves_hold_history() {
    use super::super::{decision::WaitReason, traffic::TrafficEffect};
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let mut w = tied_world(traffic, true);
        for _ in 0..2 {
            let (d, mut draws) = choose(&w, &[0.0]);
            assert_eq!(
                (d.action, d.traffic_effect, d.excavation_effect, draws.next),
                (
                    Action::Wait(WaitReason::Congestion),
                    TrafficEffect::BlockedWait,
                    ExcavationEffect::SelectedTravel,
                    1
                )
            );
            assert_eq!(
                d.agent
                    .excavation
                    .last_assessment
                    .as_ref()
                    .unwrap()
                    .selected_candidate
                    .as_ref()
                    .unwrap()
                    .approach,
                pos(2, 4)
            );
            traffic_support::decision_tick(&mut w, 0, d, &mut draws);
        }
        let values = if traffic == TrafficPolicy::LocalYieldingV1 {
            vec![0.0, 0.0, 0.0]
        } else {
            vec![0.0]
        };
        let (d, mut draws) = choose(&w, &values);
        assert_eq!(draws.next, values.len());
        if traffic == TrafficPolicy::LocalYieldingV1 {
            assert_eq!(
                (d.action, d.traffic_effect, d.excavation_effect),
                (
                    Action::Move(pos(2, 6)),
                    TrafficEffect::YieldMove,
                    ExcavationEffect::SelectedTravel
                )
            );
            traffic_support::decision_tick(&mut w, 0, d, &mut draws);
            let history = w.state.agents[0].excavation.clone();
            let (d, mut draws) = choose(&w, &[]);
            assert_eq!(
                (d.action, d.excavation_effect, draws.next),
                (
                    Action::Wait(WaitReason::TrafficHold),
                    ExcavationEffect::Ordinary,
                    0
                )
            );
            assert_eq!(d.agent.excavation, history);
            traffic_support::decision_tick(&mut w, 0, d, &mut draws);
            assert_eq!(w.state.agents[0].excavation, history);
            // Fresh reassessment may now switch to the other face at this approach.
            let (d, draws) = choose(&w, &[0.0]);
            assert_eq!(
                (d.action, d.excavation_effect, draws.next),
                (Action::Dig(pos(3, 6)), ExcavationEffect::SelectedDig, 1)
            );
        } else {
            assert_eq!(
                (d.action, d.traffic_effect),
                (
                    Action::Wait(WaitReason::Congestion),
                    TrafficEffect::BlockedWait
                )
            );
            traffic_support::decision_tick(&mut w, 0, d, &mut draws);
        }
        assert_eq!(w.state.agents[0].excavation.counts.assessments, 3);
        assert_eq!(
            w.state.agents[0].excavation.counts.selected_opportunities,
            3
        );
        assert_eq!(w.state.agents[0].excavation.counts.initiated_digs, 0);
    }
}
#[test]
fn at_site_and_protected_site_transition_before_assessment() {
    use super::super::decision::WaitReason;
    for (site, abandoned) in [(pos(2, 1), 0), (pos(2, 0), 1)] {
        let mut w = informed_world(
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1,
        );
        traffic_support::controlled_tick(&mut w, 0, Action::Wait(WaitReason::NoNeighbor), |a| {
            a.site = Some(site)
        });
        let (d, draws) = choose(&w, &[]);
        assert_eq!(
            (d.action, d.agent.phase, d.agent.site, draws.next),
            (
                Action::Wait(WaitReason::Transition),
                FoodPhase::Searching,
                None,
                0
            )
        );
        assert_eq!(d.agent.work.abandoned_targets, abandoned);
        assert_eq!(d.agent.excavation.counts, ExcavationCounts::default());
    }
}
#[test]
fn uninformed_searching_and_empty_returning_bypass_assessment() {
    use super::super::decision::WaitReason;
    for (phase, action, values) in [
        (
            FoodPhase::Departing,
            Action::Wait(WaitReason::Transition),
            vec![0.0],
        ),
        (FoodPhase::Searching, Action::Dig(pos(3, 1)), vec![0.5, 0.0]),
        (
            FoodPhase::Returning,
            Action::Wait(WaitReason::EmptyArrival),
            vec![],
        ),
    ] {
        let mut w = informed_world(
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1,
        );
        traffic_support::controlled_tick(&mut w, 0, Action::Wait(WaitReason::NoNeighbor), |a| {
            a.phase = phase;
            a.site = None;
        });
        let (d, draws) = choose(&w, &values);
        assert_eq!(d.action, action);
        assert_eq!(draws.next, values.len());
        assert_eq!(d.excavation_effect, ExcavationEffect::Ordinary);
        assert_eq!(d.agent.excavation.counts, ExcavationCounts::default());
    }
}
#[test]
fn genuine_paid_held_workers_bypass_assessment_in_all_modes_and_loaded_jobs() {
    use super::super::{decision::WaitReason, traffic::TrafficEffect};
    for mode in [
        Mode::Departing,
        Mode::Searching,
        Mode::EmptyReturning,
        Mode::FoodReturning,
        Mode::SpoilHauling,
    ] {
        let (mut w, i, o) = traffic_support::held_worker(mode);
        w.state.excavation_policy = ExcavationPolicy::InformedCostV1;
        let before = w.state.agents[i].clone();
        let mut draws = Scripted::new(&[]);
        let d = decide_with_policies(
            &Policy::from(&w.setup),
            &before,
            &o,
            w.state.traffic_policy,
            w.state.excavation_policy,
            w.state.tick + 1,
            &mut draws,
        )
        .unwrap();
        assert_eq!(
            (d.action, d.traffic_effect, d.excavation_effect, draws.next),
            (
                Action::Wait(WaitReason::TrafficHold),
                TrafficEffect::Hold,
                ExcavationEffect::Ordinary,
                0
            )
        );
        assert_eq!(d.agent.excavation, before.excavation);
        assert_eq!(
            (d.agent.cargo, d.agent.find, d.agent.phase, d.agent.site),
            (before.cargo, before.find, before.phase, before.site)
        );
        traffic_support::decision_tick(&mut w, i, d, &mut draws);
        if matches!(
            mode,
            Mode::FoodReturning | Mode::SpoilHauling | Mode::EmptyReturning
        ) {
            let o = traffic_support::view(&w, i);
            let mut draws = Scripted::new(&[0.0]);
            let d = decide_with_policies(
                &Policy::from(&w.setup),
                &w.state.agents[i],
                &o,
                w.state.traffic_policy,
                w.state.excavation_policy,
                w.state.tick + 1,
                &mut draws,
            )
            .unwrap();
            assert_eq!(d.agent.excavation, before.excavation);
            assert_eq!(d.excavation_effect, ExcavationEffect::Ordinary);
            assert_eq!(draws.next, 1);
        }
    }
}
#[test]
fn terminal_loaded_disposal_and_nest_arrival_override_defensive_hold() {
    use super::super::decision::WaitReason;
    for mode in [
        Mode::FoodReturning,
        Mode::EmptyReturning,
        Mode::SpoilHauling,
    ] {
        let (mut w, i, _) = traffic_support::held_worker(mode);
        w.state.excavation_policy = ExcavationPolicy::InformedCostV1;
        let paid_yield_memory = w.state.agents[i].traffic.clone();
        traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(3, 2)));
        for p in [pos(2, 1), pos(2, 2)] {
            traffic_support::fixture_tick(&mut w, i, Action::Move(p));
        }
        if mode == Mode::SpoilHauling {
            for p in [pos(1, 2), pos(1, 3), pos(0, 3)] {
                traffic_support::fixture_tick(&mut w, i, Action::Move(p));
            }
        }
        // Match the existing defensive terminal fixture: retain genuine paid yield
        // counters and overlay its consistent pending memory at a paid destination.
        w.state.agents[i].traffic = paid_yield_memory;
        super::super::controller::check(&w.setup, &w.state).unwrap();
        let before = w.state.agents[i].excavation.clone();
        let mut draws = Scripted::new(&[]);
        let d = decide_with_policies(
            &Policy::from(&w.setup),
            &w.state.agents[i],
            &traffic_support::view(&w, i),
            w.state.traffic_policy,
            w.state.excavation_policy,
            w.state.tick + 1,
            &mut draws,
        )
        .unwrap();
        assert_eq!(
            d.action,
            match mode {
                Mode::FoodReturning => Action::DepositFood,
                Mode::EmptyReturning => Action::Wait(WaitReason::EmptyArrival),
                _ => Action::DisposeSpoil,
            }
        );
        assert_eq!(d.agent.excavation, before);
        assert_eq!(d.excavation_effect, ExcavationEffect::Ordinary);
        let values = if mode == Mode::SpoilHauling {
            vec![]
        } else {
            vec![0.5, 0.5, 0.5]
        };
        let mut draws = Scripted::new(&values);
        traffic_support::decision_tick(&mut w, i, d, &mut draws);
        assert_eq!(draws.next, values.len());
        assert_eq!(w.state.agents[i].excavation, before);
    }
}
#[test]
fn exhausted_frontiers_retain_ordinary_fallback_dig_after_rejection() {
    let mut w = informed_world(
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    );
    traffic_support::controlled_tick(
        &mut w,
        0,
        Action::Wait(super::super::decision::WaitReason::NoNeighbor),
        |a| a.site = Some(pos(6, 1)),
    );
    assert_eq!(
        w.state.agents[0].map.kind(pos(6, 1)).unwrap(),
        CellKnowledge::Unknown
    );
    let (frontiers, _) =
        super::super::navigation::frontiers(&w.state.agents[0].map, w.state.agents[0].pos).unwrap();
    assert!(frontiers.is_empty());
    let before = w.state.agents[0].work.digs;
    let (d, mut draws) = choose(&w, &[0.0]);
    assert_eq!(
        (d.action, d.excavation_effect, draws.next),
        (Action::Dig(pos(3, 1)), ExcavationEffect::Ordinary, 1)
    );
    assert_eq!(
        d.agent
            .excavation
            .last_assessment
            .as_ref()
            .unwrap()
            .best_candidate,
        None
    );
    traffic_support::decision_tick(&mut w, 0, d, &mut draws);
    assert_eq!(w.state.agents[0].work.digs, before + 1);
    assert_eq!(
        w.state.agents[0].excavation.counts.selected_opportunities,
        0
    );
    assert_eq!(w.state.agents[0].excavation.counts.initiated_digs, 0);
}
#[test]
fn fresh_observation_removes_a_selected_face_opened_by_another_paid_worker() {
    let mut setup = informed_setup();
    setup.workers.push(pos(2, 2));
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    prepare(&mut w, false);
    traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(2, 2)));
    let (d, mut draws) = choose(&w, &[0.0, 0.0]);
    assert_eq!(
        (d.action, d.excavation_effect),
        (Action::Move(pos(2, 1)), ExcavationEffect::SelectedTravel)
    );
    traffic_support::decision_tick(&mut w, 0, d, &mut draws);
    let selected = w.state.agents[0].excavation.last_assessment.clone();
    traffic_support::fixture_tick(&mut w, 1, Action::Move(pos(2, 1)));
    traffic_support::fixture_tick(&mut w, 1, Action::Dig(pos(3, 1)));
    assert_eq!(w.state.agents[0].excavation.last_assessment, selected);
    assert_eq!(
        w.state.agents[0].map.kind(pos(3, 1)).unwrap(),
        CellKnowledge::KnownSolid { diggable: true }
    );
    let revisions = w.state.agents[0].compute.observed_revisions;
    let (d, draws) = choose(&w, &[0.0]);
    assert_eq!(
        (d.action, d.excavation_effect, draws.next),
        (Action::Move(pos(3, 1)), ExcavationEffect::Ordinary, 1)
    );
    assert_eq!(
        d.agent.map.kind(pos(3, 1)).unwrap(),
        CellKnowledge::KnownOpen
    );
    assert_eq!(d.agent.compute.observed_revisions, revisions + 1);
    assert_eq!(
        d.agent.excavation.last_assessment.unwrap().best_candidate,
        None
    );
}
#[test]
fn newly_remembered_solid_continuation_rejects_the_old_optimistic_selection() {
    let mut w = World::new_with_policies(
        informed_setup(),
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    // A paid partial U traversal observes (4,1) but never visits it, so the
    // continuation through (5,1) remains Unknown until a later paid approach.
    for y in 2..=5 {
        traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(3, 5)));
    for y in (2..=5).rev() {
        traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    for y in 3..=5 {
        traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(3, 5)));
    for y in (1..=5).rev() {
        traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(1, 1)));
    traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(2, 1)));
    traffic_support::controlled_tick(&mut w, 0, Action::Move(pos(2, 2)), |a| {
        a.site = Some(pos(6, 1))
    });
    assert_eq!(
        w.state.agents[0].map.kind(pos(5, 1)).unwrap(),
        CellKnowledge::Unknown
    );
    let (d, mut draws) = choose(&w, &[0.0, 0.0]);
    let old = d.agent.excavation.last_assessment.as_ref().unwrap();
    assert_eq!(old.selected_candidate.as_ref().unwrap().face, pos(3, 1));
    assert_eq!(
        old.selected_candidate.as_ref().unwrap().continuation_moves,
        3
    );
    assert_eq!(old.selected_candidate.as_ref().unwrap().estimated_cost, 9);
    assert_eq!(d.action, Action::Move(pos(2, 1)));
    traffic_support::decision_tick(&mut w, 0, d, &mut draws);
    // Controlled paid travel inspects the optimistic continuation without digging
    // the selected face or fabricating map revisions.
    for y in 2..=5 {
        traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(3, 5)));
    for y in (1..=5).rev() {
        traffic_support::fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    assert_eq!(
        w.state.agents[0].map.kind(pos(5, 1)).unwrap(),
        CellKnowledge::Unknown
    );
    let (d, draws) = choose(&w, &[]);
    assert_eq!(d.action, Action::Dig(pos(3, 1)));
    assert_eq!(
        d.agent.map.kind(pos(5, 1)).unwrap(),
        CellKnowledge::KnownSolid { diggable: false }
    );
    let current = d.agent.excavation.last_assessment.unwrap();
    assert_eq!(current.best_candidate, None);
    assert_eq!(current.selected_candidate, None);
    assert_eq!(d.excavation_effect, ExcavationEffect::Ordinary);
    assert_eq!(draws.next, 0); // unchanged fallback retains its existing face
}
#[test]
fn selected_travel_rejects_incompatible_action_and_record_combinations() {
    use super::super::decision::WaitReason;
    for case in 0..6 {
        let mut w = tied_world(TrafficPolicy::ShortestOnlyV1, false);
        let (mut d, mut draws) = choose(&w, &[0.0, 0.0]);
        match case {
            0 => d.action = Action::Wait(WaitReason::NoNeighbor),
            1 => d.action = Action::Dig(pos(3, 4)),
            2 => d.excavation_effect = ExcavationEffect::SelectedDig,
            3 => {
                d.agent
                    .excavation
                    .last_assessment
                    .as_mut()
                    .unwrap()
                    .selected_candidate
                    .as_mut()
                    .unwrap()
                    .approach = pos(2, 5)
            }
            4 => d.agent.site = None,
            5 => d.agent.phase = FoodPhase::Searching,
            _ => unreachable!(),
        }
        let o = traffic_support::view(&w, 0);
        let errors =
            super::super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut draws).unwrap_err();
        assert!(
            errors.iter().any(|e| e.field.ends_with("action")),
            "case {case}: {errors:?}"
        );
    }
}
#[test]
fn failed_local_yield_ticket_keeps_selected_travel_and_consumes_no_step_choice() {
    use super::super::{decision::WaitReason, traffic::TrafficEffect};
    let mut w = tied_world(TrafficPolicy::LocalYieldingV1, true);
    for _ in 0..2 {
        let (d, mut draws) = choose(&w, &[0.0]);
        traffic_support::decision_tick(&mut w, 0, d, &mut draws);
    }
    let (d, mut draws) = choose(&w, &[0.0, 0.9]);
    assert_eq!(
        (d.action, d.traffic_effect, d.excavation_effect, draws.next),
        (
            Action::Wait(WaitReason::Congestion),
            TrafficEffect::BlockedWait,
            ExcavationEffect::SelectedTravel,
            2
        )
    );
    traffic_support::decision_tick(&mut w, 0, d, &mut draws);
    assert_eq!(
        w.state.agents[0].excavation.counts.selected_opportunities,
        3
    );
    assert_eq!(w.state.agents[0].excavation.counts.initiated_digs, 0);
}
#[test]
fn each_assessment_counter_overflow_preserves_tick_worker_context() {
    for field in [
        "assessments",
        "faces_scanned",
        "candidate_pairs",
        "distance_calls",
        "distance_visits",
    ] {
        let mut w = informed_world(
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1,
        );
        // Assessment count has a paid bound, so its overflow is exercised through
        // the private decision seam, while unbounded compute counters also reach advance.
        let c = &mut w.state.agents[0].excavation.counts;
        match field {
            "assessments" => c.assessments = u64::MAX,
            "faces_scanned" => c.faces_scanned = u64::MAX,
            "candidate_pairs" => c.candidate_pairs = u64::MAX,
            "distance_calls" => c.distance_calls = u64::MAX,
            "distance_visits" => c.distance_visits = u64::MAX,
            _ => unreachable!(),
        }
        let mut draws = Scripted::new(&[0.0]);
        let errors = if field == "assessments" {
            decide_with_policies(
                &Policy::from(&w.setup),
                &w.state.agents[0],
                &traffic_support::view(&w, 0),
                w.state.traffic_policy,
                w.state.excavation_policy,
                w.state.tick + 1,
                &mut draws,
            )
            .unwrap_err()
        } else {
            super::super::controller::check(&w.setup, &w.state).unwrap();
            let errors = traffic_support::advance_scripted(&mut w, &mut draws).unwrap_err();
            assert!(
                errors[0]
                    .field
                    .starts_with(&format!("tick[{}].agents[0].empty.", w.state.tick)),
                "{errors:?}"
            );
            errors
        };
        assert!(
            errors
                .iter()
                .any(|e| e.field.ends_with(&format!("excavation.{field}"))
                    && e.message == "counter overflow"),
            "{errors:?}"
        );
    }
}
#[test]
fn initiated_dig_commit_overflow_is_detected_after_one_physical_transaction() {
    let mut w = informed_world(
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    );
    w.state.agents[0].excavation.counts.initiated_digs = u64::MAX;
    let (d, mut draws) = choose(&w, &[0.0]);
    let o = traffic_support::view(&w, 0);
    let errors =
        super::super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut draws).unwrap_err();
    assert_eq!(errors[0].field, "dig.spoil.food.excavation.initiated_digs");
    assert_eq!(w.state.spoil.inventory().excavated, 1);
}
#[test]
fn aggregate_counter_overflow_isolated_from_valid_worker_physics() {
    let mut setup = informed_setup();
    setup.workers.push(pos(2, 2));
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    w.state.agents[0].excavation.counts.faces_scanned = u64::MAX;
    w.state.agents[1].excavation.counts.faces_scanned = 1;
    for a in &w.state.agents {
        a.excavation
            .check(
                w.state.excavation_policy,
                &a.work,
                w.setup.width,
                w.setup.height,
                w.state.tick,
            )
            .unwrap();
    }
    let errors = super::super::controller::check(&w.setup, &w.state).unwrap_err();
    assert_eq!(errors[0].field, "agents[1].excavation.faces_scanned");
    assert_eq!(errors[0].message, "counter overflow");
}
#[test]
fn accepted_selection_rejects_exhausted_and_malformed_draws_without_mutating_worker() {
    for values in [
        vec![],
        vec![f64::NAN],
        vec![f64::INFINITY],
        vec![-0.1],
        vec![1.0],
    ] {
        let w = informed_world(
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1,
        );
        let before = w.state.agents[0].clone();
        let mut draws = Scripted::new(&values);
        let errors = decide_with_policies(
            &Policy::from(&w.setup),
            &w.state.agents[0],
            &traffic_support::view(&w, 0),
            w.state.traffic_policy,
            w.state.excavation_policy,
            w.state.tick + 1,
            &mut draws,
        )
        .unwrap_err();
        assert_eq!(errors[0].field, "draw");
        assert_eq!(draws.next, usize::from(!values.is_empty()));
        assert_eq!(w.state.agents[0], before);
    }
}
#[test]
fn invalid_excavation_diagnostics_fail_at_the_tick_worker_boundary_before_draws() {
    let mut w = informed_world(
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    );
    w.state.agents[0].excavation.counts.selected_opportunities = 1;
    let tick = w.state.tick;
    let mut draws = Scripted::new(&[]);
    let errors = traffic_support::advance_scripted(&mut w, &mut draws).unwrap_err();
    assert_eq!(
        errors[0].field,
        format!("tick[{tick}].agents[0].excavation.counts")
    );
    assert_eq!(draws.next, 0);
}
#[test]
fn spoil_disposal_resumes_site_pursuit_without_returning_to_the_old_approach() {
    let mut w = alternate_resumption_world();
    let old_approach = w.state.agents[0].pos;
    let (d, mut draws) = choose(&w, &[0.0]);
    assert_eq!(d.action, Action::Dig(pos(3, 1)));
    assert_eq!(
        d.agent
            .excavation
            .last_assessment
            .as_ref()
            .unwrap()
            .selected_candidate
            .as_ref()
            .unwrap()
            .approach,
        old_approach
    );
    traffic_support::decision_tick(&mut w, 0, d, &mut draws);
    let token = w.state.agents[0].cargo;
    assert!(matches!(token, Some(Cargo::Spoil(_))));
    traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    assert_eq!(w.state.agents[0].pos, pos(2, 2));
    assert_eq!(w.state.agents[0].cargo, token);
    traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[])).unwrap();
    assert_eq!(w.state.agents[0].cargo, None);
    let (d, mut draws) = choose(&w, &[0.9]);
    assert_eq!(d.action, Action::Move(pos(3, 2)));
    assert_ne!(pos(3, 2), old_approach);
    assert_eq!(d.excavation_effect, ExcavationEffect::Ordinary);
    assert_eq!(
        d.agent
            .excavation
            .last_assessment
            .as_ref()
            .unwrap()
            .selected_candidate,
        None
    );
    assert_eq!(draws.next, 1);
    traffic_support::decision_tick(&mut w, 0, d, &mut draws);
    let (d, draws) = choose(&w, &[0.0]);
    assert_eq!(d.action, Action::Move(pos(3, 1)));
    assert_eq!(draws.next, 1);
    assert_eq!(w.state.spoil.inventory().disposed, 1);
    assert_eq!(w.state.agents[0].excavation.counts.initiated_digs, 1);
    assert_eq!(w.state.agents[0].site, Some(pos(4, 1)));
}
#[test]
fn selected_effect_rejects_immediate_pursuit_transitions_even_with_a_current_record() {
    for site in [pos(2, 1), pos(2, 0)] {
        let mut w = informed_world(
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1,
        );
        let (mut d, mut draws) = choose(&w, &[0.0]);
        // Negative effect validation through the existing private seam: only the
        // intent/assessment labels are corrupted; physical state and work remain valid.
        w.state.agents[0].site = Some(site);
        d.agent.site = Some(site);
        d.agent.excavation.last_assessment.as_mut().unwrap().site = site;
        super::super::controller::check(&w.setup, &w.state).unwrap();
        let o = traffic_support::view(&w, 0);
        assert!(
            super::super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut draws).is_err()
        );
    }
}
#[test]
fn existing_constructors_and_explicit_legacy_policies_keep_zero_diagnostics_and_rng() {
    use rand::Rng;
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let mut old = World::new_with_traffic(setup(), 7, traffic).unwrap();
        let mut explicit =
            World::new_with_policies(setup(), 7, traffic, ExcavationPolicy::ExplorationFirstV1)
                .unwrap();
        assert_eq!(old.state, explicit.state);
        for _ in 0..40 {
            old.step().unwrap();
            explicit.step().unwrap();
            assert_eq!(old.state, explicit.state);
            for a in &old.state.agents {
                assert_eq!(a.excavation.counts, ExcavationCounts::default());
                assert_eq!(a.excavation.last_assessment, None);
            }
        }
        assert_eq!(old.rng.gen::<u64>(), explicit.rng.gen::<u64>());
        if traffic == TrafficPolicy::ShortestOnlyV1 {
            let mut ordinary = World::new(setup(), 7).unwrap();
            for _ in 0..40 {
                ordinary.step().unwrap();
            }
            assert_eq!(ordinary.state, old.state);
        }
    }
}
