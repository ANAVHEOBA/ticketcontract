use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DisputeRecord {
    pub dispute_id: String,
    pub organizer_id: String,
    pub event_id: String,
    pub ticket_id: String,
    pub status: Option<String>,
    pub reason: Option<String>,
    pub chargeback: Option<bool>,
    pub created_at_epoch: Option<u64>,
    pub updated_at_epoch: Option<u64>,
}
