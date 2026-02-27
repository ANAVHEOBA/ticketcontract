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
        financing::{
            crud,
            schema::{
                FinancingActionResponse, FinancingReadResponse, FinancingSimRequest,
                FinancingSimResponse, FinancingTxRequest,
            },
        },
    },
};

macro_rules! financing_handlers {
    ($(($tx_fn:ident, $sim_fn:ident, $action:literal, $guard:ident)),* $(,)?) => {
        $(
            pub async fn $tx_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<FinancingTxRequest>,
            ) -> AppResult<Json<FinancingActionResponse>> {
                $guard(&auth, &payload.organizer_id)?;
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

                Ok(Json(FinancingActionResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    offer_id: payload.offer_id,
                    signature: result.signature,
                    confirmation_status: result.confirmation_status,
                }))
            }

            pub async fn $sim_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<FinancingSimRequest>,
            ) -> AppResult<Json<FinancingSimResponse>> {
                $guard(&auth, &payload.organizer_id)?;
                let result = state
                    .chain_service
                    .simulate_transaction_program_ix(SimulateTransactionRequest {
                        transaction_base64: payload.transaction_base64,
                        sig_verify: payload.sig_verify,
                        replace_recent_blockhash: payload.replace_recent_blockhash,
                    }, &[$action])
                    .await?;

                Ok(Json(FinancingSimResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    offer_id: payload.offer_id,
                    err: result.err,
                    logs: result.logs,
                    units_consumed: result.units_consumed,
                }))
            }
        )*
    };
}

financing_handlers!(
    (
        create_offer,
        simulate_create_offer,
        "create_financing_offer",
        require_financier_or_admin
    ),
    (
        accept_offer,
        simulate_accept_offer,
        "accept_financing_offer",
        require_organizer_or_admin
    ),
    (
        reject_offer,
        simulate_reject_offer,
        "reject_financing_offer",
        require_organizer_or_admin
    ),
    (
        disburse_advance,
        simulate_disburse_advance,
        "disburse_advance",
        require_financier_or_admin
    ),
    (
        clawback_disbursement,
        simulate_clawback_disbursement,
        "clawback_disbursement",
        require_financier_or_admin
    ),
    (
        set_financing_freeze,
        simulate_set_financing_freeze,
        "set_financing_freeze",
        require_financier_or_admin
    ),
);

pub async fn get_offer(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(offer_id): Path<String>,
) -> AppResult<Json<FinancingReadResponse>> {
    let offer = crud::find_offer(&state.mongo, &offer_id).await?;
    match offer {
        Some(record) => {
            require_financing_reader(&auth, &record.organizer_id)?;
            Ok(Json(FinancingReadResponse { offer: record }))
        }
        None => Err(ApiError::NotFound),
    }
}

fn require_financing_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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

fn require_financier_or_admin(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(auth, &[Role::ProtocolAdmin, Role::Financier])?;
    require_organizer_scope(auth, organizer_id)
}

fn require_organizer_or_admin(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}
