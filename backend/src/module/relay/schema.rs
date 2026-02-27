use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RelaySubmitRequest {
    pub transaction_base64: String,
    #[serde(default)]
    pub expected_instructions: Vec<String>,
    #[serde(default)]
    pub skip_preflight: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_poll_ms")]
    pub poll_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct RelaySubmitResponse {
    pub signature: String,
    pub confirmation_status: Option<String>,
}

fn default_max_retries() -> usize {
    20
}

fn default_timeout_ms() -> u64 {
    120_000
}

fn default_poll_ms() -> u64 {
    2_000
}
