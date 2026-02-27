use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancingOfferRecord {
    pub offer_id: String,
    pub organizer_id: String,
    pub event_id: String,
    pub financier_wallet: Option<String>,
    pub advance_bps: Option<u16>,
    pub fee_bps: Option<u16>,
    pub cap_amount: Option<u64>,
    pub status: Option<String>,
    pub freeze_enabled: Option<bool>,
    pub updated_at_epoch: Option<u64>,
}
