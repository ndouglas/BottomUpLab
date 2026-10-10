use super::super::policy_runner::{
    checked_policy_snapshot_total, run_with_policies_limit, PolicyRecording,
};
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
#[test]
fn explicit_legacy_excavation_preserves_both_traffic_episodes_and_labels() {
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let old = run_with_traffic(setup(), 7, options(7, true), traffic).unwrap();
        let new = run_with_policies(
            setup(),
            7,
            options(7, true),
            traffic,
            ExcavationPolicy::ExplorationFirstV1,
        )
        .unwrap();
        assert_eq!(new.schema, "foraging-construction-policy-episode-v1");
        assert_eq!(new.episode, old.episode);
        assert_eq!(new.traffic_summary, old.traffic_summary);
        assert_eq!(new.traffic_snapshots, old.traffic_snapshots);
        assert_eq!(new.traffic_snapshot_bytes, old.traffic_snapshot_bytes);
        assert_eq!(new.excavation_summary.counts, ExcavationCounts::default());
        assert_eq!(new.traffic_policy, traffic);
        assert_eq!(new.excavation_policy, ExcavationPolicy::ExplorationFirstV1);
        let json = serde_json::to_value(&new).unwrap();
        assert_eq!(json["schema"], "foraging-construction-policy-episode-v1");
        assert_eq!(json["excavation_policy"], "exploration_first_v1");
        assert_eq!(
            json["traffic_policy"],
            serde_json::to_value(traffic).unwrap()
        );
        assert_eq!(json.as_object().unwrap().len(), 10);
        assert!(new
            .excavation_summary
            .agents
            .iter()
            .all(|a| a.last_assessment.is_none()));
    }
}
#[test]
fn excavation_views_retain_a_paid_opened_face_without_searches_or_draws() {
    let mut w = excavation_support::informed_world(
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    );
    traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    assert_eq!(
        w.state.agents[0].map.kind(pos(3, 1)).unwrap(),
        CellKnowledge::KnownOpen
    );
    let record = w.state.agents[0].excavation.last_assessment.clone();
    let tick = w.state.tick;
    for script in [&[0.0][..], &[][..]] {
        let before = w.state.clone();
        let rng = continuation(&w);
        for _ in 0..3 {
            let summary = w.excavation_summary().unwrap();
            let snapshot = w.excavation_snapshot().unwrap();
            assert_eq!(summary.agents, snapshot.agents);
            assert_eq!(summary.counts, w.state.agents[0].excavation.counts);
            assert_eq!(summary.counts.initiated_digs, 1);
            assert_eq!(summary.counts.selected_opportunities, 1);
            assert_eq!(summary.agents[0].last_assessment, record);
            assert_eq!(
                summary.agents[0].last_assessment.as_ref().unwrap().tick,
                tick
            );
            assert_eq!(summary.completed_ticks, w.state.tick);
        }
        assert_eq!(w.state, before);
        assert_eq!(continuation(&w), rng);
        traffic_support::advance_scripted(&mut w, &mut Scripted::new(script)).unwrap();
    }
    assert_eq!(
        w.excavation_snapshot().unwrap().agents[0].last_assessment,
        record
    );
}

#[test]
fn both_policy_identities_are_projected_at_every_checkpoint() {
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        for excavation in [
            ExcavationPolicy::ExplorationFirstV1,
            ExcavationPolicy::InformedCostV1,
        ] {
            let e = run_with_policies(setup(), 7, options(7, true), traffic, excavation).unwrap();
            assert_eq!(e.excavation_summary.traffic_policy, traffic);
            assert_eq!(e.excavation_summary.excavation_policy, excavation);
            assert_eq!(e.traffic_summary.policy, traffic);
            for frame in &e.excavation_snapshots {
                assert_eq!(
                    (frame.traffic_policy, frame.excavation_policy),
                    (traffic, excavation)
                );
            }
            assert_eq!(
                serde_json::to_value(&e).unwrap()["excavation_policy"],
                serde_json::to_value(excavation).unwrap()
            );
        }
    }
}

