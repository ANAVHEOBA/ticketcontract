use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResaleHistoryInputs {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub listed_count: u64,
    pub sold_count: u64,
    pub cancelled_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCandidate {
    pub candidate_id: String,
    pub max_markup_bps: u16,
    pub royalty_bps: u16,
    #[serde(default)]
    pub whitelist_enabled: bool,
    #[serde(default)]
    pub blacklist_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicySimulation {
    pub candidate_id: String,
    pub valid: bool,
    pub validation_errors: Vec<String>,
    pub expected_fill_rate: f64,
    pub expected_markup_bps: u16,
    pub expected_royalty_yield: f64,
    pub liquidity_score: f64,
    pub fairness_score: f64,
    pub royalty_score: f64,
    pub objective_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendedPolicy {
    pub candidate_id: String,
    pub confidence: f64,
    pub rationale: String,
}
