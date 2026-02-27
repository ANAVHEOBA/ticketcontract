use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ChainContextResponse {
    pub cluster: String,
    pub rpc_url: String,
    pub commitment: String,
    pub program_id: String,
    pub anchor_idl_address: Option<String>,
    pub idl_loaded: bool,
}

#[derive(Debug, Deserialize)]
pub struct DerivePdaRequest {
    pub seeds: Vec<SeedInput>,
}

#[derive(Debug, Deserialize)]
pub struct SeedInput {
    pub value: String,
    #[serde(default = "default_seed_encoding")]
    pub encoding: SeedEncoding,
}

fn default_seed_encoding() -> SeedEncoding {
    SeedEncoding::Utf8
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedEncoding {
    Utf8,
    Hex,
    Base58,
}

#[derive(Debug, Serialize)]
pub struct DerivePdaResponse {
    pub pda: String,
    pub bump: u8,
}

#[derive(Debug, Deserialize)]
pub struct SimulateTransactionRequest {
    pub transaction_base64: String,
    #[serde(default)]
    pub sig_verify: bool,
    #[serde(default = "default_true")]
    pub replace_recent_blockhash: bool,
}

#[derive(Debug, Serialize)]
pub struct SimulateTransactionResponse {
    pub err: Option<serde_json::Value>,
    pub logs: Vec<String>,
    pub units_consumed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    pub transaction_base64: String,
    #[serde(default)]
    pub skip_preflight: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
}

#[derive(Debug, Serialize)]
pub struct SubmitTransactionResponse {
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmSignatureRequest {
    pub signature: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_poll_ms")]
    pub poll_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ConfirmSignatureResponse {
    pub confirmed: bool,
    pub confirmation_status: Option<String>,
    pub err: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitAndConfirmRequest {
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

#[derive(Debug, Serialize)]
pub struct SubmitAndConfirmResponse {
    pub signature: String,
    pub confirmation_status: Option<String>,
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
