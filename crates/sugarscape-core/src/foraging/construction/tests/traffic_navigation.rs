use super::super::{
    navigation::Navigation,
    traffic::{TrafficEffect, TrafficState},
    traffic_navigation::select_route,
};
use super::*;

fn corridor() -> (Knowledge, Observation) {
    let mut scene = setup();
    scene.open = (0..5).map(|x| pos(x, 0)).collect();
    scene.nest = vec![pos(0, 0), pos(1, 0)];
    scene.waste = pos(4, 0);
    scene.workers = vec![pos(0, 0)];
    scene.food.clear();
    scene.diggable.clear();
    let terrain = Terrain::new(&scene).unwrap();
    let mut known = Knowledge::new(scene.width, scene.height).unwrap();
    for cell in &scene.open {
        known
            .learn(&observe(&terrain, *cell, &BTreeMap::new(), &BTreeSet::new()).unwrap())
            .unwrap();
    }
    let observation = observe(
        &terrain,
        pos(2, 0),
        &BTreeMap::from([(pos(1, 0), 2)]),
        &BTreeSet::new(),
    )
    .unwrap();
    (known, observation)
}
fn traffic(streak: u8) -> TrafficState {
    TrafficState {
        blocked_wait_streak: streak,
        counts: TrafficCounts {
            blocked_route_opportunities: u64::from(streak),
            ..Default::default()
        },
        ..Default::default()
    }
}
#[test]
fn third_blocked_opportunity_yields_below_quarter_and_chooses_singleton() {
    let (known, observation) = corridor();
    let mut traffic = traffic(2);
    let mut draws = Scripted::new(&[0.249, 0.0]);
    let (step, counts, effect) = select_route(
        TrafficPolicy::LocalYieldingV1,
        &mut traffic,
        &known,
        pos(2, 0),
        &[pos(1, 0), pos(4, 0)],
        &observation,
        &mut draws,
    )
    .unwrap();
    assert_eq!(step, Navigation::Move(pos(3, 0)));
    assert_eq!(effect, TrafficEffect::YieldMove);
    assert_eq!(draws.next, 2);
    assert_eq!(counts.route_calls, 1);
    assert_eq!(traffic.counts.blocked_route_opportunities, 3);
    assert_eq!(traffic.counts.yield_attempts, 1);
    assert_eq!(traffic.counts.yield_moves, 0); // Selection does not pay the action.
    assert_eq!(traffic.blocked_wait_streak, 2);
}
#[test]
fn threshold_equality_and_failed_ticket_take_one_draw() {
    for ticket in [0.25, 0.9] {
        let (known, observation) = corridor();
        let mut traffic = traffic(2);
        let mut draws = Scripted::new(&[ticket]);
        let (step, _, effect) = select_route(
            TrafficPolicy::LocalYieldingV1,
            &mut traffic,
            &known,
            pos(2, 0),
            &[pos(1, 0)],
            &observation,
            &mut draws,
        )
        .unwrap();
        assert_eq!(
            (step, effect, draws.next),
            (Navigation::Blocked, TrafficEffect::BlockedWait, 1)
        );
    }
}
#[test]
fn prethreshold_and_legacy_blocked_routes_do_not_draw() {
    for (policy, streak) in [
        (TrafficPolicy::LocalYieldingV1, 0),
        (TrafficPolicy::LocalYieldingV1, 1),
        (TrafficPolicy::ShortestOnlyV1, 2),
    ] {
        let (known, observation) = corridor();
        let mut traffic = traffic(streak);
        let mut draws = Scripted::new(&[]);
        let (step, _, effect) = select_route(
            policy,
            &mut traffic,
            &known,
            pos(2, 0),
            &[pos(1, 0)],
            &observation,
            &mut draws,
        )
        .unwrap();
        assert_eq!(
            (step, effect, draws.next),
            (Navigation::Blocked, TrafficEffect::BlockedWait, 0)
        );
        assert_eq!(traffic.counts.yield_attempts, 0);
    }
}
#[test]
fn no_free_candidate_and_unreachable_and_goal_take_no_ticket() {
    let (known, mut observation) = corridor();
    for cell in &mut observation.cells {
        if cell.open && cell.pos != pos(2, 0) {
            cell.occupants = 2;
        }
    }
    for (goals, expected, effect, blocked) in [
        (
            vec![pos(1, 0)],
            Navigation::Blocked,
            TrafficEffect::BlockedWait,
            1,
        ),
        (
            vec![pos(2, 0)],
            Navigation::AtGoal,
            TrafficEffect::Normal,
            0,
        ),
        (
            vec![pos(4, 2)],
            Navigation::Unreachable,
            TrafficEffect::Normal,
            0,
        ),
        (vec![], Navigation::Unreachable, TrafficEffect::Normal, 0),
    ] {
        let mut traffic = traffic(2);
        let before = traffic.counts.blocked_route_opportunities;
        let mut draws = Scripted::new(&[]);
        let (step, _, actual) = select_route(
            TrafficPolicy::LocalYieldingV1,
            &mut traffic,
            &known,
            pos(2, 0),
            &goals,
            &observation,
            &mut draws,
        )
        .unwrap();
        assert_eq!((step, actual, draws.next), (expected, effect, 0));
        assert_eq!(traffic.counts.blocked_route_opportunities, before + blocked);
        assert_eq!(traffic.counts.yield_attempts, 0);
    }
}
#[test]
fn free_shortest_step_uses_only_ordinary_choice() {
    let (known, mut observation) = corridor();
    for cell in &mut observation.cells {
        cell.occupants = 0;
    }
    let mut traffic = traffic(2);
    let before = traffic.counts.clone();
    let mut draws = Scripted::new(&[0.9]);
    let (step, _, effect) = select_route(
        TrafficPolicy::LocalYieldingV1,
        &mut traffic,
        &known,
        pos(2, 0),
        &[pos(1, 0)],
        &observation,
        &mut draws,
    )
    .unwrap();
    assert_eq!(
        (step, effect, draws.next),
        (Navigation::Move(pos(1, 0)), TrafficEffect::Normal, 1)
    );
    assert_eq!(traffic.counts, before);
}
#[test]
fn invalid_and_exhausted_tickets_error() {
    for values in [vec![], vec![f64::NAN], vec![1.0], vec![-0.1], vec![0.0]] {
        let (known, observation) = corridor();
        let mut traffic = traffic(2);
        let mut draws = Scripted::new(&values);
        assert!(select_route(
            TrafficPolicy::LocalYieldingV1,
            &mut traffic,
            &known,
            pos(2, 0),
            &[pos(1, 0)],
            &observation,
            &mut draws
        )
        .is_err());
    }
}
#[test]
fn tied_escape_choice_preserves_south_east_west_order() {
    let mut scene = setup();
    scene.height = 5;
    scene.open = vec![pos(2, 2), pos(2, 1), pos(2, 3), pos(3, 2), pos(1, 2)];
    scene.nest = vec![pos(2, 1), pos(2, 2)];
    scene.waste = pos(2, 3);
    scene.workers = vec![pos(2, 2)];
    scene.food.clear();
    scene.diggable.clear();
    let terrain = Terrain::new(&scene).unwrap();
    let mut known = Knowledge::new(5, 5).unwrap();
    for p in &scene.open {
        known
            .learn(&observe(&terrain, *p, &BTreeMap::new(), &BTreeSet::new()).unwrap())
            .unwrap();
    }
    let observation = observe(
        &terrain,
        pos(2, 2),
        &BTreeMap::from([(pos(2, 1), 2)]),
        &BTreeSet::new(),
    )
    .unwrap();
    for (choice, target) in [(0.0, pos(2, 3)), (0.5, pos(3, 2)), (0.99, pos(1, 2))] {
        let mut traffic = traffic(2);
        let mut draws = Scripted::new(&[0.0, choice]);
        let (step, _, effect) = select_route(
            TrafficPolicy::LocalYieldingV1,
            &mut traffic,
            &known,
            pos(2, 2),
            &[pos(2, 1)],
            &observation,
            &mut draws,
        )
        .unwrap();
        assert_eq!(
            (step, effect, draws.next),
            (Navigation::Move(target), TrafficEffect::YieldMove, 2)
        );
    }
}
