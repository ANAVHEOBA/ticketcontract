use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderwritingHistoryMetrics {
    pub organizer_id: String,
    pub event_id: String,
    pub primary_sales_count: u64,
    pub checked_in_count: u64,
    pub refunded_count: u64,
    pub disputes_count: u64,
    pub chargebacks_count: u64,
    pub resale_listed_count: u64,
    pub resale_completed_count: u64,
    pub trust_signal_total_delta: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplainabilityItem {
    pub factor: String,
    pub direction: String,
    pub impact: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepaymentMilestone {
    pub label: String,
    pub share_bps: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinancingTermProposal {
    pub advance_bps: u16,
    pub fee_bps: u16,
    pub cap_amount: u64,
    pub schedule: Vec<RepaymentMilestone>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnderwritingDecision {
    pub organizer_id: String,
    pub event_id: String,
    pub risk_score: f64,
    pub risk_tier: String,
    pub metrics: UnderwritingHistoryMetrics,
    pub proposal: FinancingTermProposal,
    pub explainability: Vec<ExplainabilityItem>,
    pub generated_at_epoch: u64,
}
