use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResalePolicyRecord {
    pub policy_id: String,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub max_markup_bps: Option<u16>,
    pub royalty_bps: Option<u16>,
    pub whitelist_enabled: Option<bool>,
    pub blacklist_enabled: Option<bool>,
    pub status: Option<String>,
    pub updated_at_epoch: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResalePolicyRecommendation {
    pub recommendation_id: String,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub max_markup_bps: u16,
    pub royalty_bps: u16,
    pub confidence: f64,
    pub rationale: Option<String>,
    pub updated_at_epoch: u64,
}
