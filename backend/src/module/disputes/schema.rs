use serde::{Deserialize, Serialize};

use super::model::DisputeRecord;

#[derive(Debug, Deserialize)]
pub struct DisputeTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub ticket_id: String,
    pub dispute_id: Option<String>,
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
pub struct DisputeQueueQuery {
    pub organizer_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct DisputeActionResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub ticket_id: String,
    pub dispute_id: Option<String>,
    pub signature: String,
    pub confirmation_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DisputeQueueResponse {
    pub items: Vec<DisputeRecord>,
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
