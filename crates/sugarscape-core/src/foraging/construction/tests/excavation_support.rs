//! Paid knowledge preparation with explicitly controlled food intent, not seeded advice.
use super::super::decision::{Action, WaitReason};
use super::traffic_support::{controlled_tick, fixture_tick};
use super::*;
pub(super) fn informed_setup() -> Setup {
    let mut open = (1..=5).map(|y| pos(2, y)).collect::<Vec<_>>();
    open.push(pos(3, 5));
    open.extend((1..=5).map(|y| pos(4, y)));
    open.push(pos(1, 1));
    Setup {
        width: 7,
        height: 7,
        open,
        diggable: vec![pos(3, 1)],
        nest: vec![pos(2, 1), pos(2, 2)],
        waste: pos(1, 1),
        workers: vec![pos(2, 1)],
        food: vec![Resource {
            id: 42,
            pos: pos(4, 1),
        }],
        parameters: Parameters {
            p_search: 0.0,
            p_return: 0.0,
            lambda_fidelity: 0.0,
            lambda_publish: 0.0,
            lambda_waypoint: 0.0,
        },
    }
}
pub(super) fn prepare(w: &mut World, frontier: bool) {
    for y in 2..=5 {
        fixture_tick(w, 0, Action::Move(pos(2, y)));
        if frontier && y == 3 {
            fixture_tick(w, 0, Action::Move(pos(1, 3)));
            fixture_tick(w, 0, Action::Move(pos(2, 3)));
        }
    }
    fixture_tick(w, 0, Action::Move(pos(3, 5)));
    fixture_tick(w, 0, Action::Move(pos(4, 5)));
    for y in (1..=4).rev() {
        fixture_tick(w, 0, Action::Move(pos(4, y)));
    }
    for y in 2..=5 {
        fixture_tick(w, 0, Action::Move(pos(4, y)));
    }
    fixture_tick(w, 0, Action::Move(pos(3, 5)));
    fixture_tick(w, 0, Action::Move(pos(2, 5)));
    for y in (1..=4).rev() {
        fixture_tick(w, 0, Action::Move(pos(2, y)));
    }
    fixture_tick(w, 0, Action::Move(pos(1, 1)));
    fixture_tick(w, 0, Action::Move(pos(2, 1)));
    // One paid NoNeighbor wait controls intent, without manufacturing advice history.
    controlled_tick(w, 0, Action::Wait(WaitReason::NoNeighbor), |a| {
        a.site = Some(pos(4, 1));
        a.phase = FoodPhase::Departing;
        a.frontier = None;
        a.face = None;
    });
    super::super::controller::check(&w.setup, &w.state).unwrap();
    assert_eq!(w.state.food.inventory().available, 1);
}
pub(super) fn informed_world(traffic: TrafficPolicy, excavation: ExcavationPolicy) -> World {
    let mut w = World::new_with_policies(informed_setup(), 7, traffic, excavation).unwrap();
    prepare(&mut w, false);
    w
}
pub(super) fn frontier_world() -> World {
    let mut setup = informed_setup();
    setup.open.extend([pos(1, 3), pos(0, 3)]);
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    prepare(&mut w, true);
    w
}
pub(super) fn short_world(depth: u32) -> World {
    let mut setup = informed_setup();
    setup.open.retain(|p| p.y <= depth && p.x != 3);
    setup.open.push(pos(3, depth));
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    for y in 2..=depth {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, depth)));
    fixture_tick(&mut w, 0, Action::Move(pos(4, depth)));
    for y in (1..depth).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    for y in 2..=depth {
        fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, depth)));
    fixture_tick(&mut w, 0, Action::Move(pos(2, depth)));
    for y in (1..depth).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(1, 1)));
    fixture_tick(&mut w, 0, Action::Move(pos(2, 1)));
    controlled_tick(&mut w, 0, Action::Wait(WaitReason::NoNeighbor), |a| {
        a.site = Some(pos(4, 1))
    });
    w
}
pub(super) fn tied_world(traffic: TrafficPolicy, blocked: bool) -> World {
    let mut setup = informed_setup();
    setup.height = 11;
    setup.open = vec![pos(1, 5), pos(3, 0), pos(3, 10)];
    setup
        .open
        .extend((0..=10).flat_map(|y| [pos(2, y), pos(4, y)]));
    setup.diggable = vec![pos(3, 4), pos(3, 6)];
    setup.nest = if blocked {
        (0..=6).map(|y| pos(2, y)).collect()
    } else {
        vec![pos(2, 5), pos(2, 6)]
    };
    setup.waste = pos(1, 5);
    setup.workers = if blocked {
        vec![pos(2, 5), pos(2, 0), pos(2, 0)]
    } else {
        vec![pos(2, 5)]
    };
    setup.food = vec![Resource {
        id: 42,
        pos: pos(4, 5),
    }];
    let mut w =
        World::new_with_policies(setup, 7, traffic, ExcavationPolicy::InformedCostV1).unwrap();
    // Learn the ring by paid travel. The occupied top-left endpoint is observed,
    // not entered; its top bridge is independently observed from the right.
    for y in 6..=10 {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, 10)));
    for y in (0..=10).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, 0)));
    fixture_tick(&mut w, 0, Action::Move(pos(4, 0)));
    for y in 1..=10 {
        fixture_tick(&mut w, 0, Action::Move(pos(4, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(3, 10)));
    for y in (1..=10).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    for y in 2..=5 {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(1, 5)));
    fixture_tick(&mut w, 0, Action::Move(pos(2, 5)));
    if blocked {
        for i in 1..=2 {
            for y in 1..=4 {
                fixture_tick(&mut w, i, Action::Move(pos(2, y)));
            }
        }
    }
    controlled_tick(&mut w, 0, Action::Wait(WaitReason::NoNeighbor), |a| {
        a.site = Some(pos(4, 5));
        a.frontier = None;
        a.face = None;
    });
    w
}
pub(super) fn alternate_resumption_world() -> World {
    let mut setup = informed_setup();
    setup.open = (1..=5).map(|y| pos(2, y)).collect();
    setup.open.extend((1..=5).map(|y| pos(5, y)));
    setup
        .open
        .extend([pos(2, 0), pos(3, 2), pos(3, 5), pos(4, 5), pos(4, 1)]);
    setup.nest = vec![pos(2, 0), pos(2, 1)];
    setup.waste = pos(2, 2);
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    for y in 2..=5 {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
        if y == 2 {
            fixture_tick(&mut w, 0, Action::Move(pos(3, 2)));
            fixture_tick(&mut w, 0, Action::Move(pos(2, 2)));
        }
    }
    for p in [pos(3, 5), pos(4, 5)] {
        fixture_tick(&mut w, 0, Action::Move(p));
    }
    for y in (1..=5).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(5, y)));
    }
    fixture_tick(&mut w, 0, Action::Move(pos(4, 1)));
    for y in 1..=5 {
        fixture_tick(&mut w, 0, Action::Move(pos(5, y)));
    }
    for p in [pos(4, 5), pos(3, 5)] {
        fixture_tick(&mut w, 0, Action::Move(p));
    }
    for y in (1..=5).rev() {
        fixture_tick(&mut w, 0, Action::Move(pos(2, y)));
    }
    controlled_tick(&mut w, 0, Action::Wait(WaitReason::NoNeighbor), |a| {
        a.site = Some(pos(4, 1))
    });
    w
}
