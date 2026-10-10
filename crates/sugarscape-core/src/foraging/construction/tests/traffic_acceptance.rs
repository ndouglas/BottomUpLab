use super::super::{
    controller::{advance, check, occupancy},
    decision::{decide_with_traffic, Action, Policy},
    draws::PcgDraws,
    traffic::TrafficEffect,
};
use super::traffic_support::*;
use super::*;
use rand::Rng;

fn threshold_queue() -> World {
    let mut w = opposing_queue(TrafficPolicy::LocalYieldingV1);
    for _ in 0..2 {
        advance_scripted(&mut w, &mut Scripted::new(&[0.5, 0.5])).unwrap();
    }
    w
}

#[test]
fn legacy_public_steps_preserve_every_snapshot_and_raw_rng() {
    let mut ordinary = World::new(setup(), 7).unwrap();
    let mut explicit = World::new_with_traffic(setup(), 7, TrafficPolicy::ShortestOnlyV1).unwrap();
    for _ in 0..40 {
        ordinary.step().unwrap();
        explicit.step().unwrap();
        assert_eq!(ordinary.snapshot().unwrap(), explicit.snapshot().unwrap());
    }
    assert_eq!(ordinary.rng.gen::<u64>(), 15278877981002486553);
    assert_eq!(explicit.rng.gen::<u64>(), 15278877981002486553);
    let mut queue = opposing_queue(TrafficPolicy::ShortestOnlyV1);
    for _ in 0..4 {
        advance_scripted(&mut queue, &mut Scripted::new(&[0.5, 0.5])).unwrap();
    }
    let counts = queue.traffic_summary().unwrap().counts;
    assert_eq!(counts.blocked_route_opportunities, 16);
    assert_eq!(
        (counts.yield_attempts, counts.yield_moves, counts.hold_waits),
        (0, 0, 0)
    );
}

#[test]
fn shared_tick_rolls_back_after_an_actual_earlier_yield() {
    let mut w = threshold_queue();
    let before = w.state.clone();
    let mut scripted = Scripted::new(&QUEUE_THIRD_TICK_SCRIPT[..5]);
    let before_draws = scripted.clone();
    let mut scratch = before.clone();
    let mut scratch_draws = scripted.clone();
    let errors = advance(&w.setup, &mut scratch, &mut scratch_draws).unwrap_err();
    assert!(errors.iter().any(|e| e.field.contains("agents[2]")));
    assert_eq!(
        scratch.agents[0].traffic.counts.yield_moves,
        before.agents[0].traffic.counts.yield_moves + 1
    );
    assert!(scratch.agents[0].traffic.hold_pending);
    assert_eq!(scratch.agents[0].pos, pos(3, 2));
    assert!(advance_scripted(&mut w, &mut scripted).is_err());
    assert_eq!(w.state, before);
    assert_eq!(scripted.next, before_draws.next);
    assert_eq!(scripted.values, before_draws.values);
}

#[test]
fn public_step_rolls_back_after_actual_hold_then_recovers_with_identical_rng() {
    let mut w = threshold_queue();
    advance_scripted(&mut w, &mut Scripted::new(QUEUE_THIRD_TICK_SCRIPT)).unwrap();
    let mut control = w.clone();
    let observations = w.state.agents[3].compute.observations;
    w.state.agents[3].compute.observations = u64::MAX;
    check(&w.setup, &w.state).unwrap();
    let before = w.state.clone();
    let mut before_rng = w.rng.clone();
    let mut scratch = before.clone();
    let mut scratch_rng = w.rng.clone();
    let errors = advance(&w.setup, &mut scratch, &mut PcgDraws(&mut scratch_rng)).unwrap_err();
    assert!(errors.iter().any(|e| e.field.contains("agents[3]")));
    assert_eq!(
        scratch.agents[0].traffic.counts.hold_waits,
        before.agents[0].traffic.counts.hold_waits + 1
    );
    assert!(!scratch.agents[0].traffic.hold_pending);
    assert_eq!(
        scratch.agents[0].work.waits,
        before.agents[0].work.waits + 1
    );
    // The later worker fails only after other candidate workers consume seeded draws.
    assert_eq!(
        scratch.agents[1].compute.observations,
        before.agents[1].compute.observations + 1
    );
    assert_eq!(
        scratch.agents[2].compute.observations,
        before.agents[2].compute.observations + 1
    );
    assert_ne!(scratch_rng.clone().gen::<u64>(), w.rng.clone().gen::<u64>());
    assert!(w.step().is_err());
    assert_eq!(w.state, before);
    assert_eq!(w.rng.clone().gen::<u64>(), before_rng.gen::<u64>());
    w.state.agents[3].compute.observations = observations;
    w.step().unwrap();
    control.step().unwrap();
    assert_eq!(w.state, control.state);
    assert_eq!(w.rng.gen::<u64>(), control.rng.gen::<u64>());
}

