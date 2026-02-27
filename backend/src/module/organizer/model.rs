use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizerRecord {
    pub organizer_id: String,
    pub owner_wallet: Option<String>,
    pub metadata_uri: Option<String>,
    pub payout_wallet: Option<String>,
    pub status: Option<String>,
    pub compliance_flags: Option<Vec<String>>,
    pub operators: Option<Vec<String>>,
    pub updated_at_epoch: Option<u64>,
}
