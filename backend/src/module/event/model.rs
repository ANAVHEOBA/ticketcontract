use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub organizer_id: String,
    pub name: Option<String>,
    pub status: Option<String>,
    pub metadata_uri: Option<String>,
    pub resale_policy_snapshot: Option<serde_json::Value>,
    pub starts_at_epoch: Option<u64>,
    pub ends_at_epoch: Option<u64>,
    pub updated_at_epoch: Option<u64>,
}
