use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListingRecord {
    pub listing_id: String,
    pub organizer_id: String,
    pub event_id: String,
    pub class_id: String,
    pub ticket_id: String,
    pub seller_wallet: Option<String>,
    pub ask_price: Option<u64>,
    pub status: Option<String>,
    pub expires_at_epoch: Option<u64>,
    pub updated_at_epoch: Option<u64>,
}
