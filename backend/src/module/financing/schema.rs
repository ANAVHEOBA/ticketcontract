use serde::{Deserialize, Serialize};

use super::model::FinancingOfferRecord;

#[derive(Debug, Deserialize)]
pub struct FinancingTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub offer_id: Option<String>,
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
pub struct FinancingSimRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub offer_id: Option<String>,
    pub transaction_base64: String,
    #[serde(default)]
    pub sig_verify: bool,
    #[serde(default = "default_true")]
    pub replace_recent_blockhash: bool,
}

#[derive(Debug, Serialize)]
pub struct FinancingActionResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub offer_id: Option<String>,
    pub signature: String,
    pub confirmation_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FinancingSimResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub offer_id: Option<String>,
    pub err: Option<serde_json::Value>,
    pub logs: Vec<String>,
    pub units_consumed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct FinancingReadResponse {
    pub offer: FinancingOfferRecord,
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
