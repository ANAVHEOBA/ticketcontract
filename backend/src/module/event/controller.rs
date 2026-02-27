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
        event::{
            crud,
            schema::{
                EventActionResponse, EventListQuery, EventListResponse, EventReadResponse,
                EventSimRequest, EventSimResponse, EventTxRequest,
            },
        },
    },
};

macro_rules! event_action_handlers {
    ($(($tx_fn:ident, $sim_fn:ident, $action:literal)),* $(,)?) => {
        $(
            pub async fn $tx_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<EventTxRequest>,
            ) -> AppResult<Json<EventActionResponse>> {
                require_event_editor(&auth, &payload.organizer_id)?;
                let result = state
                    .chain_service
                    .submit_and_confirm_program_ix(SubmitAndConfirmRequest {
                        transaction_base64: payload.transaction_base64,
                        skip_preflight: payload.skip_preflight,
                        max_retries: payload.max_retries,
                        timeout_ms: payload.timeout_ms,
                        poll_ms: payload.poll_ms,
                    }, &[$action])
                    .await?;

                Ok(Json(EventActionResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    signature: result.signature,
                    confirmation_status: result.confirmation_status,
                }))
            }

            pub async fn $sim_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<EventSimRequest>,
            ) -> AppResult<Json<EventSimResponse>> {
                require_event_editor(&auth, &payload.organizer_id)?;
                let result = state
                    .chain_service
                    .simulate_transaction_program_ix(SimulateTransactionRequest {
                        transaction_base64: payload.transaction_base64,
                        sig_verify: payload.sig_verify,
                        replace_recent_blockhash: payload.replace_recent_blockhash,
                    }, &[$action])
                    .await?;

                Ok(Json(EventSimResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    err: result.err,
                    logs: result.logs,
                    units_consumed: result.units_consumed,
                }))
            }
        )*
    };
}

event_action_handlers!(
    (create_event, simulate_create_event, "create_event"),
    (update_event, simulate_update_event, "update_event"),
    (freeze_event, simulate_freeze_event, "freeze_event"),
    (cancel_event, simulate_cancel_event, "cancel_event"),
    (pause_event, simulate_pause_event, "pause_event"),
    (close_event, simulate_close_event, "close_event"),
    (
        set_event_restrictions,
        simulate_set_event_restrictions,
        "set_event_restrictions"
    ),
    (
        set_event_loyalty_multiplier,
        simulate_set_event_loyalty_multiplier,
        "set_event_loyalty_multiplier"
    ),
);

pub async fn get_event(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> AppResult<Json<EventReadResponse>> {
    let event = crud::find_event(&state.mongo, &event_id).await?;
    match event {
        Some(record) => {
            require_event_reader(&auth, &record.organizer_id)?;
            Ok(Json(EventReadResponse { event: record }))
        }
        None => Err(ApiError::NotFound),
    }
}

pub async fn list_events(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<EventListQuery>,
) -> AppResult<Json<EventListResponse>> {
    if let Some(organizer_id) = query.organizer_id.as_deref() {
        require_event_reader(&auth, organizer_id)?;
    } else {
        require_any_role(
            &auth,
            &[
                Role::ProtocolAdmin,
                Role::Financier,
                Role::OrganizerAdmin,
                Role::Operator,
                Role::Scanner,
            ],
        )?;
    }

    let events = crud::list_events(
        &state.mongo,
        query.organizer_id.as_deref(),
        query.status.as_deref(),
    )
    .await?;

    Ok(Json(EventListResponse { events }))
}

fn require_event_editor(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_event_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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
