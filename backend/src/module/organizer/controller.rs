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
        organizer::{
            crud,
            schema::{
                OrganizerActionResponse, OrganizerReadResponse, OrganizerSimRequest,
                OrganizerSimResponse, OrganizerTxRequest,
            },
        },
    },
};

macro_rules! organizer_action_handlers {
    ($(($tx_fn:ident, $sim_fn:ident, $action:literal)),* $(,)?) => {
        $(
            pub async fn $tx_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<OrganizerTxRequest>,
            ) -> AppResult<Json<OrganizerActionResponse>> {
                require_organizer_editor(&auth, &payload.organizer_id)?;
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

                Ok(Json(OrganizerActionResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    signature: result.signature,
                    confirmation_status: result.confirmation_status,
                }))
            }

            pub async fn $sim_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<OrganizerSimRequest>,
            ) -> AppResult<Json<OrganizerSimResponse>> {
                require_organizer_editor(&auth, &payload.organizer_id)?;
                let result = state
                    .chain_service
                    .simulate_transaction_program_ix(SimulateTransactionRequest {
                        transaction_base64: payload.transaction_base64,
                        sig_verify: payload.sig_verify,
                        replace_recent_blockhash: payload.replace_recent_blockhash,
                    }, &[$action])
                    .await?;

                Ok(Json(OrganizerSimResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    err: result.err,
                    logs: result.logs,
                    units_consumed: result.units_consumed,
                }))
            }
        )*
    };
}

organizer_action_handlers!(
    (
        create_organizer,
        simulate_create_organizer,
        "create_organizer"
    ),
    (
        update_organizer,
        simulate_update_organizer,
        "update_organizer"
    ),
    (
        set_organizer_status,
        simulate_set_organizer_status,
        "set_organizer_status"
    ),
    (
        set_organizer_compliance_flags,
        simulate_set_organizer_compliance_flags,
        "set_organizer_compliance_flags"
    ),
    (
        set_organizer_operator,
        simulate_set_organizer_operator,
        "set_organizer_operator"
    ),
);

pub async fn get_organizer(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(organizer_id): Path<String>,
) -> AppResult<Json<OrganizerReadResponse>> {
    require_organizer_reader(&auth, &organizer_id)?;
    let organizer = crud::find_organizer(&state.mongo, &organizer_id).await?;

    match organizer {
        Some(record) => Ok(Json(OrganizerReadResponse { organizer: record })),
        None => Err(ApiError::NotFound),
    }
}

fn require_organizer_editor(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(auth, &[Role::ProtocolAdmin, Role::OrganizerAdmin])?;
    require_organizer_scope(auth, organizer_id)
}

fn require_organizer_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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