fn remote_world(change: usize) -> World {
    let mut s = Setup {
        width: 5,
        height: 4,
        open: (0..4)
            .flat_map(|y| (0..5).map(move |x| pos(x, y)))
            .filter(|p| *p != pos(3, 0) && *p != pos(4, 0))
            .collect(),
        diggable: vec![pos(3, 0)],
        nest: vec![
            pos(0, 0),
            pos(0, 1),
            pos(0, 2),
            pos(0, 3),
            pos(1, 3),
            pos(2, 3),
            pos(3, 3),
            pos(4, 3),
        ],
        waste: pos(1, 1),
        workers: vec![pos(0, 0), pos(3, 3)],
        food: vec![Resource {
            id: 42,
            pos: pos(3, 2),
        }],
        parameters: queue_setup().parameters,
    };
    match change {
        1 => s.workers[1] = pos(4, 3),
        2 => s.food[0].pos = pos(4, 2),
        3 => s.diggable = vec![pos(4, 0)],
        _ => (),
    }
    let mut w = World::new_with_traffic(s, 7, TrafficPolicy::LocalYieldingV1).unwrap();
    controlled_tick(
        &mut w,
        0,
        super::super::decision::Action::Wait(super::super::decision::WaitReason::NoNeighbor),
        |agent| {
            agent.phase = FoodPhase::Searching;
            agent.frontier = Some(pos(1, 0));
        },
    );
    w
}

#[test]
fn remote_occupancy_food_and_diggability_cannot_change_private_decisions_or_draws() {
    let base = remote_world(0);
    for change in 1..=3 {
        let other = remote_world(change);
        check(&other.setup, &other.state).unwrap();
        assert_ne!(base.state, other.state);
        assert_eq!(base.state.agents[0], other.state.agents[0]);
        let a = view(&base, 0);
        let b = view(&other, 0);
        assert_eq!(a, b);
        let mut da = Scripted::new(&[0.5, 0.5]);
        let mut db = da.clone();
        let decision_a = decide_with_traffic(
            &Policy::from(&base.setup),
            &base.state.agents[0],
            &a,
            base.state.traffic_policy,
            &mut da,
        )
        .unwrap();
        let decision_b = decide_with_traffic(
            &Policy::from(&other.setup),
            &other.state.agents[0],
            &b,
            other.state.traffic_policy,
            &mut db,
        )
        .unwrap();
        assert_eq!(decision_a, decision_b);
        assert_eq!(decision_a.action, Action::Move(pos(1, 0)));
        assert_eq!((da.next, db.next), (2, 2));
    }
}

#[test]
fn freshly_changed_local_capacity_selects_only_a_legal_shortest_step() {
    let blocked = threshold_queue();
    let mut freed = blocked.clone();
    fixture_tick(&mut freed, 2, Action::Move(pos(2, 0)));
    let agent = &blocked.state.agents[0];
    let a = view(&blocked, 0);
    let b = view(&freed, 0);
    assert_ne!(a, b);
    let mut da = Scripted::new(&[0.5, 0.0, 0.5]);
    let mut db = Scripted::new(&[0.5, 0.0]);
    let yielding = decide_with_traffic(
        &Policy::from(&blocked.setup),
        agent,
        &a,
        blocked.state.traffic_policy,
        &mut da,
    )
    .unwrap();
    let shortest = decide_with_traffic(
        &Policy::from(&freed.setup),
        agent,
        &b,
        freed.state.traffic_policy,
        &mut db,
    )
    .unwrap();
    assert_eq!(
        (yielding.action, yielding.traffic_effect, da.next),
        (Action::Move(pos(3, 2)), TrafficEffect::YieldMove, 3)
    );
    assert_eq!(
        (shortest.action, shortest.traffic_effect, db.next),
        (Action::Move(pos(2, 1)), TrafficEffect::Normal, 2)
    );
    assert!(occupancy(&freed.state)[&pos(2, 1)] < 2);
}

