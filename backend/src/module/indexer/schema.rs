use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct BackfillRequest {
    pub start_slot: u64,
    pub end_slot: u64,
}

#[derive(Debug, Deserialize)]
pub struct FinancingKpiQuery {
    pub organizer_id: String,
    pub event_id: String,
}

#[derive(Debug, Serialize)]
pub struct BackfillResponse {
    pub queued: bool,
}

#[derive(Debug, Serialize)]
pub struct IndexerStatusResponse {
    pub enabled: bool,
    pub running: bool,
    pub last_poll_epoch: i64,
    pub last_processed_slot: i64,
    pub last_signature: Option<String>,
    pub backfill_active: bool,
    pub backfill_pending: usize,
}

#[derive(Debug, Serialize)]
pub struct KpiRefreshResponse {
    pub refreshed: bool,
}
