//! Independent excavation policy and compact private-estimate diagnostics.
use super::{Pos, TrafficPolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExcavationPolicy {
    #[serde(rename = "exploration_first_v1")]
    ExplorationFirstV1,
    #[serde(rename = "informed_cost_v1")]
    InformedCostV1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct ExcavationCounts {
    pub assessments: u64,
    pub faces_scanned: u64,
    pub candidate_pairs: u64,
    pub selected_opportunities: u64,
    pub initiated_digs: u64,
    pub distance_calls: u64,
    pub distance_visits: u64,
    pub peak_queue: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ExcavationCandidate {
    pub face: Pos,
    pub approach: Pos,
    pub approach_moves: u64,
    pub spoil_leg_moves: u64,
    pub continuation_moves: u64,
    pub pre_opening_distance: Option<u64>,
    pub estimated_cost: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ExcavationAssessment {
    pub tick: u32,
    pub site: Pos,
    pub reference_cost: Option<u64>,
    pub candidate_pairs: u64,
    pub best_candidate: Option<ExcavationCandidate>,
    pub selected_candidate: Option<ExcavationCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ExcavationAgentView {
    pub id: u32,
    pub counts: ExcavationCounts,
    pub last_assessment: Option<ExcavationAssessment>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ExcavationSummary {
    pub traffic_policy: TrafficPolicy,
    pub excavation_policy: ExcavationPolicy,
    pub completed_ticks: u32,
    pub counts: ExcavationCounts,
    pub agents: Vec<ExcavationAgentView>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ExcavationSnapshot {
    pub traffic_policy: TrafficPolicy,
    pub excavation_policy: ExcavationPolicy,
    pub completed_ticks: u32,
    pub agents: Vec<ExcavationAgentView>,
}
