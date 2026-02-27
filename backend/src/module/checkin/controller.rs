use std::time::{SystemTime, UNIX_EPOCH};

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
        checkin::schema::{
            CheckInActionResponse, CheckInPolicyActionResponse, CheckInPolicyTxRequest,
            CheckInSimRequest, CheckInSimResponse, CheckInTxRequest, GateCheckInPayload,
        },
    },
};

pub async fn set_checkin_policy(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CheckInPolicyTxRequest>,
) -> AppResult<Json<CheckInPolicyActionResponse>> {
    require_checkin_policy_writer(&auth, &payload.organizer_id)?;
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
            &["set_check_in_policy"],
        )
        .await?;

    Ok(Json(CheckInPolicyActionResponse {
        action: "set_checkin_policy",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        class_id: payload.class_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}

pub async fn check_in_ticket(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CheckInTxRequest>,
) -> AppResult<Json<CheckInActionResponse>> {
    require_scanner_or_admin(&auth, &payload.organizer_id)?;
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
            &["check_in_ticket"],
        )
        .await?;

    Ok(Json(CheckInActionResponse {
        action: "check_in_ticket",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        class_id: payload.class_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
        gate_payload: GateCheckInPayload {
            gate_id: payload.gate_id,
            ticket_id: payload.ticket_id,
            scanner_id: payload.scanner_id,
            accepted: true,
            reason: None,
            checked_in_at_epoch: epoch_now(),
        },
    }))
}

pub async fn simulate_checkin(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CheckInSimRequest>,
) -> AppResult<Json<CheckInSimResponse>> {
    require_scanner_or_admin(&auth, &payload.organizer_id)?;
    let result = state
        .chain_service
        .simulate_transaction_program_ix(
            SimulateTransactionRequest {
                transaction_base64: payload.transaction_base64,
                sig_verify: payload.sig_verify,
                replace_recent_blockhash: payload.replace_recent_blockhash,
            },
            &["check_in_ticket"],
        )
        .await?;

    Ok(Json(CheckInSimResponse {
        action: "check_in_ticket",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        class_id: payload.class_id,
        ticket_id: payload.ticket_id,
        gate_id: payload.gate_id,
        err: result.err,
        logs: result.logs,
        units_consumed: result.units_consumed,
    }))
}

fn require_checkin_policy_writer(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_scanner_or_admin(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Scanner,
        ],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