fn isolated_bound(case: usize, expected_field: &str) {
    let mut w = if case == 5 {
        opposing_queue(TrafficPolicy::LocalYieldingV1)
    } else {
        threshold_queue()
    };
    let work = &w.state.agents[0].work;
    work.check().unwrap();
    let mut c = TrafficCounts {
        blocked_route_opportunities: 2,
        yield_attempts: 1,
        yield_moves: 1,
        hold_waits: 0,
    };
    match case {
        0 => c.yield_attempts = 0,
        1 => c.yield_attempts = 3,
        2 => c.hold_waits = 2,
        3 => c.blocked_route_opportunities = work.opportunities + 1,
        4 => {
            c.yield_moves = work.moves + 1;
            c.yield_attempts = c.yield_moves;
            c.blocked_route_opportunities = c.yield_moves;
        }
        _ => c.hold_waits = 1,
    }
    let congestion =
        work.empty_congestion_waits + work.food_congestion_waits + work.spoil_congestion_waits;
    let violations = [
        c.yield_moves > c.yield_attempts,
        c.yield_attempts > c.blocked_route_opportunities,
        c.hold_waits > c.yield_moves,
        c.blocked_route_opportunities > work.opportunities,
        c.yield_moves > work.moves,
        c.hold_waits > congestion,
    ];
    assert_eq!(violations.iter().filter(|v| **v).count(), 1);
    assert!(violations[case]);
    w.state.agents[0].traffic.counts = c;
    let errors = check(&w.setup, &w.state).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].field.ends_with(expected_field));
}
#[test]
fn yield_moves_cannot_exceed_attempts_in_isolation() {
    isolated_bound(0, "yield_moves");
}
#[test]
fn attempts_cannot_exceed_blocked_routes_in_isolation() {
    isolated_bound(1, "yield_attempts");
}
#[test]
fn holds_cannot_exceed_yield_moves_in_isolation() {
    isolated_bound(2, "hold_waits");
}
#[test]
fn blocked_routes_cannot_exceed_opportunities_in_isolation() {
    isolated_bound(3, "blocked_route_opportunities");
}
#[test]
fn yield_moves_cannot_exceed_paid_moves_in_isolation() {
    isolated_bound(4, "yield_moves");
}
#[test]
fn holds_cannot_exceed_paid_congestion_in_isolation() {
    isolated_bound(5, "hold_waits");
}

#[test]
fn opposing_queue_progress_preserves_capacity_material_and_full_work_sums_each_tick() {
    let mut w = opposing_queue(TrafficPolicy::LocalYieldingV1);
    let food = w.state.food.inventory();
    let spoil = w.state.spoil.inventory();
    for values in [
        &[0.5, 0.5][..],
        &[0.5, 0.5][..],
        QUEUE_THIRD_TICK_SCRIPT,
        QUEUE_FOURTH_TICK_SCRIPT,
    ] {
        let before = w.summary().unwrap().work.opportunities;
        advance_scripted(&mut w, &mut Scripted::new(values)).unwrap();
        let mut total = WorkCounts::default();
        for a in &w.state.agents {
            a.work.check().unwrap();
            total.checked_include(&a.work).unwrap();
        }
        total.check().unwrap();
        assert_eq!(w.summary().unwrap().work, total);
        assert_eq!(total.opportunities, before + 4);
        assert!(occupancy(&w.state).values().all(|n| *n <= 2));
        assert_eq!(w.state.food.inventory(), food);
        assert_eq!(w.state.spoil.inventory(), spoil);
    }
    assert_eq!(w.state.agents[1].pos, pos(2, 1));
    assert_eq!(w.state.agents[2].pos, pos(2, 2));
    assert_eq!(w.state.agents[3].pos, pos(2, 2));
    assert_eq!(w.traffic_summary().unwrap().counts.hold_waits, 1);
}

#[test]
fn every_public_traffic_label_matches_the_explicit_profile() {
    for policy in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let w = World::new_with_traffic(setup(), 7, policy).unwrap();
        assert_eq!(w.traffic_summary().unwrap().policy, policy);
        assert_eq!(w.traffic_snapshot().unwrap().policy, policy);
        let e = run_with_traffic(
            setup(),
            7,
            RunOptions {
                ticks: 40,
                sample_every: 7,
                snapshots: true,
            },
            policy,
        )
        .unwrap();
        assert_eq!(e.policy, policy);
        assert_eq!(e.traffic_summary.policy, policy);
        assert!(e.traffic_snapshots.iter().all(|s| s.policy == policy));
    }
}
