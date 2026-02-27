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
        chain::schema::SubmitAndConfirmRequest,
        disputes::{
            crud,
            schema::{
                DisputeActionResponse, DisputeQueueQuery, DisputeQueueResponse, DisputeTxRequest,
            },
        },
    },
};

pub async fn refund_ticket(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<DisputeTxRequest>,
) -> AppResult<Json<DisputeActionResponse>> {
    require_refund_writer(&auth, &payload.organizer_id)?;
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
            &["refund_ticket"],
        )
        .await?;

    Ok(Json(DisputeActionResponse {
        action: "refund_ticket",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        ticket_id: payload.ticket_id,
        dispute_id: payload.dispute_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}

pub async fn flag_dispute(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<DisputeTxRequest>,
) -> AppResult<Json<DisputeActionResponse>> {
    require_dispute_writer(&auth, &payload.organizer_id)?;
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
            &["flag_dispute"],
        )
        .await?;

    Ok(Json(DisputeActionResponse {
        action: "flag_dispute",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        ticket_id: payload.ticket_id,
        dispute_id: payload.dispute_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}

pub async fn flag_chargeback(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<DisputeTxRequest>,
) -> AppResult<Json<DisputeActionResponse>> {
    require_dispute_writer(&auth, &payload.organizer_id)?;
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
            &["flag_dispute"],
        )
        .await?;

    Ok(Json(DisputeActionResponse {
        action: "flag_chargeback",
        organizer_id: payload.organizer_id,
        event_id: payload.event_id,
        ticket_id: payload.ticket_id,
        dispute_id: payload.dispute_id,
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}

pub async fn query_dispute_queue(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<DisputeQueueQuery>,
) -> AppResult<Json<DisputeQueueResponse>> {
    if let Some(org) = query.organizer_id.as_deref() {
        require_dispute_reader(&auth, org)?;
    } else {
        require_any_role(
            &auth,
            &[
                Role::ProtocolAdmin,
                Role::OrganizerAdmin,
                Role::Operator,
                Role::Financier,
            ],
        )?;
    }

    let items = crud::list_disputes(
        &state.mongo,
        query.organizer_id.as_deref(),
        query.status.as_deref(),
        query.limit.unwrap_or(100),
    )
    .await?;

    Ok(Json(DisputeQueueResponse { items }))
}

fn require_refund_writer(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_dispute_writer(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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

fn require_dispute_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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
