use serde::{Deserialize, Serialize};

use super::model::Role;

#[derive(Debug, Deserialize)]
pub struct NonceRequest {
    pub wallet: String,
}

#[derive(Debug, Serialize)]
pub struct NonceResponse {
    pub wallet: String,
    pub nonce: String,
    pub message: String,
    pub expires_at_epoch: u64,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub wallet: String,
    pub nonce: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct ProviderVerifyRequest {
    pub provider: String,
    pub id_token: String,
    pub wallet: String,
    pub nonce: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub role: Role,
    pub organizer_scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub wallet: String,
    pub role: Role,
    pub organizer_scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OrganizerAccessResponse {
    pub allowed: bool,
    pub organizer_id: String,
    pub role: Role,
}
