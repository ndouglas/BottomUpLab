//! Versioned traffic policy and checked diagnostics, separate from paid work.
use super::{Checked, WorkCounts};
use crate::config::FieldError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TrafficPolicy {
    #[serde(rename = "shortest_only_v1")]
    ShortestOnlyV1,
    #[serde(rename = "local_yielding_v1")]
    LocalYieldingV1,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct TrafficCounts {
    pub blocked_route_opportunities: u64,
    pub yield_attempts: u64,
    pub yield_moves: u64,
    pub hold_waits: u64,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct TrafficAgentView {
    pub id: u32,
    pub blocked_wait_streak: u8,
    pub hold_pending: bool,
    pub counts: TrafficCounts,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct TrafficSummary {
    pub policy: TrafficPolicy,
    pub completed_ticks: u32,
    pub counts: TrafficCounts,
    pub agents: Vec<TrafficAgentView>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct TrafficSnapshot {
    pub policy: TrafficPolicy,
    pub completed_ticks: u32,
    pub agents: Vec<TrafficAgentView>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TrafficState {
    pub(super) blocked_wait_streak: u8,
    pub(super) hold_pending: bool,
    pub(super) counts: TrafficCounts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TrafficEffect {
    Normal,
    BlockedWait,
    YieldMove,
    Hold,
}

fn error(field: &str, message: &str) -> Vec<FieldError> {
    vec![FieldError::new(format!("traffic.{field}"), message)]
}
fn add(a: u64, b: u64, field: &str) -> Checked<u64> {
    a.checked_add(b)
        .ok_or_else(|| error(field, "counter overflow"))
}

impl TrafficCounts {
    pub(super) fn checked_include(&mut self, delta: &Self) -> Checked<()> {
        let candidate = Self {
            blocked_route_opportunities: add(
                self.blocked_route_opportunities,
                delta.blocked_route_opportunities,
                "blocked_route_opportunities",
            )?,
            yield_attempts: add(self.yield_attempts, delta.yield_attempts, "yield_attempts")?,
            yield_moves: add(self.yield_moves, delta.yield_moves, "yield_moves")?,
            hold_waits: add(self.hold_waits, delta.hold_waits, "hold_waits")?,
        };
        *self = candidate;
        Ok(())
    }
}

impl TrafficState {
    pub(super) fn commit(&mut self, effect: TrafficEffect) -> Checked<()> {
        let mut candidate = self.clone();
        candidate.blocked_wait_streak = 0;
        candidate.hold_pending = false;
        match effect {
            TrafficEffect::Normal => {}
            TrafficEffect::BlockedWait => {
                candidate.blocked_wait_streak = self.blocked_wait_streak.saturating_add(1).min(2);
            }
            TrafficEffect::YieldMove => {
                candidate.counts.checked_include(&TrafficCounts {
                    yield_moves: 1,
                    ..Default::default()
                })?;
                candidate.hold_pending = true;
            }
            TrafficEffect::Hold => candidate.counts.checked_include(&TrafficCounts {
                hold_waits: 1,
                ..Default::default()
            })?,
        }
        *self = candidate;
        Ok(())
    }

    pub(super) fn check(&self, policy: TrafficPolicy, work: &WorkCounts) -> Checked<()> {
        if self.blocked_wait_streak > 2 {
            return Err(error("blocked_wait_streak", "must be at most two"));
        }
        if self.hold_pending && self.blocked_wait_streak != 0 {
            return Err(error(
                "hold_pending",
                "pending hold requires zero blocked streak",
            ));
        }
        let c = &self.counts;
        if policy == TrafficPolicy::ShortestOnlyV1
            && (self.hold_pending
                || c.yield_attempts != 0
                || c.yield_moves != 0
                || c.hold_waits != 0)
        {
            return Err(error(
                "policy",
                "shortest_only_v1 cannot attempt yielding, yield or hold",
            ));
        }
        let congestion = add(
            add(
                work.empty_congestion_waits,
                work.food_congestion_waits,
                "congestion_waits",
            )?,
            work.spoil_congestion_waits,
            "congestion_waits",
        )?;
        for (field, invalid) in [
            ("yield_moves", c.yield_moves > c.yield_attempts),
            (
                "yield_attempts",
                c.yield_attempts > c.blocked_route_opportunities,
            ),
            ("hold_waits", c.hold_waits > c.yield_moves),
            (
                "blocked_route_opportunities",
                c.blocked_route_opportunities > work.opportunities,
            ),
            ("yield_moves", c.yield_moves > work.moves),
            ("hold_waits", c.hold_waits > congestion),
        ] {
            if invalid {
                return Err(error(field, "exceeds traffic or paid-work bound"));
            }
        }
        Ok(())
    }
}
