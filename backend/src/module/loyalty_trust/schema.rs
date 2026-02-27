use serde::{Deserialize, Serialize};

use super::model::{LoyaltyLedgerRecord, TrustSignalRecord};

#[derive(Debug, Deserialize)]
pub struct LoyaltyTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub wallet: String,
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
pub struct TrustSignalTxRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub wallet: String,
    pub signal_id: Option<String>,
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
pub struct TrustSchemaTxRequest {
    pub organizer_id: Option<String>,
    pub schema_version: u32,
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
pub struct LoyaltyQuery {
    pub wallet: String,
    pub organizer_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrustSignalQuery {
    pub wallet: Option<String>,
    pub organizer_id: Option<String>,
    pub event_id: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct LoyaltyTrustActionResponse {
    pub action: &'static str,
    pub organizer_id: Option<String>,
    pub event_id: Option<String>,
    pub wallet: Option<String>,
    pub signal_id: Option<String>,
    pub signature: String,
    pub confirmation_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoyaltyReadResponse {
    pub rows: Vec<LoyaltyLedgerRecord>,
}

#[derive(Debug, Serialize)]
pub struct TrustSignalReadResponse {
    pub rows: Vec<TrustSignalRecord>,
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
