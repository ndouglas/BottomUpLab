use super::{
    runner::{validate_options, CountingWriter, SNAPSHOT_LIMIT},
    Checked, Episode, RunOptions, Setup, Snapshot, TrafficPolicy, TrafficSnapshot, TrafficSummary,
    World,
};
use crate::config::FieldError;

/// Explicit policy provenance around the unchanged base episode payload.
/// Serialize-only observational output; it cannot restore a World.
/// Compact base-plus-traffic snapshot JSON is capped at 64 MiB. Counts exclude
/// this wrapper, final summaries and frame delimiters. A pending final hold is
/// a legal censored state; no extra opportunity is paid beyond the horizon.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct TrafficEpisode {
    pub schema: &'static str,
    pub policy: TrafficPolicy,
    pub episode: Episode,
    pub traffic_summary: TrafficSummary,
    pub traffic_snapshots: Vec<TrafficSnapshot>,
    pub traffic_snapshot_bytes: u64,
}
/// Runs the full horizon with explicitly labeled traffic policy and paired frames.
pub fn run_with_traffic(
    setup: Setup,
    seed: u64,
    options: RunOptions,
    policy: TrafficPolicy,
) -> Checked<TrafficEpisode> {
    run_with_traffic_limit(setup, seed, options, policy, SNAPSHOT_LIMIT)
}
pub(super) fn run_with_traffic_limit(
    setup: Setup,
    seed: u64,
    options: RunOptions,
    policy: TrafficPolicy,
    limit: u64,
) -> Checked<TrafficEpisode> {
    validate_options(&setup, &options)?;
    let mut world = World::new_with_traffic(setup, seed, policy)?;
    let mut recording = TrafficRecording {
        base_snapshots: vec![],
        traffic_snapshots: vec![],
        base_bytes: 0,
        traffic_bytes: 0,
    };
    if options.snapshots {
        recording.append(world.snapshot()?, world.traffic_snapshot()?, limit)?;
    }
    for completed in 1..=options.ticks {
        world.step()?;
        if options.snapshots
            && (completed % options.sample_every == 0 || completed == options.ticks)
        {
            recording.append(world.snapshot()?, world.traffic_snapshot()?, limit)?;
        }
    }
    Ok(TrafficEpisode {
        schema: "foraging-traffic-episode-v1",
        policy,
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
    })
}
pub(super) struct TrafficRecording {
    pub(super) base_snapshots: Vec<Snapshot>,
    pub(super) traffic_snapshots: Vec<TrafficSnapshot>,
    pub(super) base_bytes: u64,
    pub(super) traffic_bytes: u64,
}
impl TrafficRecording {
    pub(super) fn append(
        &mut self,
        base: Snapshot,
        traffic: TrafficSnapshot,
        limit: u64,
    ) -> Checked<()> {
        if base.summary.completed_ticks != traffic.completed_ticks {
            return Err(vec![FieldError::new(
                "traffic_snapshots",
                "base and traffic checkpoints must match",
            )]);
        }
        if self
            .traffic_snapshots
            .last()
            .is_some_and(|previous| previous.policy != traffic.policy)
        {
            return Err(vec![FieldError::new(
                "traffic_snapshots.policy",
                "policy must match previous checkpoints",
            )]);
        }
        if base.agents.len() != traffic.agents.len()
            || base
                .agents
                .iter()
                .zip(&traffic.agents)
                .enumerate()
                .any(|(id, (base, traffic))| base.id as usize != id || base.id != traffic.id)
        {
            return Err(vec![FieldError::new(
                "traffic_snapshots.agents",
                "base and traffic worker rows must match in ID order",
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
        checked_snapshot_total(next_base, next_traffic, limit)?;
        self.base_bytes = next_base;
        self.traffic_bytes = next_traffic;
        self.base_snapshots.push(base);
        self.traffic_snapshots.push(traffic);
        Ok(())
    }
}
pub(super) fn count_bytes<T: serde::Serialize>(
    snapshot: &T,
    bytes: u64,
    limit: u64,
    field: &str,
    tick: u32,
) -> Checked<u64> {
    let mut writer = CountingWriter { bytes, limit };
    serde_json::to_writer(&mut writer, snapshot).map_err(|error| {
        vec![FieldError::new(
            field,
            format!("completed tick {tick}: {error}; limit {limit} bytes"),
        )]
    })?;
    Ok(writer.bytes)
}
pub(super) fn checked_snapshot_total(base: u64, traffic: u64, limit: u64) -> Checked<u64> {
    base.checked_add(traffic)
        .filter(|total| *total <= limit)
        .ok_or_else(|| {
            vec![FieldError::new(
                "traffic_snapshot_bytes",
                "combined snapshot byte budget exceeded",
            )]
        })
}
