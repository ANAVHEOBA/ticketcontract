use serde::{Deserialize, Serialize};

use super::model::UnderwritingDecision;

#[derive(Debug, Deserialize)]
pub struct UnderwritingRequest {
    pub organizer_id: String,
    pub event_id: String,
    pub requested_advance_amount: u64,
    pub projected_gross_revenue: u64,
    #[serde(default = "default_tenor_days")]
    pub tenor_days: u16,
}

#[derive(Debug, Serialize)]
pub struct UnderwritingResponse {
    pub decision: UnderwritingDecision,
}

fn default_tenor_days() -> u16 {
    45
}
