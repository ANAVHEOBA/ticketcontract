use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketClassRecord {
    pub class_id: String,
    pub event_id: String,
    pub organizer_id: String,
    pub name: Option<String>,
    pub status: Option<String>,
    pub supply_total: Option<u64>,
    pub supply_reserved: Option<u64>,
    pub supply_sold: Option<u64>,
    pub updated_at_epoch: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TicketClassAnalytics {
    pub class_id: String,
    pub event_id: String,
    pub organizer_id: String,
    pub supply_total: u64,
    pub supply_reserved: u64,
    pub supply_sold: u64,
    pub supply_remaining: u64,
    pub pacing_ratio: f64,
}
