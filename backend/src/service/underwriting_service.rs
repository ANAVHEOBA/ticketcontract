use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    error::ApiError,
    module::underwriting::{
        crud,
        model::{
            ExplainabilityItem, FinancingTermProposal, RepaymentMilestone, UnderwritingDecision,
            UnderwritingHistoryMetrics,
        },
        schema::UnderwritingRequest,
    },
};

#[derive(Clone)]
pub struct UnderwritingService {
    mongo: mongodb::Client,
}

impl UnderwritingService {
    pub fn new(mongo: mongodb::Client) -> Self {
        Self { mongo }
    }

    pub async fn evaluate(
        &self,
        request: UnderwritingRequest,
    ) -> Result<UnderwritingDecision, ApiError> {
        if request.projected_gross_revenue == 0 {
            return Err(ApiError::BadRequest(
                "projected_gross_revenue must be > 0".to_string(),
            ));
        }

        let metrics =
            crud::load_history_metrics(&self.mongo, &request.organizer_id, &request.event_id)
                .await?;

        let risk_score = compute_risk_score(&metrics);
        let risk_tier = risk_tier_from_score(risk_score).to_string();
        let proposal = build_financing_terms(&request, risk_score);
        let explainability = build_explainability(&metrics, risk_score, &proposal);

        Ok(UnderwritingDecision {
            organizer_id: request.organizer_id,
            event_id: request.event_id,
            risk_score,
            risk_tier,
            metrics,
            proposal,
            explainability,
            generated_at_epoch: now_epoch(),
        })
    }
}

fn compute_risk_score(metrics: &UnderwritingHistoryMetrics) -> f64 {
    let sales = metrics.primary_sales_count.max(1) as f64;
    let refund_ratio = metrics.refunded_count as f64 / sales;
    let dispute_ratio = metrics.disputes_count as f64 / sales;
    let chargeback_ratio = metrics.chargebacks_count as f64 / sales;
    let checkin_ratio = metrics.checked_in_count as f64 / sales;
    let resale_fill = if metrics.resale_listed_count == 0 {
        0.0
    } else {
        metrics.resale_completed_count as f64 / metrics.resale_listed_count as f64
    };

    let history_credit = (metrics.primary_sales_count as f64).sqrt().min(12.0);
    let trust_credit = (metrics.trust_signal_total_delta.max(0) as f64)
        .sqrt()
        .min(10.0);

    let mut score = 50.0;
    score += history_credit;
    score += checkin_ratio.min(1.0) * 18.0;
    score += resale_fill.min(1.0) * 10.0;
    score += trust_credit;
    score -= refund_ratio.min(1.0) * 28.0;
    score -= dispute_ratio.min(1.0) * 20.0;
    score -= chargeback_ratio.min(1.0) * 35.0;

    score.clamp(0.0, 100.0)
}

fn risk_tier_from_score(score: f64) -> &'static str {
    if score >= 80.0 {
        "low_risk"
    } else if score >= 60.0 {
        "medium_risk"
    } else {
        "high_risk"
    }
}

fn build_financing_terms(request: &UnderwritingRequest, risk_score: f64) -> FinancingTermProposal {
    let advance_bps = (3000.0 + risk_score * 45.0).round().clamp(2500.0, 8000.0) as u16;
    let fee_bps = (1200.0 - risk_score * 7.0).round().clamp(250.0, 1200.0) as u16;
    let projected_cap =
        ((request.projected_gross_revenue as u128 * advance_bps as u128) / 10_000) as u64;
    let cap_amount = projected_cap.min(request.requested_advance_amount.saturating_mul(12) / 10);
    let schedule = build_schedule(request.tenor_days, risk_score);

    FinancingTermProposal {
        advance_bps,
        fee_bps,
        cap_amount,
        schedule,
    }
}

fn build_schedule(tenor_days: u16, risk_score: f64) -> Vec<RepaymentMilestone> {
    if risk_score >= 80.0 {
        vec![
            RepaymentMilestone {
                label: "t+0".to_string(),
                share_bps: 5000,
            },
            RepaymentMilestone {
                label: format!("t+{}", tenor_days / 2),
                share_bps: 3000,
            },
            RepaymentMilestone {
                label: format!("t+{tenor_days}"),
                share_bps: 2000,
            },
        ]
    } else if risk_score >= 60.0 {
        vec![
            RepaymentMilestone {
                label: "t+0".to_string(),
                share_bps: 4000,
            },
            RepaymentMilestone {
                label: format!("t+{}", tenor_days / 2),
                share_bps: 3500,
            },
            RepaymentMilestone {
                label: format!("t+{tenor_days}"),
                share_bps: 2500,
            },
        ]
    } else {
        vec![
            RepaymentMilestone {
                label: "t+0".to_string(),
                share_bps: 3000,
            },
            RepaymentMilestone {
                label: format!("t+{}", tenor_days / 2),
                share_bps: 3500,
            },
            RepaymentMilestone {
                label: format!("t+{tenor_days}"),
                share_bps: 3500,
            },
        ]
    }
}

fn build_explainability(
    metrics: &UnderwritingHistoryMetrics,
    risk_score: f64,
    proposal: &FinancingTermProposal,
) -> Vec<ExplainabilityItem> {
    let sales = metrics.primary_sales_count.max(1) as f64;
    let refund_ratio = metrics.refunded_count as f64 / sales;
    let dispute_ratio = metrics.disputes_count as f64 / sales;
    let checkin_ratio = metrics.checked_in_count as f64 / sales;
    let resale_fill = if metrics.resale_listed_count == 0 {
        0.0
    } else {
        metrics.resale_completed_count as f64 / metrics.resale_listed_count as f64
    };

    vec![
        ExplainabilityItem {
            factor: "ticket_velocity".to_string(),
            direction: "positive".to_string(),
            impact: "higher history depth increases confidence".to_string(),
            value: metrics.primary_sales_count.to_string(),
        },
        ExplainabilityItem {
            factor: "attendance_quality".to_string(),
            direction: if checkin_ratio >= 0.5 {
                "positive".to_string()
            } else {
                "neutral".to_string()
            },
            impact: "higher check-in ratio improves risk score".to_string(),
            value: format!("{:.2}", checkin_ratio),
        },
        ExplainabilityItem {
            factor: "resale_liquidity".to_string(),
            direction: if resale_fill >= 0.5 {
                "positive".to_string()
            } else {
                "negative".to_string()
            },
            impact: "healthy resale fill supports repayment confidence".to_string(),
            value: format!("{:.2}", resale_fill),
        },
        ExplainabilityItem {
            factor: "refund_pressure".to_string(),
            direction: if refund_ratio > 0.08 {
                "negative".to_string()
            } else {
                "neutral".to_string()
            },
            impact: "higher refund ratio decreases offer terms".to_string(),
            value: format!("{:.3}", refund_ratio),
        },
        ExplainabilityItem {
            factor: "dispute_pressure".to_string(),
            direction: if dispute_ratio > 0.03 {
                "negative".to_string()
            } else {
                "neutral".to_string()
            },
            impact: "dispute and chargeback frequency lower advance".to_string(),
            value: format!("{:.3}", dispute_ratio),
        },
        ExplainabilityItem {
            factor: "recommended_terms".to_string(),
            direction: "summary".to_string(),
            impact: format!(
                "risk_score {:.1} mapped to advance {} bps and fee {} bps",
                risk_score, proposal.advance_bps, proposal.fee_bps
            ),
            value: proposal.cap_amount.to_string(),
        },
    ]
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
