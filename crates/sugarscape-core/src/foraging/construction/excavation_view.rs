use super::{
    excavation_state::include_counts, Checked, ExcavationAgentView, ExcavationCounts,
    ExcavationSnapshot, ExcavationSummary, World,
};

impl World {
    /// Checked historical excavation totals; consumes no observations, searches or draws.
    pub fn excavation_summary(&self) -> Checked<ExcavationSummary> {
        super::controller::check(&self.setup, &self.state)?;
        let mut counts = ExcavationCounts::default();
        for agent in &self.state.agents {
            include_counts(&mut counts, &agent.excavation.counts)?;
        }
        Ok(ExcavationSummary {
            traffic_policy: self.state.traffic_policy,
            excavation_policy: self.state.excavation_policy,
            completed_ticks: self.state.tick,
            counts,
            agents: self.excavation_agents(),
        })
    }
    /// Clones checked worker records at their original assessment ticks.
    /// An assessed face may already be open and the final worker may carry spoil.
    pub fn excavation_snapshot(&self) -> Checked<ExcavationSnapshot> {
        super::controller::check(&self.setup, &self.state)?;
        Ok(ExcavationSnapshot {
            traffic_policy: self.state.traffic_policy,
            excavation_policy: self.state.excavation_policy,
            completed_ticks: self.state.tick,
            agents: self.excavation_agents(),
        })
    }
    fn excavation_agents(&self) -> Vec<ExcavationAgentView> {
        self.state
            .agents
            .iter()
            .map(|agent| ExcavationAgentView {
                id: agent.id,
                counts: agent.excavation.counts,
                last_assessment: agent.excavation.last_assessment.clone(),
            })
            .collect()
    }
}
