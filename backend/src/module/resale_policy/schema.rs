use serde::{Deserialize, Serialize};

use super::model::{ResalePolicyRecommendation, ResalePolicyRecord};

#[derive(Debug, Deserialize)]
pub struct ResalePolicyTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub transaction_base64: String,
    #[serde(default)]
    pub skip_preflight: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_poll_ms")]
    pub poll_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct ResalePolicySimRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub transaction_base64: String,
    #[serde(default)]
    pub sig_verify: bool,
    #[serde(default = "default_true")]
    pub replace_recent_blockhash: bool,
}

#[derive(Debug, Deserialize)]
pub struct PolicyValidationRequest {
    pub max_markup_bps: u16,
    pub royalty_bps: u16,
    #[serde(default)]
    pub whitelist_enabled: bool,
    #[serde(default)]
    pub blacklist_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct PolicyValidationResponse {
    pub valid: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecommendationWriteRequest {
    pub recommendation_id: String,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub max_markup_bps: u16,
    pub royalty_bps: u16,
    pub confidence: f64,
    pub rationale: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationWriteResponse {
    pub saved: bool,
    pub recommendation: ResalePolicyRecommendation,
}

#[derive(Debug, Serialize)]
pub struct ResalePolicyActionResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub signature: String,
    pub confirmation_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResalePolicySimResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: Option<String>,
    pub err: Option<serde_json::Value>,
    pub logs: Vec<String>,
    pub units_consumed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ResalePolicyReadResponse {
    pub policy: ResalePolicyRecord,
}

fn default_true() -> bool {
    true
}

fn default_max_retries() -> usize {
    5
}

fn default_timeout_ms() -> u64 {
    45_000
}

fn default_poll_ms() -> u64 {
    1_500
}
