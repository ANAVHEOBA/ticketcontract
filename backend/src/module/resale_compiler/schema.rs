use serde::{Deserialize, Serialize};

use super::model::{PolicyCandidate, PolicySimulation, RecommendedPolicy, ResaleHistoryInputs};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulationGoals {
    #[serde(default = "default_liquidity_weight")]
    pub liquidity_weight: f64,
    #[serde(default = "default_fairness_weight")]
    pub fairness_weight: f64,
    #[serde(default = "default_royalty_weight")]
    pub royalty_weight: f64,
}

#[derive(Debug, Deserialize)]
pub struct ResaleSimulationRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub estimated_face_value: Option<u64>,
    pub goals: Option<SimulationGoals>,
    #[serde(default)]
    pub candidates: Vec<PolicyCandidate>,
}

#[derive(Debug, Serialize)]
pub struct ResaleSimulationResponse {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub goals: SimulationGoals,
    pub inputs: ResaleHistoryInputs,
    pub simulations: Vec<PolicySimulation>,
    pub recommendation: Option<RecommendedPolicy>,
}

fn default_liquidity_weight() -> f64 {
    0.5
}

fn default_fairness_weight() -> f64 {
    0.3
}

fn default_royalty_weight() -> f64 {
    0.2
}
