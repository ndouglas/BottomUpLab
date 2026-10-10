use super::super::traffic::{TrafficEffect, TrafficState};
use super::*;

#[test]
fn policy_identity_is_explicit_and_unknown_is_rejected() {
    for (policy, json) in [
        (TrafficPolicy::ShortestOnlyV1, "\"shortest_only_v1\""),
        (TrafficPolicy::LocalYieldingV1, "\"local_yielding_v1\""),
    ] {
        assert_eq!(serde_json::to_string(&policy).unwrap(), json);
        assert_eq!(serde_json::from_str::<TrafficPolicy>(json).unwrap(), policy);
    }
    assert!(serde_json::from_str::<TrafficPolicy>("\"future_policy\"").is_err());
}
#[test]
fn checked_counts_add_every_field() {
    let mut counts = TrafficCounts {
        blocked_route_opportunities: 2,
        yield_attempts: 3,
        yield_moves: 4,
        hold_waits: 5,
    };
    counts
        .checked_include(&TrafficCounts {
            blocked_route_opportunities: 7,
            yield_attempts: 11,
            yield_moves: 13,
            hold_waits: 17,
        })
        .unwrap();
    assert_eq!(
        counts,
        TrafficCounts {
            blocked_route_opportunities: 9,
            yield_attempts: 14,
            yield_moves: 17,
            hold_waits: 22
        }
    );
}
#[test]
fn overflow_in_each_field_leaves_all_counts_unchanged() {
    for field in 0..4 {
        let mut c = TrafficCounts::default();
        match field {
            0 => c.blocked_route_opportunities = u64::MAX,
            1 => c.yield_attempts = u64::MAX,
            2 => c.yield_moves = u64::MAX,
            _ => c.hold_waits = u64::MAX,
        }
        let before = c.clone();
        assert!(c
            .checked_include(&TrafficCounts {
                blocked_route_opportunities: 1,
                yield_attempts: 1,
                yield_moves: 1,
                hold_waits: 1
            })
            .is_err());
        assert_eq!(c, before);
    }
}
#[test]
fn paid_effects_saturate_blocking_and_reset_traffic_memory() {
    let mut s = TrafficState::default();
    for expected in [1, 2, 2] {
        s.commit(TrafficEffect::BlockedWait).unwrap();
        assert_eq!(s.blocked_wait_streak, expected);
    }
    assert_eq!(s.counts, TrafficCounts::default());
    s.commit(TrafficEffect::YieldMove).unwrap();
    assert_eq!(
        (s.blocked_wait_streak, s.hold_pending, s.counts.yield_moves),
        (0, true, 1)
    );
    s.commit(TrafficEffect::Hold).unwrap();
    assert_eq!(
        (s.blocked_wait_streak, s.hold_pending, s.counts.hold_waits),
        (0, false, 1)
    );
    s.commit(TrafficEffect::BlockedWait).unwrap();
    s.commit(TrafficEffect::Normal).unwrap();
    assert_eq!((s.blocked_wait_streak, s.hold_pending), (0, false));
    s.commit(TrafficEffect::YieldMove).unwrap();
    s.commit(TrafficEffect::Normal).unwrap();
    assert_eq!((s.hold_pending, s.counts.hold_waits), (false, 1));
}
#[test]
fn failed_paid_effect_leaves_memory_and_counts_unchanged() {
    for effect in [TrafficEffect::YieldMove, TrafficEffect::Hold] {
        let mut s = TrafficState {
            blocked_wait_streak: 2,
            counts: TrafficCounts {
                yield_moves: u64::MAX,
                hold_waits: u64::MAX,
                ..Default::default()
            },
            ..Default::default()
        };
        let before = s.clone();
        assert!(s.commit(effect).is_err());
        assert_eq!(s, before);
    }
}
#[test]
fn streak_and_pending_hold_invariants_are_checked() {
    for s in [
        TrafficState {
            blocked_wait_streak: 3,
            ..Default::default()
        },
        TrafficState {
            blocked_wait_streak: 1,
            hold_pending: true,
            ..Default::default()
        },
    ] {
        assert!(s
            .check(TrafficPolicy::LocalYieldingV1, &WorkCounts::default())
            .is_err());
    }
}
#[test]
fn legacy_allows_blocking_but_rejects_yielding_and_holds() {
    let work = WorkCounts {
        opportunities: 4,
        moves: 2,
        empty_congestion_waits: 2,
        ..Default::default()
    };
    let valid = TrafficState {
        blocked_wait_streak: 2,
        counts: TrafficCounts {
            blocked_route_opportunities: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    valid.check(TrafficPolicy::ShortestOnlyV1, &work).unwrap();
    for s in [
        TrafficState {
            blocked_wait_streak: 0,
            hold_pending: true,
            ..valid.clone()
        },
        TrafficState {
            counts: TrafficCounts {
                blocked_route_opportunities: 2,
                yield_attempts: 1,
                ..Default::default()
            },
            ..valid.clone()
        },
        TrafficState {
            counts: TrafficCounts {
                blocked_route_opportunities: 2,
                yield_attempts: 1,
                yield_moves: 1,
                hold_waits: 1,
            },
            ..valid
        },
    ] {
        assert!(s.check(TrafficPolicy::ShortestOnlyV1, &work).is_err());
    }
}
#[test]
fn traffic_bounds_are_checked_against_counts_and_paid_work() {
    let work = WorkCounts {
        opportunities: 2,
        moves: 1,
        food_congestion_waits: 1,
        ..Default::default()
    };
    for counts in [
        TrafficCounts {
            blocked_route_opportunities: 3,
            ..Default::default()
        },
        TrafficCounts {
            yield_attempts: 1,
            ..Default::default()
        },
        TrafficCounts {
            blocked_route_opportunities: 1,
            yield_moves: 1,
            ..Default::default()
        },
        TrafficCounts {
            hold_waits: 1,
            ..Default::default()
        },
        TrafficCounts {
            blocked_route_opportunities: 2,
            yield_attempts: 2,
            yield_moves: 2,
            ..Default::default()
        },
    ] {
        assert!(TrafficState {
            counts,
            ..Default::default()
        }
        .check(TrafficPolicy::LocalYieldingV1, &work)
        .is_err());
    }
    let valid = TrafficState {
        hold_pending: true,
        counts: TrafficCounts {
            blocked_route_opportunities: 2,
            yield_attempts: 1,
            yield_moves: 1,
            hold_waits: 0,
        },
        ..Default::default()
    };
    valid.check(TrafficPolicy::LocalYieldingV1, &work).unwrap();
    let holds = TrafficState {
        hold_pending: false,
        counts: TrafficCounts {
            hold_waits: 1,
            ..valid.counts
        },
        ..Default::default()
    };
    holds.check(TrafficPolicy::LocalYieldingV1, &work).unwrap();
    assert!(holds
        .check(
            TrafficPolicy::LocalYieldingV1,
            &WorkCounts {
                food_congestion_waits: 0,
                ..work
            }
        )
        .is_err());
}
#[test]
fn congestion_sum_overflow_is_an_error() {
    let work = WorkCounts {
        empty_congestion_waits: u64::MAX,
        food_congestion_waits: 1,
        ..Default::default()
    };
    assert!(TrafficState::default()
        .check(TrafficPolicy::LocalYieldingV1, &work)
        .is_err());
}
