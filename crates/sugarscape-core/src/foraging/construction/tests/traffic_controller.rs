use super::super::{
    controller::check,
    decision::{decide_with_traffic, Action, Policy, WaitReason},
    traffic::TrafficEffect,
};
use super::traffic_support::*;
use super::*;
#[test]
fn paid_opposing_queue_yields_on_third_then_holds_and_passes_both_directions() {
    let mut revised = opposing_queue(TrafficPolicy::LocalYieldingV1);
    let mut legacy = revised.clone();
    legacy.state.traffic_policy = TrafficPolicy::ShortestOnlyV1;
    let initial = revised.state.tick;
    for _ in 0..2 {
        for w in [&mut revised, &mut legacy] {
            let before = w.state.agents.clone();
            let mut d = Scripted::new(&[0.5, 0.5]);
            advance_scripted(w, &mut d).unwrap();
            for (a, b) in w.state.agents.iter().zip(&before) {
                assert_eq!(a.work.opportunities, b.work.opportunities + 1);
                assert_eq!(a.work.waits, b.work.waits + 1);
                assert_eq!(
                    a.work.empty_congestion_waits,
                    b.work.empty_congestion_waits + 1
                );
                assert_eq!(a.work.moves, b.work.moves);
                assert_eq!(
                    a.traffic.counts.blocked_route_opportunities,
                    b.traffic.counts.blocked_route_opportunities + 1
                );
                assert_eq!(a.compute.observations, b.compute.observations + 1);
            }
            assert_eq!(d.next, 2);
            assert_eq!(
                w.state.agents.iter().map(|a| a.pos).collect::<Vec<_>>(),
                vec![pos(2, 2), pos(2, 2), pos(2, 1), pos(2, 1)]
            );
            assert!(w
                .state
                .agents
                .iter()
                .all(|a| a.traffic.counts.yield_attempts == 0));
        }
    }
    assert!(revised
        .state
        .agents
        .iter()
        .all(|a| a.traffic.blocked_wait_streak == 2));
    for (i, goals, expected) in [
        (0, vec![pos(2, 1)], vec![pos(2, 3), pos(3, 2), pos(1, 2)]),
        (2, revised.setup.nest.clone(), vec![pos(2, 0)]),
    ] {
        let a = &revised.state.agents[i];
        let super::super::navigation::RoutePlan::Reachable {
            free_shortest,
            yield_candidates,
            ..
        } = super::super::navigation::plan_route(&a.map, a.pos, &goals, &view(&revised, i))
            .unwrap()
        else {
            panic!("queue route must be reachable")
        };
        assert!(free_shortest.is_empty());
        assert_eq!(yield_candidates, expected);
    }
    let mut d = Scripted::new(QUEUE_THIRD_TICK_SCRIPT);
    advance_scripted(&mut revised, &mut d).unwrap();
    assert_eq!(d.next, 7);
    assert_eq!(
        revised
            .state
            .agents
            .iter()
            .map(|a| a.pos)
            .collect::<Vec<_>>(),
        vec![pos(3, 2), pos(2, 2), pos(2, 2), pos(2, 1)]
    );
    assert!(revised.state.agents[0].traffic.hold_pending);
    assert_eq!(revised.state.agents[0].frontier, Some(pos(2, 1)));
    let observations = revised.state.agents[0].compute.observations;
    let mut d = Scripted::new(QUEUE_FOURTH_TICK_SCRIPT);
    advance_scripted(&mut revised, &mut d).unwrap();
    assert_eq!(d.next, 6);
    assert_eq!(
        revised
            .state
            .agents
            .iter()
            .map(|a| a.pos)
            .collect::<Vec<_>>(),
        vec![pos(3, 2), pos(2, 1), pos(2, 2), pos(2, 2)]
    );
    assert_eq!(
        revised.state.agents[0].compute.observations,
        observations + 1
    );
    assert_eq!(revised.state.agents[0].traffic.counts.hold_waits, 1);
    assert!(!revised.state.agents[0].traffic.hold_pending);
    assert_eq!(revised.state.tick, initial + 4);
    for _ in 0..2 {
        let mut d = Scripted::new(&[0.5, 0.5]);
        advance_scripted(&mut legacy, &mut d).unwrap();
        assert_eq!(d.next, 2);
    }
    assert_eq!(
        legacy
            .state
            .agents
            .iter()
            .map(|a| a.pos)
            .collect::<Vec<_>>(),
        vec![pos(2, 2), pos(2, 2), pos(2, 1), pos(2, 1)]
    );
    for w in [&revised, &legacy] {
        check(&w.setup, &w.state).unwrap();
        assert!(super::super::controller::occupancy(&w.state)
            .values()
            .all(|n| *n <= 2));
    }
}
#[test]
fn all_modes_hold_once_learn_and_retain_job_without_draws() {
    for mode in [
        Mode::Departing,
        Mode::Searching,
        Mode::EmptyReturning,
        Mode::FoodReturning,
        Mode::SpoilHauling,
    ] {
        let (mut w, i, o) = held_worker(mode);
        let before = w.state.agents[i].clone();
        let inv_food = w.state.food.inventory();
        let inv_spoil = w.state.spoil.inventory();
        let mut draws = Scripted::new(&[]);
        let decision = decide_with_traffic(
            &Policy::from(&w.setup),
            &before,
            &o,
            TrafficPolicy::LocalYieldingV1,
            &mut draws,
        )
        .unwrap();
        assert_eq!(decision.traffic_effect, TrafficEffect::Hold);
        assert_eq!(decision.action, Action::Wait(WaitReason::TrafficHold));
        assert_eq!(decision.agent.cargo, before.cargo);
        assert_eq!(decision.agent.find, before.find);
        assert_eq!(
            (
                decision.agent.phase,
                decision.agent.site,
                decision.agent.frontier,
                decision.agent.face
            ),
            (before.phase, before.site, before.frontier, before.face)
        );
        assert_eq!(
            decision.agent.compute.observations,
            before.compute.observations + 1
        );
        assert_eq!(draws.next, 0);
        decision_tick(&mut w, i, decision, &mut draws);
        let after = &w.state.agents[i];
        assert_eq!(
            after.traffic.counts.hold_waits,
            before.traffic.counts.hold_waits + 1
        );
        assert!(!after.traffic.hold_pending);
        assert_eq!(after.work.opportunities, before.work.opportunities + 1);
        assert_eq!(after.work.waits, before.work.waits + 1);
        assert_eq!(after.work.moves, before.work.moves);
        assert_eq!(
            (
                after.work.pickups,
                after.work.digs,
                after.work.deposits,
                after.work.disposals,
                after.work.search_entries
            ),
            (
                before.work.pickups,
                before.work.digs,
                before.work.deposits,
                before.work.disposals,
                before.work.search_entries
            )
        );
        let category = |a: &super::super::state::Agent| match mode {
            Mode::FoodReturning => a.work.food_congestion_waits,
            Mode::SpoilHauling => a.work.spoil_congestion_waits,
            _ => a.work.empty_congestion_waits,
        };
        assert_eq!(category(after), category(&before) + 1);
        assert_eq!(after.work.publications, before.work.publications);
        assert_eq!(
            after.work.uninformed_departures,
            before.work.uninformed_departures
        );
        assert_eq!(w.state.food.inventory(), inv_food);
        assert_eq!(w.state.spoil.inventory(), inv_spoil);
        check(&w.setup, &w.state).unwrap();
    }
}
#[test]
fn terminal_arrivals_override_defensive_pending_hold_with_original_cadence() {
    for mode in [
        Mode::FoodReturning,
        Mode::EmptyReturning,
        Mode::SpoilHauling,
    ] {
        let (mut w, i, _) = held_worker(mode);
        let paid_yield_memory = w.state.agents[i].traffic.clone();
        // All physical destinations/cargo are reached via paid transactions.
        fixture_tick(&mut w, 0, Action::Move(pos(3, 2)));
        for p in [pos(2, 1), pos(2, 2)] {
            fixture_tick(&mut w, i, Action::Move(p));
        }
        if mode == Mode::SpoilHauling {
            for p in [pos(1, 2), pos(1, 3), pos(0, 3)] {
                fixture_tick(&mut w, i, Action::Move(p));
            }
        }
        // Defensive synthetic traffic-state control, not a natural yielding arrival:
        // a goal neighbor is ordinarily a free shortest step. Retain the genuine
        // prior paid-yield counters and overlay its consistent pending memory.
        w.state.agents[i].traffic = paid_yield_memory;
        check(&w.setup, &w.state).unwrap();
        let before = w.state.agents[i].clone();
        let o = view(&w, i);
        let mut selection = Scripted::new(&[]);
        let decision = decide_with_traffic(
            &Policy::from(&w.setup),
            &before,
            &o,
            w.state.traffic_policy,
            &mut selection,
        )
        .unwrap();
        let expected = match mode {
            Mode::FoodReturning => Action::DepositFood,
            Mode::EmptyReturning => Action::Wait(WaitReason::EmptyArrival),
            _ => Action::DisposeSpoil,
        };
        assert_eq!(
            (decision.action, decision.traffic_effect, selection.next),
            (expected, TrafficEffect::Normal, 0)
        );
        let values = if mode == Mode::SpoilHauling {
            vec![]
        } else {
            vec![0.5, 0.5, 0.5]
        };
        let mut draws = Scripted::new(&values);
        decision_tick(&mut w, i, decision, &mut draws);
        let after = &w.state.agents[i];
        assert_eq!(draws.next, values.len());
        assert_eq!(after.cargo, None);
        assert_eq!(
            after.traffic.counts.hold_waits,
            before.traffic.counts.hold_waits
        );
        assert!(!after.traffic.hold_pending);
        assert_eq!(after.traffic.blocked_wait_streak, 0);
        if mode == Mode::SpoilHauling {
            assert_eq!(after.phase, before.phase);
            assert_eq!(after.work.disposals, before.work.disposals + 1);
            assert_eq!(
                after.work.uninformed_departures,
                before.work.uninformed_departures
            );
        } else {
            assert_eq!(after.phase, FoodPhase::Departing);
            if mode == Mode::FoodReturning {
                // Existing lambda=0 Poisson CDF publishes and selects fidelity.
                assert_eq!(
                    after.work.fidelity_departures,
                    before.work.fidelity_departures + 1
                );
                assert_eq!(after.work.publications, before.work.publications + 1);
                assert_eq!(after.site, Some(pos(2, 1)));
            } else {
                assert_eq!(
                    after.work.uninformed_departures,
                    before.work.uninformed_departures + 1
                );
            }
        }
        check(&w.setup, &w.state).unwrap();
    }
}
#[test]
fn corrupt_traffic_state_is_rejected_with_tick_worker_context() {
    for corruption in 0..4 {
        let mut w = opposing_queue(TrafficPolicy::LocalYieldingV1);
        let a = &mut w.state.agents[0];
        match corruption {
            0 => a.traffic.blocked_wait_streak = 3,
            1 => {
                a.traffic.hold_pending = true;
                a.traffic.blocked_wait_streak = 1;
            }
            2 => {
                w.state.traffic_policy = TrafficPolicy::ShortestOnlyV1;
                a.traffic.hold_pending = true;
            }
            _ => {
                w.state.traffic_policy = TrafficPolicy::ShortestOnlyV1;
                a.traffic.counts.yield_attempts = 1;
            }
        }
        let mut draws = Scripted::new(&[]);
        let errors = advance_scripted(&mut w, &mut draws).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field.contains("tick[") && e.field.contains("agents[0].traffic")));
        assert_eq!(draws.next, 0);
    }
}
#[test]
fn ordinary_transition_and_no_neighbor_wait_reset_paid_blocked_streak() {
    for mode in [Mode::Departing, Mode::Searching] {
        let mut w = opposing_queue(TrafficPolicy::LocalYieldingV1);
        for _ in 0..2 {
            let mut d = Scripted::new(&[0.5, 0.5]);
            advance_scripted(&mut w, &mut d).unwrap();
        }
        assert_eq!(w.state.agents[0].traffic.blocked_wait_streak, 2);
        controlled_tick(
            &mut w,
            0,
            Action::Wait(if mode == Mode::Departing {
                WaitReason::Transition
            } else {
                WaitReason::NoNeighbor
            }),
            |_| {},
        );
        assert_eq!(w.state.agents[0].traffic.blocked_wait_streak, 0);
    }
}
#[test]
fn hold_on_food_cell_defers_pickup_then_resumes_search() {
    let (mut w, i, o) = held_worker(Mode::Searching);
    assert!(o
        .cells
        .iter()
        .any(|c| c.pos == w.state.agents[i].pos && c.food));
    let before = w.state.agents[i].clone();
    let mut draws = Scripted::new(&[]);
    let decision = decide_with_traffic(
        &Policy::from(&w.setup),
        &before,
        &o,
        w.state.traffic_policy,
        &mut draws,
    )
    .unwrap();
    assert_eq!(decision.traffic_effect, TrafficEffect::Hold);
    decision_tick(&mut w, i, decision, &mut draws);
    assert_eq!(w.state.agents[i].cargo, None);
    assert_eq!(w.state.agents[i].work.pickups, before.work.pickups);
    let o = view(&w, i);
    let mut draws = Scripted::new(&[0.5]);
    let decision = decide_with_traffic(
        &Policy::from(&w.setup),
        &w.state.agents[i],
        &o,
        w.state.traffic_policy,
        &mut draws,
    )
    .unwrap();
    assert_eq!(
        (decision.action, decision.traffic_effect, draws.next),
        (Action::PickupFood, TrafficEffect::Normal, 1)
    );
    decision_tick(&mut w, i, decision, &mut draws);
    assert!(matches!(w.state.agents[i].cargo, Some(Cargo::Food(_))));
    check(&w.setup, &w.state).unwrap();
}
