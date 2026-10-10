use super::{Checked, TrafficAgentView, TrafficCounts, TrafficSnapshot, TrafficSummary, World};

impl World {
    /// Checked observational traffic totals; consumes no draws or observations.
    pub fn traffic_summary(&self) -> Checked<TrafficSummary> {
        super::controller::check(&self.setup, &self.state)?;
        let mut counts = TrafficCounts::default();
        for agent in &self.state.agents {
            counts.checked_include(&agent.traffic.counts)?;
        }
        Ok(TrafficSummary {
            policy: self.state.traffic_policy,
            completed_ticks: self.state.tick,
            counts,
            agents: self.traffic_agents(),
        })
    }
    /// Clones checked worker traffic memory without changing state or RNG.
    pub fn traffic_snapshot(&self) -> Checked<TrafficSnapshot> {
        super::controller::check(&self.setup, &self.state)?;
        Ok(TrafficSnapshot {
            policy: self.state.traffic_policy,
            completed_ticks: self.state.tick,
            agents: self.traffic_agents(),
        })
    }
    fn traffic_agents(&self) -> Vec<TrafficAgentView> {
        self.state
            .agents
            .iter()
            .map(|agent| TrafficAgentView {
                id: agent.id,
                blocked_wait_streak: agent.traffic.blocked_wait_streak,
                hold_pending: agent.traffic.hold_pending,
                counts: agent.traffic.counts.clone(),
            })
            .collect()
    }
}
