use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    ProtocolAdmin,
    OrganizerAdmin,
    Operator,
    Scanner,
    Financier,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessRule {
    pub wallet: String,
    pub role: Role,
    #[serde(default)]
    pub organizer_scopes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessProfile {
    pub wallet: String,
    pub role: Role,
    pub organizer_scopes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct NonceRecord {
    pub nonce: String,
    pub message: String,
    pub expires_at_epoch: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub role: Role,
    pub organizer_scopes: Vec<String>,
    pub iat: u64,
    pub exp: u64,
}
