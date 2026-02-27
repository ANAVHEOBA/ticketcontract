use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app::AppState,
    error::{ApiError, AppResult},
    module::{
        auth::{
            guard::{AuthContext, require_any_role, require_organizer_scope},
            model::Role,
        },
        chain::schema::{SimulateTransactionRequest, SubmitAndConfirmRequest},
        resale_policy::{
            crud,
            model::ResalePolicyRecommendation,
            schema::{
                PolicyValidationRequest, PolicyValidationResponse, RecommendationWriteRequest,
                RecommendationWriteResponse, ResalePolicyActionResponse, ResalePolicyReadResponse,
                ResalePolicySimRequest, ResalePolicySimResponse, ResalePolicyTxRequest,
            },
        },
    },
};

#[derive(serde::Deserialize)]
pub struct PolicyQuery {
    pub event_id: String,
    pub class_id: Option<String>,
}

pub async fn set_policy(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<ResalePolicyTxRequest>,
) -> AppResult<Json<ResalePolicyActionResponse>> {
    require_policy_editor(&auth, &payload.organizer_id)?;
    let result = state
        .chain_service
        .submit_and_confirm_program_ix(
            SubmitAndConfirmRequest {
                transaction_base64: payload.transaction_base64,
                skip_preflight: payload.skip_preflight,
                max_retries: payload.max_retries,
                timeout_ms: payload.timeout_ms,
                poll_ms: payload.poll_ms,
            },
            &["set_resale_policy"],
        )
        .await?;

    Ok(Json(ResalePolicyActionResponse {
        action: "set_resale_policy",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        class_id: payload.class_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}

pub async fn simulate_set_policy(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<ResalePolicySimRequest>,
) -> AppResult<Json<ResalePolicySimResponse>> {
    require_policy_editor(&auth, &payload.organizer_id)?;
    let result = state
        .chain_service
        .simulate_transaction_program_ix(
            SimulateTransactionRequest {
                transaction_base64: payload.transaction_base64,
                sig_verify: payload.sig_verify,
                replace_recent_blockhash: payload.replace_recent_blockhash,
            },
            &["set_resale_policy"],
        )
        .await?;

    Ok(Json(ResalePolicySimResponse {
        action: "set_resale_policy",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        class_id: payload.class_id,
        err: result.err,
        logs: result.logs,
        units_consumed: result.units_consumed,
    }))
}

pub async fn get_policy(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<PolicyQuery>,
) -> AppResult<Json<ResalePolicyReadResponse>> {
    let policy =
        crud::find_policy(&state.mongo, &query.event_id, query.class_id.as_deref()).await?;
    match policy {
        Some(record) => {
            require_policy_reader(&auth, &record.organizer_id)?;
            Ok(Json(ResalePolicyReadResponse { policy: record }))
        }
        None => Err(ApiError::NotFound),
    }
}

pub async fn write_recommendation(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<RecommendationWriteRequest>,
) -> AppResult<Json<RecommendationWriteResponse>> {
    require_policy_editor(&auth, &payload.organizer_id)?;

    let validation = validate_policy(payload.max_markup_bps, payload.royalty_bps, false, false);
    if !validation.valid {
        return Err(ApiError::BadRequest(format!(
            "invalid recommendation: {}",
            validation.reasons.join(", ")
        )));
    }

    let recommendation = ResalePolicyRecommendation {
        recommendation_id: payload.recommendation_id,
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        class_id: payload.class_id,
        max_markup_bps: payload.max_markup_bps,
        royalty_bps: payload.royalty_bps,
        confidence: payload.confidence,
        rationale: payload.rationale,
        updated_at_epoch: epoch_now(),
    };

    crud::upsert_recommendation(&state.mongo, &recommendation).await?;

    Ok(Json(RecommendationWriteResponse {
        saved: true,
        recommendation,
    }))
}

pub async fn validate_policy_request(
    auth: AuthContext,
    Json(payload): Json<PolicyValidationRequest>,
) -> AppResult<Json<PolicyValidationResponse>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    Ok(Json(validate_policy(
        payload.max_markup_bps,
        payload.royalty_bps,
        payload.whitelist_enabled,
        payload.blacklist_enabled,
    )))
}

fn require_policy_editor(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_policy_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Scanner,
            Role::Financier,
        ],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn validate_policy(
    max_markup_bps: u16,
    royalty_bps: u16,
    whitelist_enabled: bool,
    blacklist_enabled: bool,
) -> PolicyValidationResponse {
    let mut reasons = Vec::new();

    if max_markup_bps > 10_000 {
        reasons.push("max_markup_bps must be <= 10000".to_string());
    }

    if royalty_bps > 10_000 {
        reasons.push("royalty_bps must be <= 10000".to_string());
    }

    if max_markup_bps < royalty_bps {
        reasons.push("max_markup_bps should be >= royalty_bps".to_string());
    }

    if whitelist_enabled && blacklist_enabled {
        reasons.push("whitelist and blacklist cannot both be enabled".to_string());
    }

    PolicyValidationResponse {
        valid: reasons.is_empty(),
        reasons,
    }
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
