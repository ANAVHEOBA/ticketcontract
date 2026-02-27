use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CheckInPolicyTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
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
pub struct CheckInTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub ticket_id: String,
    pub gate_id: String,
    pub scanner_id: Option<String>,
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
pub struct CheckInSimRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub ticket_id: String,
    pub gate_id: String,
    pub transaction_base64: String,
    #[serde(default)]
    pub sig_verify: bool,
    #[serde(default = "default_true")]
    pub replace_recent_blockhash: bool,
}

#[derive(Debug, Serialize)]
pub struct CheckInPolicyActionResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub signature: String,
    pub confirmation_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GateCheckInPayload {
    pub gate_id: String,
    pub ticket_id: String,
    pub scanner_id: Option<String>,
    pub accepted: bool,
    pub reason: Option<String>,
    pub checked_in_at_epoch: u64,
}

#[derive(Debug, Serialize)]
pub struct CheckInActionResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub signature: String,
    pub confirmation_status: Option<String>,
    pub gate_payload: GateCheckInPayload,
}

#[derive(Debug, Serialize)]
pub struct CheckInSimResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub ticket_id: String,
    pub gate_id: String,
    pub err: Option<serde_json::Value>,
    pub logs: Vec<String>,
    pub units_consumed: Option<u64>,
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
