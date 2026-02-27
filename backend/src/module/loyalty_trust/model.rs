use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoyaltyLedgerRecord {
    pub wallet: String,
    pub organizer_id: String,
    pub event_id: Option<String>,
    pub points_balance: Option<i64>,
    pub points_earned: Option<i64>,
    pub points_redeemed: Option<i64>,
    pub updated_at_epoch: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrustSignalRecord {
    pub signal_id: String,
    pub wallet: String,
    pub organizer_id: String,
    pub event_id: String,
    pub signal_type: String,
    pub schema_version: Option<u32>,
    pub score_delta: Option<i32>,
    pub created_at_epoch: Option<u64>,
}