#[test]
fn triple_checkpoints_are_chronological_with_initial_and_final_once() {
    for (interval, expected) in [
        (7, vec![0, 7, 14, 21, 28, 35, 40]),
        (10, vec![0, 10, 20, 30, 40]),
        (100, vec![0, 40]),
    ] {
        let e = run_with_policies(
            setup(),
            7,
            options(interval, true),
            TrafficPolicy::LocalYieldingV1,
            ExcavationPolicy::InformedCostV1,
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
        assert_eq!(
            e.excavation_snapshots
                .iter()
                .map(|s| s.completed_ticks)
                .collect::<Vec<_>>(),
            expected
        );
        for ((base, traffic), excavation) in e
            .episode
            .snapshots
            .iter()
            .zip(&e.traffic_snapshots)
            .zip(&e.excavation_snapshots)
        {
            let ids = base.agents.iter().map(|a| a.id).collect::<Vec<_>>();
            assert_eq!(traffic.agents.iter().map(|a| a.id).collect::<Vec<_>>(), ids);
            assert_eq!(
                excavation.agents.iter().map(|a| a.id).collect::<Vec<_>>(),
                ids
            );
        }
        assert_eq!(
            e.excavation_snapshots.last().unwrap().agents,
            e.excavation_summary.agents
        );
        assert!(e.excavation_snapshots[0]
            .agents
            .iter()
            .all(|a| a.counts == ExcavationCounts::default() && a.last_assessment.is_none()));
    }
}

#[test]
fn disabled_triple_recording_needs_no_budget_and_emits_no_frames_or_bytes() {
    let e = run_with_policies_limit(
        setup(),
        7,
        options(7, false),
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
        0,
    )
    .unwrap();
    assert!(e.episode.snapshots.is_empty());
    assert!(e.traffic_snapshots.is_empty());
    assert!(e.excavation_snapshots.is_empty());
    assert_eq!(
        (
            e.episode.snapshot_bytes,
            e.traffic_snapshot_bytes,
            e.excavation_snapshot_bytes
        ),
        (0, 0, 0)
    );
    assert_eq!(e.excavation_summary.completed_ticks, 40);
}

#[test]
fn recording_options_and_repeated_views_preserve_complete_state_rng_and_diagnostics() {
    for traffic in [
        TrafficPolicy::ShortestOnlyV1,
        TrafficPolicy::LocalYieldingV1,
    ] {
        let mut control =
            World::new_with_policies(setup(), 7, traffic, ExcavationPolicy::InformedCostV1)
                .unwrap();
        let mut observed = control.clone();
        for tick in 0..=40 {
            let before = observed.state.clone();
            let rng = continuation(&observed);
            for _ in 0..2 {
                observed.excavation_summary().unwrap();
                observed.excavation_snapshot().unwrap();
            }
            if tick % 7 == 0 || tick == 40 {
                observed.snapshot().unwrap();
                observed.traffic_snapshot().unwrap();
            }
            assert_eq!(observed.state, before);
            assert_eq!(continuation(&observed), rng);
            assert_eq!(observed.state, control.state);
            assert_eq!(continuation(&observed), continuation(&control));
            if tick < 40 {
                observed.step().unwrap();
                control.step().unwrap();
            }
        }
        let frequent = run_with_policies(
            setup(),
            7,
            options(1, true),
            traffic,
            ExcavationPolicy::InformedCostV1,
        )
        .unwrap();
        assert_eq!(frequent.episode.summary, control.summary().unwrap());
        assert_eq!(
            frequent.excavation_summary,
            control.excavation_summary().unwrap()
        );
        for (interval, enabled) in [(7, true), (40, true), (100, true), (1, false)] {
            let e = run_with_policies(
                setup(),
                7,
                options(interval, enabled),
                traffic,
                ExcavationPolicy::InformedCostV1,
            )
            .unwrap();
            assert_eq!(e.episode.summary, frequent.episode.summary);
            assert_eq!(e.traffic_summary, frequent.traffic_summary);
            assert_eq!(e.excavation_summary, frequent.excavation_summary);
        }
    }
}

#[test]
fn policy_runner_replays_engineering_seeds_and_continues_after_depletion() {
    for seed in [7, 8] {
        let e = run_with_policies(
            setup(),
            seed,
            options(7, true),
            TrafficPolicy::LocalYieldingV1,
            ExcavationPolicy::InformedCostV1,
        )
        .unwrap();
        assert_eq!(
            run_with_policies(
                e.episode.setup.clone(),
                e.episode.seed,
                e.episode.options.clone(),
                e.traffic_policy,
                e.excavation_policy
            )
            .unwrap(),
            e
        );
    }
    let mut s = setup();
    s.food.clear();
    s.diggable.clear();
    let e = run_with_policies(
        s,
        7,
        options(100, true),
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    assert_eq!(e.episode.summary.completed_ticks, 40);
    assert_eq!(e.episode.summary.work.opportunities, 40);
    assert_eq!(e.episode.summary.food.initial, 0);
}

#[test]
fn policy_runner_aggregates_invalid_setup_and_zero_interval_even_when_disabled() {
    let mut s = setup();
    s.width = 0;
    let errors = run_with_policies(
        s,
        7,
        RunOptions {
            ticks: 0,
            sample_every: 0,
            snapshots: false,
        },
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap_err();
    for field in ["width", "ticks", "sample_every"] {
        assert!(
            errors.iter().any(|e| e.field == field),
            "missing {field}: {errors:?}"
        );
    }
    for enabled in [false, true] {
        assert!(run_with_policies(
            setup(),
            7,
            options(0, enabled),
            TrafficPolicy::ShortestOnlyV1,
            ExcavationPolicy::InformedCostV1
        )
        .is_err());
    }
}

fn compact_size<T: serde::Serialize>(value: &T) -> u64 {
    serde_json::to_vec(value).unwrap().len() as u64
}

#[test]
fn all_three_compact_stream_sizes_share_the_exact_combined_cap() {
    let e = run_with_policies(
        setup(),
        7,
        options(7, true),
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    let base = e.episode.snapshots.iter().map(compact_size).sum::<u64>();
    let traffic = e.traffic_snapshots.iter().map(compact_size).sum::<u64>();
    let excavation = e.excavation_snapshots.iter().map(compact_size).sum::<u64>();
    assert_eq!(
        (
            e.episode.snapshot_bytes,
            e.traffic_snapshot_bytes,
            e.excavation_snapshot_bytes
        ),
        (base, traffic, excavation)
    );
    let total = base + traffic + excavation;
    assert!(compact_size(&e) > total);
    assert_eq!(
        run_with_policies_limit(
            setup(),
            7,
            options(7, true),
            e.traffic_policy,
            e.excavation_policy,
            total
        )
        .unwrap(),
        e
    );
    assert!(run_with_policies_limit(
        setup(),
        7,
        options(7, true),
        e.traffic_policy,
        e.excavation_policy,
        total - 1
    )
    .is_err());
    let component_limit = base.max(traffic).max(excavation) + 1;
    assert!(component_limit < total);
    assert!(run_with_policies_limit(
        setup(),
        7,
        options(7, true),
        e.traffic_policy,
        e.excavation_policy,
        component_limit
    )
    .is_err());
    println!("compact JSON: base={base}, traffic={traffic}, excavation={excavation}, total={total}, individual cap={component_limit}");
}

#[test]
fn checked_triple_total_enforces_exact_limit_and_both_overflow_positions() {
    assert_eq!(checked_policy_snapshot_total(8, 5, 3, 16).unwrap(), 16);
    assert_eq!(
        checked_policy_snapshot_total(8, 5, 3, 15).unwrap_err()[0].field,
        "excavation_snapshot_bytes"
    );
    assert!(checked_policy_snapshot_total(u64::MAX, 1, 0, u64::MAX).is_err());
    assert!(checked_policy_snapshot_total(u64::MAX - 1, 1, 1, u64::MAX).is_err());
}

fn initial_world() -> World {
    World::new_with_policies(
        setup(),
        7,
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap()
}
fn append_world(recording: &mut PolicyRecording, w: &World, limit: u64) -> Checked<()> {
    recording.append(
        w.snapshot()?,
        w.traffic_snapshot()?,
        w.excavation_snapshot()?,
        limit,
    )
}

#[test]
fn failed_triple_append_preserves_all_previous_vectors_and_totals() {
    let w = initial_world();
    let total = compact_size(&w.snapshot().unwrap())
        + compact_size(&w.traffic_snapshot().unwrap())
        + compact_size(&w.excavation_snapshot().unwrap());
    let mut recording = PolicyRecording::default();
    append_world(&mut recording, &w, total).unwrap();
    let before = recording.clone();
    // Synthetic accumulator repeats a real frame to exercise cumulative limits;
    // it is not an episode with a fabricated seeded trajectory.
    let errors = append_world(&mut recording, &w, total * 2 - 1).unwrap_err();
    assert_eq!(errors[0].field, "excavation_snapshot_bytes");
    assert_eq!(recording, before);
}

#[test]
fn triple_append_rejects_each_checkpoint_policy_and_id_fault_atomically() {
    let w = initial_world();
    for fault in 0..12 {
        let mut recording = PolicyRecording::default();
        append_world(&mut recording, &w, u64::MAX).unwrap();
        let before = recording.clone();
        let mut base = w.snapshot().unwrap();
        let mut traffic = w.traffic_snapshot().unwrap();
        let mut excavation = w.excavation_snapshot().unwrap();
        match fault {
            0 => base.summary.completed_ticks += 1,
            1 => traffic.completed_ticks += 1,
            2 => excavation.completed_ticks += 1,
            3 => traffic.policy = TrafficPolicy::ShortestOnlyV1,
            4 => excavation.traffic_policy = TrafficPolicy::ShortestOnlyV1,
            5 => excavation.excavation_policy = ExcavationPolicy::ExplorationFirstV1,
            6 => base.agents[0].id = 3,
            7 => traffic.agents[0].id = 3,
            8 => excavation.agents[0].id = 3,
            9 => base.agents.clear(),
            10 => traffic.agents.clear(),
            11 => excavation.agents.clear(),
            _ => unreachable!(),
        }
        assert!(
            recording
                .append(base, traffic, excavation, u64::MAX)
                .is_err(),
            "fault {fault}"
        );
        assert_eq!(recording, before, "fault {fault}");
    }
    let mut fresh = PolicyRecording::default();
    let mut excavation = w.excavation_snapshot().unwrap();
    excavation.traffic_policy = TrafficPolicy::ShortestOnlyV1;
    assert!(fresh
        .append(
            w.snapshot().unwrap(),
            w.traffic_snapshot().unwrap(),
            excavation,
            u64::MAX
        )
        .is_err());
    assert_eq!(fresh, PolicyRecording::default());
}

#[test]
fn changing_both_traffic_labels_still_rejects_cross_checkpoint_policy_change() {
    let w = initial_world();
    let mut recording = PolicyRecording::default();
    append_world(&mut recording, &w, u64::MAX).unwrap();
    let before = recording.clone();
    let mut traffic = w.traffic_snapshot().unwrap();
    let mut excavation = w.excavation_snapshot().unwrap();
    traffic.policy = TrafficPolicy::ShortestOnlyV1;
    excavation.traffic_policy = TrafficPolicy::ShortestOnlyV1;
    assert!(recording
        .append(w.snapshot().unwrap(), traffic, excavation, u64::MAX)
        .is_err());
    assert_eq!(recording, before);
}

#[test]
fn triple_append_rejects_equal_but_nonascending_worker_rows() {
    let w = traffic_support::opposing_queue(TrafficPolicy::LocalYieldingV1);
    let mut base = w.snapshot().unwrap();
    let mut traffic = w.traffic_snapshot().unwrap();
    let mut excavation = w.excavation_snapshot().unwrap();
    base.agents.swap(0, 1);
    traffic.agents.swap(0, 1);
    excavation.agents.swap(0, 1);
    let mut recording = PolicyRecording::default();
    assert!(recording
        .append(base, traffic, excavation, u64::MAX)
        .is_err());
    assert_eq!(recording, PolicyRecording::default());
}

#[test]
fn sixty_four_mib_triple_cap_is_cumulative_without_large_allocation() {
    let w = initial_world();
    let frame_bytes = compact_size(&w.snapshot().unwrap())
        + compact_size(&w.traffic_snapshot().unwrap())
        + compact_size(&w.excavation_snapshot().unwrap());
    let limit = 64 * 1024 * 1024;
    // Synthetic previously counted bytes; no claim of an actual seeded episode.
    let mut recording = PolicyRecording {
        base_bytes: limit - frame_bytes,
        ..Default::default()
    };
    append_world(&mut recording, &w, limit).unwrap();
    assert_eq!(
        recording.base_bytes + recording.traffic_bytes + recording.excavation_bytes,
        limit
    );
    let before = recording.clone();
    assert!(append_world(&mut recording, &w, limit).is_err());
    assert_eq!(recording, before);
}

#[test]
fn overflow_in_each_stream_preserves_all_recording_state() {
    let w = initial_world();
    for component in 0..3 {
        let mut recording = PolicyRecording::default();
        append_world(&mut recording, &w, u64::MAX).unwrap();
        match component {
            0 => recording.base_bytes = u64::MAX,
            1 => recording.traffic_bytes = u64::MAX,
            2 => recording.excavation_bytes = u64::MAX,
            _ => unreachable!(),
        }
        let before = recording.clone();
        assert!(append_world(&mut recording, &w, u64::MAX).is_err());
        assert_eq!(recording, before);
    }
}

#[test]
fn historical_views_reject_corrupt_cost_geometry_reference_and_tick_records() {
    let mut valid = excavation_support::informed_world(
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    );
    traffic_support::advance_scripted(&mut valid, &mut Scripted::new(&[0.0])).unwrap();
    assert_eq!(valid.excavation_summary().unwrap().counts.initiated_digs, 1);
    for fault in 0..8 {
        let mut w = valid.clone();
        let record = w.state.agents[0]
            .excavation
            .last_assessment
            .as_mut()
            .unwrap();
        match fault {
            0 => record.selected_candidate.as_mut().unwrap().estimated_cost += 1,
            1 => record.selected_candidate.as_mut().unwrap().approach = pos(0, 0),
            2 => {
                record.reference_cost =
                    Some(record.selected_candidate.as_ref().unwrap().estimated_cost)
            }
            3 => record.tick = 0,
            4 => record.tick = w.state.tick + 1,
            5 => record.site = pos(w.setup.width, 0),
            6 => {
                record
                    .selected_candidate
                    .as_mut()
                    .unwrap()
                    .pre_opening_distance = Some(1)
            }
            7 => record.best_candidate = None,
            _ => unreachable!(),
        }
        let before = w.state.clone();
        let rng = continuation(&w);
        assert!(w.excavation_summary().is_err(), "fault {fault}");
        assert!(w.excavation_snapshot().is_err(), "fault {fault}");
        assert_eq!(w.state, before);
        assert_eq!(continuation(&w), rng);
    }
}

#[test]
fn fixture_cutoffs_preserve_carried_spoil_prior_assessment_and_pending_hold() {
    let mut w = excavation_support::informed_world(
        TrafficPolicy::LocalYieldingV1,
        ExcavationPolicy::InformedCostV1,
    );
    traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    let mut recording = PolicyRecording::default();
    append_world(&mut recording, &w, 64 * 1024 * 1024).unwrap();
    assert!(matches!(
        recording.base_snapshots[0].agents[0].cargo,
        Some(Cargo::Spoil(_))
    ));
    assert_eq!(
        recording.excavation_snapshots[0].agents[0]
            .counts
            .initiated_digs,
        1
    );
    let record = recording.excavation_snapshots[0].agents[0]
        .last_assessment
        .clone();
    traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    append_world(&mut recording, &w, 64 * 1024 * 1024).unwrap();
    assert_eq!(
        recording.excavation_snapshots[1].agents[0].last_assessment,
        record
    );
    assert!(record.as_ref().unwrap().tick < recording.excavation_snapshots[1].completed_ticks);

    let mut queue = traffic_support::opposing_queue(TrafficPolicy::LocalYieldingV1);
    for values in [
        &[0.5, 0.5][..],
        &[0.5, 0.5][..],
        traffic_support::QUEUE_THIRD_TICK_SCRIPT,
    ] {
        traffic_support::advance_scripted(&mut queue, &mut Scripted::new(values)).unwrap();
    }
    let mut holds = PolicyRecording::default();
    append_world(&mut holds, &queue, 64 * 1024 * 1024).unwrap();
    assert!(holds.traffic_snapshots[0].agents[0].hold_pending);
    let tick = queue.state.tick;
    let before = queue.state.clone();
    let rng = continuation(&queue);
    queue.excavation_summary().unwrap();
    queue.excavation_snapshot().unwrap();
    assert_eq!(queue.state, before);
    assert_eq!(continuation(&queue), rng);
    assert_eq!(holds.excavation_snapshots[0].completed_ticks, tick);
}

// Supplemental GREEN coverage: the primary RED/GREEN tests above establish
// projection and append behavior; these paid fixtures cover additional histories.
fn selected_fixture_tick(w: &mut World, values: &[f64]) {
    use super::super::decision::{decide_with_policies, Policy};
    let mut draws = Scripted::new(values);
    let decision = decide_with_policies(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &traffic_support::view(w, 0),
        w.state.traffic_policy,
        w.state.excavation_policy,
        w.state.tick + 1,
        &mut draws,
    )
    .unwrap();
    traffic_support::decision_tick(w, 0, decision, &mut draws);
}

#[test]
fn revised_policy_hold_cutoff_retains_assessment_through_the_paid_hold() {
    let mut w = excavation_support::tied_world(TrafficPolicy::LocalYieldingV1, true);
    for script in [&[0.0][..], &[0.0][..], &[0.0, 0.0, 0.0][..]] {
        selected_fixture_tick(&mut w, script);
    }
    let before = w.state.clone();
    let rng = continuation(&w);
    let mut recording = PolicyRecording::default();
    append_world(&mut recording, &w, 64 * 1024 * 1024).unwrap();
    assert!(recording.traffic_snapshots[0].agents[0].hold_pending);
    let history = recording.excavation_snapshots[0].agents[0].clone();
    assert_eq!(history.counts.assessments, 3);
    assert_eq!(history.counts.selected_opportunities, 3);
    assert_eq!(history.last_assessment.as_ref().unwrap().tick, w.state.tick);
    assert_eq!(w.state, before);
    assert_eq!(continuation(&w), rng);
    selected_fixture_tick(&mut w, &[]);
    assert!(!w.state.agents[0].traffic.hold_pending);
    assert_eq!(w.excavation_snapshot().unwrap().agents[0], history);
    assert!(history.last_assessment.unwrap().tick < w.state.tick);
    assert!(recording.traffic_snapshots[0].agents[0].hold_pending);
}

#[test]
fn historical_assessment_remains_valid_when_another_worker_opens_its_face() {
    use super::super::decision::Action;
    let mut w = excavation_support::tied_world(TrafficPolicy::ShortestOnlyV1, true);
    selected_fixture_tick(&mut w, &[0.0]);
    let history = w.excavation_snapshot().unwrap().agents[0].clone();
    let face = history
        .last_assessment
        .as_ref()
        .unwrap()
        .selected_candidate
        .as_ref()
        .unwrap()
        .face;
    assert_eq!(face, pos(3, 4));
    traffic_support::fixture_tick(&mut w, 1, Action::Dig(face));
    assert!(w.snapshot().unwrap().open.contains(&face));
    assert_eq!(w.excavation_summary().unwrap().agents[0], history);
    assert_eq!(w.excavation_snapshot().unwrap().agents[0], history);
    assert!(history.last_assessment.unwrap().tick < w.state.tick);
}

#[test]
fn paid_multiworker_summary_sums_counters_and_takes_the_maximum_queue() {
    use super::super::decision::{Action, WaitReason};
    let mut setup = excavation_support::informed_setup();
    setup.workers = vec![pos(2, 1), pos(2, 1)];
    let mut w = World::new_with_policies(
        setup,
        7,
        TrafficPolicy::ShortestOnlyV1,
        ExcavationPolicy::InformedCostV1,
    )
    .unwrap();
    excavation_support::prepare(&mut w, false);
    // The second worker independently learns the same ring by ordinary paid moves.
    let mut path = (2..=5).map(|y| pos(2, y)).collect::<Vec<_>>();
    path.extend([pos(3, 5), pos(4, 5)]);
    path.extend((1..=4).rev().map(|y| pos(4, y)));
    path.extend((2..=5).map(|y| pos(4, y)));
    path.extend([pos(3, 5), pos(2, 5)]);
    path.extend((1..=4).rev().map(|y| pos(2, y)));
    path.extend([pos(1, 1), pos(2, 1)]);
    for target in path {
        traffic_support::fixture_tick(&mut w, 1, Action::Move(target));
    }
    traffic_support::controlled_tick(&mut w, 1, Action::Wait(WaitReason::NoNeighbor), |agent| {
        agent.site = Some(pos(4, 1));
        agent.phase = FoodPhase::Departing;
        agent.frontier = None;
        agent.face = None;
    });
    traffic_support::advance_scripted(&mut w, &mut Scripted::new(&[0.0, 0.0])).unwrap();
    let a = w.state.agents[0].excavation.counts;
    let b = w.state.agents[1].excavation.counts;
    assert_eq!((a.assessments, b.assessments), (1, 1));
    assert!(a.peak_queue > 0 && b.peak_queue > 0);
    let summary = w.excavation_summary().unwrap();
    assert_eq!(
        summary.counts,
        ExcavationCounts {
            assessments: 2,
            faces_scanned: a.faces_scanned + b.faces_scanned,
            candidate_pairs: a.candidate_pairs + b.candidate_pairs,
            selected_opportunities: 1,
            initiated_digs: 1,
            distance_calls: a.distance_calls + b.distance_calls,
            distance_visits: a.distance_visits + b.distance_visits,
            peak_queue: a.peak_queue.max(b.peak_queue),
        }
    );
    assert_eq!(
        summary
            .agents
            .iter()
            .map(|agent| agent.id)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(w.excavation_snapshot().unwrap().agents, summary.agents);
}
