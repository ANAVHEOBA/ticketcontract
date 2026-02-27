use axum::{Json, extract::State};

use crate::{
    app::AppState,
    error::{ApiError, AppResult},
    module::{
        auth::{
            guard::{AuthContext, require_any_role, require_organizer_scope},
            model::Role,
        },
        chain::schema::{SimulateTransactionRequest, SubmitAndConfirmRequest},
        settlement::schema::{
            SettlementActionResponse, SettlementSimRequest, SettlementSimResponse,
            SettlementTxRequest,
        },
    },
};

pub async fn settle_primary_revenue(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SettlementTxRequest>,
) -> AppResult<Json<SettlementActionResponse>> {
    execute_settlement_action(
        &auth,
        &state,
        payload,
        "settle_primary_revenue",
        require_settlement_writer,
    )
    .await
}

pub async fn settle_resale_revenue(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SettlementTxRequest>,
) -> AppResult<Json<SettlementActionResponse>> {
    execute_settlement_action(
        &auth,
        &state,
        payload,
        "settle_resale_revenue",
        require_settlement_writer,
    )
    .await
}

pub async fn finalize_settlement(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SettlementTxRequest>,
) -> AppResult<Json<SettlementActionResponse>> {
    execute_settlement_action(
        &auth,
        &state,
        payload,
        "finalize_settlement",
        require_settlement_admin,
    )
    .await
}

pub async fn simulate_settle_primary_revenue(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SettlementSimRequest>,
) -> AppResult<Json<SettlementSimResponse>> {
    execute_settlement_simulation(
        &auth,
        &state,
        payload,
        "settle_primary_revenue",
        require_settlement_writer,
    )
    .await
}

pub async fn simulate_settle_resale_revenue(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SettlementSimRequest>,
) -> AppResult<Json<SettlementSimResponse>> {
    execute_settlement_simulation(
        &auth,
        &state,
        payload,
        "settle_resale_revenue",
        require_settlement_writer,
    )
    .await
}

pub async fn simulate_finalize_settlement(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SettlementSimRequest>,
) -> AppResult<Json<SettlementSimResponse>> {
    execute_settlement_simulation(
        &auth,
        &state,
        payload,
        "finalize_settlement",
        require_settlement_admin,
    )
    .await
}

async fn execute_settlement_action(
    auth: &AuthContext,
    state: &AppState,
    payload: SettlementTxRequest,
    action: &'static str,
    guard: fn(&AuthContext, &str) -> Result<(), ApiError>,
) -> AppResult<Json<SettlementActionResponse>> {
    guard(auth, &payload.organizer_id)?;

    let idempotency_key = format!(
        "settlement:{action}:{}:{}:{}",
        payload.organizer_id, payload.event_id, payload.settlement_ref
    );

    if let Some(value) = state.idempotency_service.get(&idempotency_key).await {
        let mut response: SettlementActionResponse =
            serde_json::from_value(value).map_err(|_| ApiError::Internal)?;
        response.idempotent_replay = true;
        return Ok(Json(response));
    }

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
            &[action],
        )
        .await?;

    let response = SettlementActionResponse {
        action: action.to_string(),
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        settlement_ref: payload.settlement_ref,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
        idempotent_replay: false,
    };

    state
        .idempotency_service
        .put(
            idempotency_key,
            serde_json::to_value(&response).map_err(|_| ApiError::Internal)?,
        )
        .await;

    Ok(Json(response))
}

async fn execute_settlement_simulation(
    auth: &AuthContext,
    state: &AppState,
    payload: SettlementSimRequest,
    action: &'static str,
    guard: fn(&AuthContext, &str) -> Result<(), ApiError>,
) -> AppResult<Json<SettlementSimResponse>> {
    guard(auth, &payload.organizer_id)?;

    let result = state
        .chain_service
        .simulate_transaction_program_ix(
            SimulateTransactionRequest {
                transaction_base64: payload.transaction_base64,
                sig_verify: payload.sig_verify,
                replace_recent_blockhash: payload.replace_recent_blockhash,
            },
            &[action],
        )
        .await?;

    Ok(Json(SettlementSimResponse {
        action,
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        settlement_ref: payload.settlement_ref,
        err: result.err,
        logs: result.logs,
        units_consumed: result.units_consumed,
    }))
}

fn require_settlement_writer(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_settlement_admin(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Financier],
    )?;
    require_organizer_scope(auth, organizer_id)
}
