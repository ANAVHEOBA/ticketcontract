use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PrimarySaleTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub transaction_base64: String,
    pub buyer_wallet: Option<String>,
    pub ticket_pda: Option<String>,
    pub gross_amount: Option<u64>,
    pub protocol_fee_amount: Option<u64>,
    pub net_amount: Option<u64>,
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
pub struct PrimarySaleSimRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub transaction_base64: String,
    #[serde(default)]
    pub sig_verify: bool,
    #[serde(default = "default_true")]
    pub replace_recent_blockhash: bool,
}

#[derive(Debug, Serialize)]
pub struct PurchaseReceipt {
    pub signature: String,
    pub ticket_pda: Option<String>,
    pub buyer_wallet: Option<String>,
    pub gross_amount: Option<u64>,
    pub protocol_fee_amount: Option<u64>,
    pub net_amount: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PrimarySaleActionResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub confirmation_status: Option<String>,
    pub receipt: PurchaseReceipt,
}

#[derive(Debug, Serialize)]
pub struct PrimarySaleSimResponse {
    pub action: &'static str,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
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
