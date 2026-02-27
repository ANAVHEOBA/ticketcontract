use axum::{
    Json,
    extract::{Path, State},
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
        ticket_state::{
            crud,
            schema::{
                TicketReadResponse, TicketStateActionResponse, TicketStateSimRequest,
                TicketStateSimResponse, TicketStateTxRequest, TicketTransitionTxRequest,
            },
        },
    },
};

pub async fn get_ticket(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
) -> AppResult<Json<TicketReadResponse>> {
    let ticket = crud::find_ticket(&state.mongo, &ticket_id).await?;
    match ticket {
        Some(record) => {
            require_ticket_reader(&auth, &record.organizer_id)?;
            Ok(Json(TicketReadResponse { ticket: record }))
        }
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_ticket_metadata(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<TicketStateTxRequest>,
) -> AppResult<Json<TicketStateActionResponse>> {
    require_ticket_admin_or_operator(&auth, &payload.organizer_id)?;
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
            &["set_ticket_metadata"],
        )
        .await?;

    Ok(Json(TicketStateActionResponse {
        action: "update_ticket_metadata",
        organizer_id: payload.organizer_id,
        ticket_id: payload.ticket_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}

pub async fn simulate_update_ticket_metadata(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<TicketStateSimRequest>,
) -> AppResult<Json<TicketStateSimResponse>> {
    require_ticket_admin_or_operator(&auth, &payload.organizer_id)?;
    let result = state
        .chain_service
        .simulate_transaction_program_ix(
            SimulateTransactionRequest {
                transaction_base64: payload.transaction_base64,
                sig_verify: payload.sig_verify,
                replace_recent_blockhash: payload.replace_recent_blockhash,
            },
            &["set_ticket_metadata"],
        )
        .await?;

    Ok(Json(TicketStateSimResponse {
        action: "update_ticket_metadata",
        organizer_id: payload.organizer_id,
        ticket_id: payload.ticket_id,
        err: result.err,
        logs: result.logs,
        units_consumed: result.units_consumed,
    }))
}

pub async fn transition_ticket_status(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<TicketTransitionTxRequest>,
) -> AppResult<Json<TicketStateActionResponse>> {
    validate_transition_permissions(&auth, &payload.organizer_id, &payload.target_status)?;
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
            &["transition_ticket_status"],
        )
        .await?;

    Ok(Json(TicketStateActionResponse {
        action: "transition_ticket_status",
        organizer_id: payload.organizer_id,
        ticket_id: payload.ticket_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}

fn require_ticket_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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

fn require_ticket_admin_or_operator(
    auth: &AuthContext,
    organizer_id: &str,
) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn validate_transition_permissions(
    auth: &AuthContext,
    organizer_id: &str,
    target_status: &str,
) -> Result<(), ApiError> {
    require_organizer_scope(auth, organizer_id)?;
    let target = target_status.to_lowercase();

    match auth.role {
        Role::ProtocolAdmin | Role::OrganizerAdmin => Ok(()),
        Role::Operator => {
            if matches!(target.as_str(), "checked_in" | "transferred" | "active") {
                Ok(())
            } else {
                Err(ApiError::Forbidden)
            }
        }
        _ => Err(ApiError::Forbidden),
    }
}
