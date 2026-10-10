use super::super::traffic_runner::{
    checked_snapshot_total, run_with_traffic_limit, TrafficRecording,
};
use super::traffic_support::*;
use super::*;
use rand::RngCore;

fn options(sample_every: u32, snapshots: bool) -> RunOptions {
    RunOptions {
        ticks: 40,
        sample_every,
        snapshots,
    }
}
fn continuation(w: &World) -> [u64; 4] {
    let mut rng = w.rng.clone();
    std::array::from_fn(|_| rng.next_u64())
}
fn empty_recording() -> TrafficRecording {
    TrafficRecording {
        base_snapshots: vec![],
        traffic_snapshots: vec![],
        base_bytes: 0,
        traffic_bytes: 0,
    }
}
#[test]
fn labeled_legacy_runner_preserves_the_complete_base_episode() {
    let base = run(setup(), 7, options(7, true)).unwrap();
    let labeled =
        run_with_traffic(setup(), 7, options(7, true), TrafficPolicy::ShortestOnlyV1).unwrap();
    assert_eq!(labeled.schema, "foraging-traffic-episode-v1");
    assert_eq!(labeled.episode, base);
    assert_eq!(labeled.policy, TrafficPolicy::ShortestOnlyV1);
    assert_eq!(labeled.traffic_summary.policy, labeled.policy);
    assert_eq!(
        labeled.traffic_snapshots.len(),
        labeled.episode.snapshots.len()
    );
    let json = serde_json::to_value(&labeled).unwrap();
    assert_eq!(json["policy"], "shortest_only_v1");
    assert_eq!(json.as_object().unwrap().len(), 6);
}
#[test]
fn checkpoints_align_once_for_irregular_divisible_and_long_intervals() {
    for (cadence, expected) in [
        (7, vec![0, 7, 14, 21, 28, 35, 40]),
        (10, vec![0, 10, 20, 30, 40]),
        (100, vec![0, 40]),
    ] {
        let e = run_with_traffic(
            setup(),
            7,
            options(cadence, true),
            TrafficPolicy::LocalYieldingV1,
        )
        .unwrap();
        assert_eq!(
            e.episode
                .snapshots
                .iter()
                .map(|s| s.summary.completed_ticks)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            e.traffic_snapshots
                .iter()
                .map(|s| s.completed_ticks)
                .collect::<Vec<_>>(),
            expected
        );
        for (base, traffic) in e.episode.snapshots.iter().zip(&e.traffic_snapshots) {
            assert_eq!(traffic.policy, e.policy);
            assert_eq!(
                base.agents.iter().map(|a| a.id).collect::<Vec<_>>(),
                traffic.agents.iter().map(|a| a.id).collect::<Vec<_>>()
            );
        }
        let final_frame = e.traffic_snapshots.last().unwrap();
        assert_eq!(final_frame.agents, e.traffic_summary.agents);
        assert_eq!(e.traffic_summary.completed_ticks, 40);
    }
}
#[test]
fn disabled_recording_has_no_frames_or_bytes_even_with_zero_budget() {
    let e = run_with_traffic_limit(
        setup(),
        7,
        options(7, false),
        TrafficPolicy::LocalYieldingV1,
        0,
    )
    .unwrap();
    assert!(e.episode.snapshots.is_empty());
    assert!(e.traffic_snapshots.is_empty());
    assert_eq!((e.episode.snapshot_bytes, e.traffic_snapshot_bytes), (0, 0));
    assert_eq!(e.traffic_summary.completed_ticks, 40);
}
#[test]
fn traffic_views_do_not_mutate_world_or_rng_across_sampling_options() {
    for policy in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let mut observed = World::new_with_traffic(setup(), 7, policy).unwrap();
        let mut control = observed.clone();
        for tick in 0..=40 {
            let before = observed.state.clone();
            let rng = continuation(&observed);
            observed.traffic_summary().unwrap();
            observed.traffic_snapshot().unwrap();
            observed.summary().unwrap();
            observed.snapshot().unwrap();
            assert_eq!(observed.state, before);
            assert_eq!(continuation(&observed), rng);
            assert_eq!(observed.state, control.state);
            assert_eq!(continuation(&observed), continuation(&control));
            if tick < 40 {
                observed.step().unwrap();
                control.step().unwrap();
            }
        }
        let frequent = run_with_traffic(setup(), 7, options(1, true), policy).unwrap();
        for (cadence, enabled) in [(7, true), (40, true), (100, true), (1, false)] {
            let e = run_with_traffic(setup(), 7, options(cadence, enabled), policy).unwrap();
            assert_eq!(e.episode.summary, frequent.episode.summary);
            assert_eq!(e.traffic_summary, frequent.traffic_summary);
        }
    }
}
#[test]
fn traffic_views_project_paid_counters_in_worker_id_order() {
    let mut w = opposing_queue(TrafficPolicy::LocalYieldingV1);
    for values in [&[0.5, 0.5][..], &[0.5, 0.5][..], QUEUE_THIRD_TICK_SCRIPT] {
        advance_scripted(&mut w, &mut Scripted::new(values)).unwrap();
    }
    let summary = w.traffic_summary().unwrap();
    assert_eq!(
        summary.counts,
        TrafficCounts {
            blocked_route_opportunities: 11,
            yield_attempts: 3,
            yield_moves: 1,
            hold_waits: 0
        }
    );
    assert_eq!(
        summary.agents.iter().map(|a| a.id).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        summary.agents[0].counts,
        TrafficCounts {
            blocked_route_opportunities: 3,
            yield_attempts: 1,
            yield_moves: 1,
            hold_waits: 0
        }
    );
    assert!(summary.agents[0].hold_pending);
    assert_eq!(w.traffic_snapshot().unwrap().agents, summary.agents);
    assert_eq!(
        serde_json::to_value(w.traffic_snapshot().unwrap()).unwrap()["agents"][0]
            .as_object()
            .unwrap()
            .len(),
        4
    );
}
#[test]
fn traffic_views_validate_state_before_projection() {
    let mut w = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    w.state.agents[0].traffic.blocked_wait_streak = 3;
    for errors in [
        w.traffic_summary().unwrap_err(),
        w.traffic_snapshot().unwrap_err(),
    ] {
        assert!(errors
            .iter()
            .any(|e| e.field.contains("agents[0].traffic.blocked_wait_streak")));
    }
}
#[test]
fn seeded_traffic_episodes_replay_and_continue_after_depletion() {
    for seed in [7, 8] {
        let e = run_with_traffic(
            setup(),
            seed,
            options(7, true),
            TrafficPolicy::LocalYieldingV1,
        )
        .unwrap();
        assert_eq!(
            run_with_traffic(
                e.episode.setup.clone(),
                seed,
                e.episode.options.clone(),
                e.policy
            )
            .unwrap(),
            e
        );
    }
    let mut s = setup();
    s.food.clear();
    s.diggable.clear();
    let e = run_with_traffic(s, 7, options(100, true), TrafficPolicy::LocalYieldingV1).unwrap();
    assert_eq!(e.episode.summary.completed_ticks, 40);
    assert_eq!(e.episode.summary.work.opportunities, 40);
    assert_eq!(e.episode.summary.food.initial, 0);
}
#[test]
fn explicit_runner_aggregates_invalid_setup_and_options() {
    let mut s = setup();
    s.width = 0;
    let errors = run_with_traffic(
        s,
        7,
        RunOptions {
            ticks: 0,
            sample_every: 0,
            snapshots: false,
        },
        TrafficPolicy::LocalYieldingV1,
    )
    .unwrap_err();
    for field in ["width", "ticks", "sample_every"] {
        assert!(errors.iter().any(|e| e.field == field));
    }
    assert!(run_with_traffic(
        setup(),
        7,
        RunOptions {
            ticks: 40,
            sample_every: 0,
            snapshots: false
        },
        TrafficPolicy::LocalYieldingV1
    )
    .is_err());
}
#[test]
fn compact_bytes_are_separate_and_combined_limit_is_exact() {
    let e = run_with_traffic(setup(), 7, options(7, true), TrafficPolicy::LocalYieldingV1).unwrap();
    let base = e
        .episode
        .snapshots
        .iter()
        .map(|s| serde_json::to_vec(s).unwrap().len() as u64)
        .sum::<u64>();
    let traffic = e
        .traffic_snapshots
        .iter()
        .map(|s| serde_json::to_vec(s).unwrap().len() as u64)
        .sum::<u64>();
    assert_eq!(e.episode.snapshot_bytes, base);
    assert_eq!(e.traffic_snapshot_bytes, traffic);
    let total = base.checked_add(traffic).unwrap();
    assert!(serde_json::to_vec(&e).unwrap().len() as u64 > total);
    assert_eq!(
        run_with_traffic_limit(setup(), 7, options(7, true), e.policy, total).unwrap(),
        e
    );
    assert!(run_with_traffic_limit(setup(), 7, options(7, true), e.policy, total - 1).is_err());
    let component_cap = base.max(traffic) + 1;
    assert!(component_cap < total);
    println!("compact snapshot bytes: base={base}, traffic={traffic}, total={total}, independent-component cap={component_cap}");
    assert!(run_with_traffic_limit(setup(), 7, options(7, true), e.policy, component_cap).is_err());
}
#[test]
fn checked_combined_sum_enforces_exact_limit() {
    assert_eq!(checked_snapshot_total(8, 5, 13).unwrap(), 13);
    assert_eq!(
        checked_snapshot_total(8, 5, 12).unwrap_err()[0].field,
        "traffic_snapshot_bytes"
    );
}
#[test]
fn checked_combined_sum_rejects_overflow_without_large_allocation() {
    assert!(checked_snapshot_total(u64::MAX, 1, u64::MAX).is_err());
}
#[test]
fn failed_pair_append_preserves_all_previous_frames_and_counts() {
    let w = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    let base = w.snapshot().unwrap();
    let traffic = w.traffic_snapshot().unwrap();
    let total = (serde_json::to_vec(&base).unwrap().len()
        + serde_json::to_vec(&traffic).unwrap().len()) as u64;
    let mut recording = empty_recording();
    recording
        .append(base.clone(), traffic.clone(), total)
        .unwrap();
    let before_base = recording.base_snapshots.clone();
    let before_traffic = recording.traffic_snapshots.clone();
    let before_counts = (recording.base_bytes, recording.traffic_bytes);
    let errors = recording.append(base, traffic, total * 2 - 1).unwrap_err();
    assert_eq!(errors[0].field, "traffic_snapshot_bytes");
    assert_eq!(recording.base_snapshots, before_base);
    assert_eq!(recording.traffic_snapshots, before_traffic);
    assert_eq!(
        (recording.base_bytes, recording.traffic_bytes),
        before_counts
    );
}
fn rejects_pair_fault_atomically(fault: usize, existing: bool) {
    let w = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    let mut recording = empty_recording();
    if existing {
        recording
            .append(
                w.snapshot().unwrap(),
                w.traffic_snapshot().unwrap(),
                u64::MAX,
            )
            .unwrap();
    }
    let before_base = recording.base_snapshots.clone();
    let before_traffic = recording.traffic_snapshots.clone();
    let before_counts = (recording.base_bytes, recording.traffic_bytes);
    let mut traffic = w.traffic_snapshot().unwrap();
    match fault {
        0 => traffic.completed_ticks += 1,
        1 => traffic.policy = TrafficPolicy::ShortestOnlyV1,
        2 => traffic.agents[0].id = 5,
        _ => traffic.agents.clear(),
    }
    assert!(recording
        .append(w.snapshot().unwrap(), traffic, u64::MAX)
        .is_err());
    assert_eq!(recording.base_snapshots, before_base);
    assert_eq!(recording.traffic_snapshots, before_traffic);
    assert_eq!(
        (recording.base_bytes, recording.traffic_bytes),
        before_counts
    );
}
#[test]
fn paired_recording_rejects_mismatched_checkpoints_atomically() {
    rejects_pair_fault_atomically(0, false);
}
#[test]
fn paired_recording_rejects_changed_policy_atomically() {
    rejects_pair_fault_atomically(1, true);
}
#[test]
fn paired_recording_rejects_mismatched_worker_ids_atomically() {
    rejects_pair_fault_atomically(2, false);
}
#[test]
fn paired_recording_rejects_mismatched_worker_counts_atomically() {
    rejects_pair_fault_atomically(3, false);
}
#[test]
fn sixty_four_mib_pair_limit_checks_cumulative_bytes_before_accepting() {
    let w = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    let base = w.snapshot().unwrap();
    let traffic = w.traffic_snapshot().unwrap();
    let base_size = serde_json::to_vec(&base).unwrap().len() as u64;
    let traffic_size = serde_json::to_vec(&traffic).unwrap().len() as u64;
    let limit = 64 * 1024 * 1024;
    // Simulate earlier counted compact bytes, avoiding a 64MiB allocation.
    let mut recording = empty_recording();
    recording.base_bytes = limit - base_size - traffic_size;
    recording
        .append(base.clone(), traffic.clone(), limit)
        .unwrap();
    assert_eq!(recording.base_bytes + recording.traffic_bytes, limit);
    let before_base = recording.base_snapshots.clone();
    let before_traffic = recording.traffic_snapshots.clone();
    let before_counts = (recording.base_bytes, recording.traffic_bytes);
    assert!(recording.append(base, traffic, limit).is_err());
    assert_eq!(recording.base_snapshots, before_base);
    assert_eq!(recording.traffic_snapshots, before_traffic);
    assert_eq!(
        (recording.base_bytes, recording.traffic_bytes),
        before_counts
    );
}
#[test]
fn component_byte_overflow_preserves_both_recordings() {
    let w = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    for traffic_overflow in [false, true] {
        let mut recording = empty_recording();
        if traffic_overflow {
            recording.traffic_bytes = u64::MAX;
        } else {
            recording.base_bytes = u64::MAX;
        }
        let before = (recording.base_bytes, recording.traffic_bytes);
        assert!(recording
            .append(
                w.snapshot().unwrap(),
                w.traffic_snapshot().unwrap(),
                u64::MAX
            )
            .is_err());
        assert_eq!((recording.base_bytes, recording.traffic_bytes), before);
        assert!(recording.base_snapshots.is_empty());
        assert!(recording.traffic_snapshots.is_empty());
    }
}
#[test]
fn synthetic_cutoff_records_a_legal_pending_hold_then_pays_it_on_continuation() {
    // Labeled synthetic paid fixture checkpoints, never a fabricated seeded Episode.
    let mut w = opposing_queue(TrafficPolicy::LocalYieldingV1);
    for values in [&[0.5, 0.5][..], &[0.5, 0.5][..], QUEUE_THIRD_TICK_SCRIPT] {
        advance_scripted(&mut w, &mut Scripted::new(values)).unwrap();
    }
    let before_recording = w.state.clone();
    let rng_before_recording = continuation(&w);
    let mut recording = empty_recording();
    recording
        .append(
            w.snapshot().unwrap(),
            w.traffic_snapshot().unwrap(),
            64 * 1024 * 1024,
        )
        .unwrap();
    assert_eq!(w.state, before_recording);
    assert_eq!(continuation(&w), rng_before_recording);
    let cutoff = &recording.traffic_snapshots[0].agents[0];
    assert!(cutoff.hold_pending);
    assert!(cutoff.counts.hold_waits < cutoff.counts.yield_moves);
    let before = w.state.agents[0].clone();
    advance_scripted(&mut w, &mut Scripted::new(QUEUE_FOURTH_TICK_SCRIPT)).unwrap();
    let after = &w.state.agents[0];
    assert_eq!(after.pos, before.pos);
    assert_eq!(after.work.opportunities, before.work.opportunities + 1);
    assert_eq!(
        after.work.empty_congestion_waits,
        before.work.empty_congestion_waits + 1
    );
    assert_eq!(
        after.traffic.counts.hold_waits,
        before.traffic.counts.hold_waits + 1
    );
    assert!(!after.traffic.hold_pending);
    assert!(recording.traffic_snapshots[0].agents[0].hold_pending);
}
