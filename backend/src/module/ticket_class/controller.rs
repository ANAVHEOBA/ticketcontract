use axum::{
    Json,
    extract::{Path, Query, State},
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
        ticket_class::{
            crud,
            schema::{
                TicketClassActionResponse, TicketClassAnalyticsResponse, TicketClassListQuery,
                TicketClassListResponse, TicketClassReadResponse, TicketClassSimRequest,
                TicketClassSimResponse, TicketClassTxRequest,
            },
        },
    },
};

macro_rules! class_action_handlers {
    ($(($tx_fn:ident, $sim_fn:ident, $action:literal)),* $(,)?) => {
        $(
            pub async fn $tx_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<TicketClassTxRequest>,
            ) -> AppResult<Json<TicketClassActionResponse>> {
                require_class_editor(&auth, &payload.organizer_id)?;
                let result = state.chain_service.submit_and_confirm_program_ix(SubmitAndConfirmRequest {
                    transaction_base64: payload.transaction_base64,
                    skip_preflight: payload.skip_preflight,
                    max_retries: payload.max_retries,
                    timeout_ms: payload.timeout_ms,
                    poll_ms: payload.poll_ms,
                }, &[$action]).await?;

                Ok(Json(TicketClassActionResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    class_id: payload.class_id,
                    signature: result.signature,
                    confirmation_status: result.confirmation_status,
                }))
            }

            pub async fn $sim_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<TicketClassSimRequest>,
            ) -> AppResult<Json<TicketClassSimResponse>> {
                require_class_editor(&auth, &payload.organizer_id)?;
                let result = state.chain_service.simulate_transaction_program_ix(SimulateTransactionRequest {
                    transaction_base64: payload.transaction_base64,
                    sig_verify: payload.sig_verify,
                    replace_recent_blockhash: payload.replace_recent_blockhash,
                }, &[$action]).await?;

                Ok(Json(TicketClassSimResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    class_id: payload.class_id,
                    err: result.err,
                    logs: result.logs,
                    units_consumed: result.units_consumed,
                }))
            }
        )*
    };
}

class_action_handlers!(
    (
        create_ticket_class,
        simulate_create_ticket_class,
        "create_ticket_class"
    ),
    (
        update_ticket_class,
        simulate_update_ticket_class,
        "update_ticket_class"
    ),
    (
        reserve_inventory,
        simulate_reserve_inventory,
        "reserve_inventory"
    ),
);

pub async fn get_ticket_class(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(class_id): Path<String>,
) -> AppResult<Json<TicketClassReadResponse>> {
    let class = crud::find_ticket_class(&state.mongo, &class_id).await?;
    match class {
        Some(record) => {
            require_class_reader(&auth, &record.organizer_id)?;
            Ok(Json(TicketClassReadResponse { class: record }))
        }
        None => Err(ApiError::NotFound),
    }
}

pub async fn ticket_class_analytics(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(class_id): Path<String>,
) -> AppResult<Json<TicketClassAnalyticsResponse>> {
    let analytics = crud::class_analytics(&state.mongo, &class_id).await?;
    match analytics {
        Some(value) => {
            require_class_reader(&auth, &value.organizer_id)?;
            Ok(Json(TicketClassAnalyticsResponse { analytics: value }))
        }
        None => Err(ApiError::NotFound),
    }
}

pub async fn list_ticket_classes(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<TicketClassListQuery>,
) -> AppResult<Json<TicketClassListResponse>> {
    if let Some(org) = query.organizer_id.as_deref() {
        require_class_reader(&auth, org)?;
    } else {
        require_any_role(
            &auth,
            &[
                Role::ProtocolAdmin,
                Role::OrganizerAdmin,
                Role::Operator,
                Role::Scanner,
                Role::Financier,
            ],
        )?;
    }

    let classes = crud::list_ticket_classes(
        &state.mongo,
        query.organizer_id.as_deref(),
        query.event_id.as_deref(),
    )
    .await?;

    Ok(Json(TicketClassListResponse { classes }))
}

fn require_class_editor(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_class_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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
