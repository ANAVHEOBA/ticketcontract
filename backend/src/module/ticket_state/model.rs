use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketRecord {
    pub ticket_id: String,
    pub event_id: String,
    pub class_id: String,
    pub organizer_id: String,
    pub owner_wallet: Option<String>,
    pub status: Option<String>,
    pub metadata_uri: Option<String>,
    pub metadata_version: Option<u64>,
    pub updated_at_epoch: Option<u64>,
}
