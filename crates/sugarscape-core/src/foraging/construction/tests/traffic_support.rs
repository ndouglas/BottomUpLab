//! Paid fixture actions and explicitly controlled trip intent; no ledger/position teleportation.
use super::super::{
    actions::apply,
    controller::{advance, check, occupancy},
    decision::{decide_with_traffic, Action, Decision, Policy, WaitReason},
    state::Agent,
    traffic::TrafficEffect,
};
use super::*;
pub(super) const QUEUE_THIRD_TICK_SCRIPT: &[f64] = &[0.5, 0.0, 0.5, 0.5, 0.9, 0.0, 0.9];
pub(super) const QUEUE_FOURTH_TICK_SCRIPT: &[f64] = &[0.5, 0.0, 0.5, 0.5, 0.5, 0.0];
pub(super) fn queue_setup() -> Setup {
    Setup {
        width: 5,
        height: 4,
        open: vec![
            pos(1, 2),
            pos(1, 3),
            pos(2, 2),
            pos(2, 3),
            pos(0, 3),
            pos(2, 1),
            pos(2, 0),
            pos(3, 2),
        ],
        diggable: vec![],
        nest: vec![pos(1, 2), pos(1, 3), pos(2, 2), pos(2, 3)],
        waste: pos(0, 3),
        workers: vec![pos(2, 2), pos(2, 2), pos(1, 2), pos(1, 2)],
        food: vec![],
        parameters: Parameters {
            p_search: 1.0,
            p_return: 0.0,
            lambda_fidelity: 0.0,
            lambda_publish: 0.0,
            lambda_waypoint: 0.0,
        },
    }
}
pub(super) fn view(w: &World, i: usize) -> Observation {
    observe(
        &w.state.terrain,
        w.state.agents[i].pos,
        &occupancy(&w.state),
        &w.state.food.available(),
    )
    .unwrap()
}
pub(super) fn advance_scripted(world: &mut World, draws: &mut Scripted) -> Checked<()> {
    let mut state = world.state.clone();
    let mut candidate_draws = draws.clone();
    advance(&world.setup, &mut state, &mut candidate_draws)?;
    world.state = state;
    *draws = candidate_draws;
    Ok(())
}
fn learn(w: &World, i: usize, o: &Observation) -> Agent {
    let mut agent = w.state.agents[i].clone();
    let delta = agent.map.learn(o).unwrap();
    agent
        .compute
        .checked_include(&ComputeCounts {
            observations: 1,
            cells_inspected: o.cells.len() as u64,
            cells_learned: delta.first,
            observed_revisions: delta.observed_revisions,
            dig_confirmations: delta.dig_confirmations,
            ..Default::default()
        })
        .unwrap();
    agent
}
pub(super) fn controlled_tick(
    w: &mut World,
    i: usize,
    action: Action,
    control: impl FnOnce(&mut Agent),
) {
    let mut control = Some(control);
    for j in 0..w.state.agents.len() {
        let o = view(w, j);
        let mut agent = learn(w, j, &o);
        let action = if i == j {
            control.take().unwrap()(&mut agent);
            action
        } else {
            Action::Wait(WaitReason::NoNeighbor)
        };
        apply(
            &w.setup,
            &mut w.state,
            j,
            Decision {
                agent,
                action,
                traffic_effect: TrafficEffect::Normal,
            },
            &o,
            &mut Scripted::new(&[]),
        )
        .unwrap();
    }
    w.state.tick += 1;
    check(&w.setup, &w.state).unwrap();
}
pub(super) fn fixture_tick(w: &mut World, i: usize, action: Action) {
    controlled_tick(w, i, action, |_| {});
}
fn prepare_queue(w: &mut World) {
    for (i, p) in [
        (0, pos(3, 2)),
        (2, pos(2, 2)),
        (2, pos(2, 1)),
        (3, pos(2, 2)),
        (3, pos(2, 1)),
        (0, pos(2, 2)),
    ] {
        fixture_tick(w, i, Action::Move(p));
    }
    // Test controls set trip intent through checked paid Decisions, not a seeded trajectory.
    for i in 0..4 {
        controlled_tick(w, i, Action::Wait(WaitReason::NoNeighbor), |a| {
            a.phase = if i < 2 {
                FoodPhase::Searching
            } else {
                FoodPhase::Returning
            };
            a.frontier = if i < 2 { Some(pos(2, 1)) } else { None };
            a.site = None;
            a.face = None;
        });
    }
    check(&w.setup, &w.state).unwrap();
}
pub(super) fn opposing_queue(policy: TrafficPolicy) -> World {
    let mut w = World::new_with_traffic(queue_setup(), 7, policy).unwrap();
    prepare_queue(&mut w);
    w
}
fn selected_tick(w: &mut World, i: usize, values: &[f64]) -> usize {
    let mut draws = Scripted::new(values);
    for j in 0..w.state.agents.len() {
        let o = view(w, j);
        let decision = if i == j {
            decide_with_traffic(
                &Policy::from(&w.setup),
                &w.state.agents[j],
                &o,
                w.state.traffic_policy,
                &mut draws,
            )
            .unwrap()
        } else {
            Decision {
                agent: learn(w, j, &o),
                action: Action::Wait(WaitReason::NoNeighbor),
                traffic_effect: TrafficEffect::Normal,
            }
        };
        let before = w.state.agents[j].clone();
        let yielding = decision.traffic_effect == TrafficEffect::YieldMove;
        if yielding {
            assert_eq!(
                (
                    decision.agent.cargo,
                    decision.agent.find,
                    decision.agent.phase,
                    decision.agent.site,
                    decision.agent.frontier,
                    decision.agent.face
                ),
                (
                    before.cargo,
                    before.find,
                    before.phase,
                    before.site,
                    before.frontier,
                    before.face
                )
            );
        }
        apply(&w.setup, &mut w.state, j, decision, &o, &mut draws).unwrap();
        if yielding {
            let after = &w.state.agents[j];
            assert_eq!(
                (
                    after.cargo,
                    after.find,
                    after.phase,
                    after.site,
                    after.frontier,
                    after.face
                ),
                (
                    before.cargo,
                    before.find,
                    before.phase,
                    before.site,
                    before.frontier,
                    before.face
                )
            );
            assert_eq!(after.work.moves, before.work.moves + 1);
            let category = |a: &Agent| match before.mode() {
                Mode::Departing => a.work.departure_moves,
                Mode::Searching => a.work.search_moves,
                Mode::EmptyReturning => a.work.empty_return_moves,
                Mode::FoodReturning => a.work.food_moves,
                Mode::SpoilHauling => a.work.spoil_moves,
            };
            assert_eq!(category(after), category(&before) + 1);
            assert_eq!(
                (
                    after.work.pickups,
                    after.work.digs,
                    after.work.disposals,
                    after.work.deposits,
                    after.work.publications,
                    after.work.uninformed_departures
                ),
                (
                    before.work.pickups,
                    before.work.digs,
                    before.work.disposals,
                    before.work.deposits,
                    before.work.publications,
                    before.work.uninformed_departures
                )
            );
            assert_eq!(
                after.traffic.counts.yield_moves,
                before.traffic.counts.yield_moves + 1
            );
            assert_eq!(after.traffic.blocked_wait_streak, 0);
        }
    }
    w.state.tick += 1;
    check(&w.setup, &w.state).unwrap();
    draws.next
}
// A preselected worker decision still pays within the complete ascending-ID tick.
pub(super) fn decision_tick(w: &mut World, i: usize, decision: Decision, draws: &mut Scripted) {
    let mut decision = Some(decision);
    for j in 0..w.state.agents.len() {
        let o = view(w, j);
        let selected = if i == j {
            decision.take().unwrap()
        } else {
            Decision {
                agent: learn(w, j, &o),
                action: Action::Wait(WaitReason::NoNeighbor),
                traffic_effect: TrafficEffect::Normal,
            }
        };
        apply(&w.setup, &mut w.state, j, selected, &o, draws).unwrap();
    }
    w.state.tick += 1;
    check(&w.setup, &w.state).unwrap();
}
pub(super) fn held_worker(mode: Mode) -> (World, usize, Observation) {
    let mut setup = queue_setup();
    setup.food = vec![Resource {
        id: 43,
        pos: pos(3, 2),
    }];
    setup.diggable = vec![pos(4, 2), pos(3, 0)];
    if mode == Mode::FoodReturning {
        setup.food.push(Resource {
            id: 42,
            pos: pos(2, 1),
        });
    }
    if mode == Mode::SpoilHauling {
        setup.diggable.push(pos(3, 1));
    }
    let mut w = World::new_with_traffic(setup, 7, TrafficPolicy::LocalYieldingV1).unwrap();
    // Loaded/returning worker learns its outlet/home path by actual paid travel.
    for p in [pos(1, 3), pos(0, 3), pos(1, 3), pos(1, 2)] {
        fixture_tick(&mut w, 2, Action::Move(p));
    }
    prepare_queue(&mut w);
    let i = if matches!(mode, Mode::Departing | Mode::Searching) {
        0
    } else {
        2
    };
    if mode == Mode::Departing {
        controlled_tick(&mut w, i, Action::Wait(WaitReason::NoNeighbor), |a| {
            a.phase = FoodPhase::Departing;
            a.site = Some(pos(2, 1));
        });
    }
    if matches!(mode, Mode::FoodReturning | Mode::SpoilHauling) {
        controlled_tick(&mut w, i, Action::Wait(WaitReason::NoNeighbor), |a| {
            a.phase = FoodPhase::Searching
        });
        fixture_tick(
            &mut w,
            i,
            if mode == Mode::FoodReturning {
                Action::PickupFood
            } else {
                Action::Dig(pos(3, 1))
            },
        );
    }
    let phase_draw = mode == Mode::Searching;
    for _ in 0..2 {
        assert_eq!(
            selected_tick(&mut w, i, if phase_draw { &[0.5] } else { &[] }),
            usize::from(phase_draw)
        );
    }
    let choice = if i == 0 { 0.5 } else { 0.0 };
    let values = if phase_draw {
        vec![0.5, 0.0, choice]
    } else {
        vec![0.0, choice]
    };
    assert_eq!(selected_tick(&mut w, i, &values), values.len());
    assert!(w.state.agents[i].traffic.hold_pending);
    assert_eq!(w.state.agents[i].mode(), mode);
    let o = view(&w, i);
    (w, i, o)
}
