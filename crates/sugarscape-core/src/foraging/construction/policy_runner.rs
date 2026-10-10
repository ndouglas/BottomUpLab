use super::{
    runner::{validate_options, SNAPSHOT_LIMIT},
    traffic_runner::count_bytes,
    Checked, Episode, ExcavationPolicy, ExcavationSnapshot, ExcavationSummary, RunOptions, Setup,
    Snapshot, TrafficPolicy, TrafficSnapshot, TrafficSummary, World,
};
use crate::config::FieldError;

/// Serialize-only provenance for independently selected construction policies.
/// The three compact snapshot streams share a 64 MiB cap, excluding wrapper
/// metadata, final summaries and frame delimiters. Final pending holds, carried
/// spoil and earlier assessments are legal censored states at the fixed horizon.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ConstructionPolicyEpisode {
    pub schema: &'static str,
    pub traffic_policy: TrafficPolicy,
    pub excavation_policy: ExcavationPolicy,
    pub episode: Episode,
    pub traffic_summary: TrafficSummary,
    pub traffic_snapshots: Vec<TrafficSnapshot>,
    pub traffic_snapshot_bytes: u64,
    pub excavation_summary: ExcavationSummary,
    pub excavation_snapshots: Vec<ExcavationSnapshot>,
    pub excavation_snapshot_bytes: u64,
}
/// Runs the complete requested horizon with aligned observational checkpoints.
pub fn run_with_policies(
    setup: Setup,
    seed: u64,
    options: RunOptions,
    traffic_policy: TrafficPolicy,
    excavation_policy: ExcavationPolicy,
) -> Checked<ConstructionPolicyEpisode> {
    run_with_policies_limit(
        setup,
        seed,
        options,
        traffic_policy,
        excavation_policy,
        SNAPSHOT_LIMIT,
    )
}
pub(super) fn run_with_policies_limit(
    setup: Setup,
    seed: u64,
    options: RunOptions,
    traffic_policy: TrafficPolicy,
    excavation_policy: ExcavationPolicy,
    limit: u64,
) -> Checked<ConstructionPolicyEpisode> {
    validate_options(&setup, &options)?;
    let mut world = World::new_with_policies(setup, seed, traffic_policy, excavation_policy)?;
    let mut recording = PolicyRecording::default();
    if options.snapshots {
        recording.append(
            world.snapshot()?,
            world.traffic_snapshot()?,
            world.excavation_snapshot()?,
            limit,
        )?;
    }
    for completed in 1..=options.ticks {
        world.step()?;
        if options.snapshots
            && (completed % options.sample_every == 0 || completed == options.ticks)
        {
            recording.append(
                world.snapshot()?,
                world.traffic_snapshot()?,
                world.excavation_snapshot()?,
                limit,
            )?;
        }
    }
    Ok(ConstructionPolicyEpisode {
        schema: "foraging-construction-policy-episode-v1",
        traffic_policy,
        excavation_policy,
        episode: Episode {
            setup: world.setup.clone(),
            seed,
            options,
            summary: world.summary()?,
            snapshots: recording.base_snapshots,
            snapshot_bytes: recording.base_bytes,
        },
        traffic_summary: world.traffic_summary()?,
        traffic_snapshots: recording.traffic_snapshots,
        traffic_snapshot_bytes: recording.traffic_bytes,
        excavation_summary: world.excavation_summary()?,
        excavation_snapshots: recording.excavation_snapshots,
        excavation_snapshot_bytes: recording.excavation_bytes,
    })
}
pub(super) fn checked_policy_snapshot_total(
    base: u64,
    traffic: u64,
    excavation: u64,
    limit: u64,
) -> Checked<u64> {
    base.checked_add(traffic)
        .and_then(|n| n.checked_add(excavation))
        .filter(|n| *n <= limit)
        .ok_or_else(|| {
            vec![FieldError::new(
                "excavation_snapshot_bytes",
                "combined snapshot byte budget exceeded",
            )]
        })
}
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct PolicyRecording {
    pub(super) base_snapshots: Vec<Snapshot>,
    pub(super) traffic_snapshots: Vec<TrafficSnapshot>,
    pub(super) excavation_snapshots: Vec<ExcavationSnapshot>,
    pub(super) base_bytes: u64,
    pub(super) traffic_bytes: u64,
    pub(super) excavation_bytes: u64,
}
impl PolicyRecording {
    pub(super) fn append(
        &mut self,
        base: Snapshot,
        traffic: TrafficSnapshot,
        excavation: ExcavationSnapshot,
        limit: u64,
    ) -> Checked<()> {
        if base.summary.completed_ticks != traffic.completed_ticks
            || traffic.completed_ticks != excavation.completed_ticks
        {
            return Err(vec![FieldError::new(
                "excavation_snapshots",
                "base, traffic and excavation checkpoints must match",
            )]);
        }
        if traffic.policy != excavation.traffic_policy
            || self
                .traffic_snapshots
                .last()
                .is_some_and(|previous| previous.policy != traffic.policy)
            || self.excavation_snapshots.last().is_some_and(|previous| {
                previous.traffic_policy != excavation.traffic_policy
                    || previous.excavation_policy != excavation.excavation_policy
            })
        {
            return Err(vec![FieldError::new(
                "excavation_snapshots.policy",
                "both policies must match within and across checkpoints",
            )]);
        }
        if base.agents.len() != traffic.agents.len()
            || base.agents.len() != excavation.agents.len()
            || base
                .agents
                .iter()
                .zip(&traffic.agents)
                .zip(&excavation.agents)
                .enumerate()
                .any(|(id, ((base, traffic), excavation))| {
                    base.id as usize != id || base.id != traffic.id || base.id != excavation.id
                })
        {
            return Err(vec![FieldError::new(
                "excavation_snapshots.agents",
                "base, traffic and excavation worker rows must match in ID order",
            )]);
        }
        let next_base = count_bytes(
            &base,
            self.base_bytes,
            limit,
            "snapshot_bytes",
            base.summary.completed_ticks,
        )?;
        let next_traffic = count_bytes(
            &traffic,
            self.traffic_bytes,
            limit,
            "traffic_snapshot_bytes",
            traffic.completed_ticks,
        )?;
        let next_excavation = count_bytes(
            &excavation,
            self.excavation_bytes,
            limit,
            "excavation_snapshot_bytes",
            excavation.completed_ticks,
        )?;
        checked_policy_snapshot_total(next_base, next_traffic, next_excavation, limit)?;
        self.base_bytes = next_base;
        self.traffic_bytes = next_traffic;
        self.excavation_bytes = next_excavation;
        self.base_snapshots.push(base);
        self.traffic_snapshots.push(traffic);
        self.excavation_snapshots.push(excavation);
        Ok(())
    }
}
