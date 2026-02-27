use crate::{
    error::ApiError,
    module::resale_compiler::{
        crud,
        model::{PolicyCandidate, PolicySimulation, RecommendedPolicy},
        schema::{ResaleSimulationRequest, ResaleSimulationResponse, SimulationGoals},
    },
};

#[derive(Clone)]
pub struct ResaleCompilerService {
    mongo: mongodb::Client,
}

impl ResaleCompilerService {
    pub fn new(mongo: mongodb::Client) -> Self {
        Self { mongo }
    }

    pub async fn simulate(
        &self,
        request: ResaleSimulationRequest,
    ) -> Result<ResaleSimulationResponse, ApiError> {
        let goals = normalized_goals(request.goals.clone().unwrap_or(SimulationGoals {
            liquidity_weight: 0.5,
            fairness_weight: 0.3,
            royalty_weight: 0.2,
        }));

        let inputs = crud::load_history_inputs(
            &self.mongo,
            &request.organizer_id,
            &request.event_id,
            request.class_id.as_deref(),
        )
        .await?;

        let candidates = if request.candidates.is_empty() {
            default_candidates()
        } else {
            request.candidates
        };

        let historical_fill = if inputs.listed_count == 0 {
            0.45
        } else {
            (inputs.sold_count as f64 / inputs.listed_count as f64).clamp(0.05, 0.98)
        };

        let mut simulations = candidates
            .into_iter()
            .map(|candidate| simulate_candidate(candidate, &goals, historical_fill))
            .collect::<Vec<_>>();

        simulations.sort_by(|a, b| {
            b.objective_score
                .partial_cmp(&a.objective_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let recommendation = recommend_policy(&simulations);

        Ok(ResaleSimulationResponse {
            organizer_id: request.organizer_id,
            event_id: request.event_id,
            class_id: request.class_id,
            goals,
            inputs,
            simulations,
            recommendation,
        })
    }
}

fn simulate_candidate(
    candidate: PolicyCandidate,
    goals: &SimulationGoals,
    historical_fill: f64,
) -> PolicySimulation {
    let validation_errors = validate_candidate(&candidate);
    let valid = validation_errors.is_empty();
    if !valid {
        return PolicySimulation {
            candidate_id: candidate.candidate_id,
            valid: false,
            validation_errors,
            expected_fill_rate: 0.0,
            expected_markup_bps: candidate.max_markup_bps,
            expected_royalty_yield: 0.0,
            liquidity_score: 0.0,
            fairness_score: 0.0,
            royalty_score: 0.0,
            objective_score: 0.0,
        };
    }

    let markup_penalty = (candidate.max_markup_bps as f64 / 11_000.0).clamp(0.0, 0.9);
    let whitelist_penalty = if candidate.whitelist_enabled {
        0.08
    } else {
        0.0
    };
    let blacklist_penalty = if candidate.blacklist_enabled {
        0.04
    } else {
        0.0
    };
    let expected_fill_rate = (historical_fill
        * (1.0 - markup_penalty)
        * (1.0 - whitelist_penalty)
        * (1.0 - blacklist_penalty))
        .clamp(0.05, 0.98);

    let liquidity_score = (expected_fill_rate * 100.0).clamp(0.0, 100.0);
    let fairness_score = ((1.0 - candidate.max_markup_bps as f64 / 10_000.0) * 80.0
        + if candidate.whitelist_enabled {
            8.0
        } else {
            14.0
        }
        - if candidate.blacklist_enabled {
            4.0
        } else {
            0.0
        })
    .clamp(0.0, 100.0);
    let royalty_score = (candidate.royalty_bps as f64 / 10_000.0 * 100.0).clamp(0.0, 100.0);

    let objective_score = (goals.liquidity_weight * liquidity_score)
        + (goals.fairness_weight * fairness_score)
        + (goals.royalty_weight * royalty_score);

    PolicySimulation {
        candidate_id: candidate.candidate_id,
        valid: true,
        validation_errors: vec![],
        expected_fill_rate,
        expected_markup_bps: candidate.max_markup_bps,
        expected_royalty_yield: candidate.royalty_bps as f64 * expected_fill_rate,
        liquidity_score,
        fairness_score,
        royalty_score,
        objective_score: objective_score.clamp(0.0, 100.0),
    }
}

fn recommend_policy(simulations: &[PolicySimulation]) -> Option<RecommendedPolicy> {
    let valid = simulations.iter().filter(|s| s.valid).collect::<Vec<_>>();
    if valid.is_empty() {
        return None;
    }

    let best = valid[0];
    let second_score = valid
        .get(1)
        .map(|s| s.objective_score)
        .unwrap_or(best.objective_score * 0.85);
    let gap = (best.objective_score - second_score).max(0.0);
    let confidence =
        (0.55 + (gap / 100.0) * 0.35 + (best.expected_fill_rate * 0.1)).clamp(0.0, 0.98);

    Some(RecommendedPolicy {
        candidate_id: best.candidate_id.clone(),
        confidence,
        rationale: format!(
            "best objective {:.2} with fill {:.2} and fairness {:.2}",
            best.objective_score, best.expected_fill_rate, best.fairness_score
        ),
    })
}

fn validate_candidate(candidate: &PolicyCandidate) -> Vec<String> {
    let mut errors = Vec::new();
    if candidate.max_markup_bps > 10_000 {
        errors.push("max_markup_bps must be <= 10000".to_string());
    }
    if candidate.royalty_bps > 10_000 {
        errors.push("royalty_bps must be <= 10000".to_string());
    }
    if candidate.max_markup_bps < candidate.royalty_bps {
        errors.push("max_markup_bps should be >= royalty_bps".to_string());
    }
    if candidate.whitelist_enabled && candidate.blacklist_enabled {
        errors.push("whitelist and blacklist cannot both be enabled".to_string());
    }
    errors
}

fn normalized_goals(goals: SimulationGoals) -> SimulationGoals {
    let liquidity = goals.liquidity_weight.max(0.0);
    let fairness = goals.fairness_weight.max(0.0);
    let royalty = goals.royalty_weight.max(0.0);
    let sum = liquidity + fairness + royalty;

    if sum <= f64::EPSILON {
        return SimulationGoals {
            liquidity_weight: 0.5,
            fairness_weight: 0.3,
            royalty_weight: 0.2,
        };
    }

    SimulationGoals {
        liquidity_weight: liquidity / sum,
        fairness_weight: fairness / sum,
        royalty_weight: royalty / sum,
    }
}

fn default_candidates() -> Vec<PolicyCandidate> {
    vec![
        PolicyCandidate {
            candidate_id: "balanced".to_string(),
            max_markup_bps: 2500,
            royalty_bps: 800,
            whitelist_enabled: false,
            blacklist_enabled: true,
        },
        PolicyCandidate {
            candidate_id: "liquidity_first".to_string(),
            max_markup_bps: 1800,
            royalty_bps: 500,
            whitelist_enabled: false,
            blacklist_enabled: false,
        },
        PolicyCandidate {
            candidate_id: "yield_first".to_string(),
            max_markup_bps: 3200,
            royalty_bps: 1100,
            whitelist_enabled: true,
            blacklist_enabled: false,
        },
    ]
}
