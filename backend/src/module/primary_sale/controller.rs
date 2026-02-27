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
        primary_sale::schema::{
            PrimarySaleActionResponse, PrimarySaleSimRequest, PrimarySaleSimResponse,
            PrimarySaleTxRequest, PurchaseReceipt,
        },
    },
};

macro_rules! primary_sale_handlers {
    ($(($tx_fn:ident, $sim_fn:ident, $action:literal)),* $(,)?) => {
        $(
            pub async fn $tx_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<PrimarySaleTxRequest>,
            ) -> AppResult<Json<PrimarySaleActionResponse>> {
                require_primary_sale_editor(&auth, &payload.organizer_id)?;

                let result = state.chain_service.submit_and_confirm_program_ix(SubmitAndConfirmRequest {
                    transaction_base64: payload.transaction_base64,
                    skip_preflight: payload.skip_preflight,
                    max_retries: payload.max_retries,
                    timeout_ms: payload.timeout_ms,
                    poll_ms: payload.poll_ms,
                }, &[$action]).await?;

                Ok(Json(PrimarySaleActionResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    class_id: payload.class_id,
                    confirmation_status: result.confirmation_status,
                    receipt: PurchaseReceipt {
                        signature: result.signature,
                        ticket_pda: payload.ticket_pda,
                        buyer_wallet: payload.buyer_wallet,
                        gross_amount: payload.gross_amount,
                        protocol_fee_amount: payload.protocol_fee_amount,
                        net_amount: payload.net_amount,
                    },
                }))
            }

            pub async fn $sim_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<PrimarySaleSimRequest>,
            ) -> AppResult<Json<PrimarySaleSimResponse>> {
                require_primary_sale_editor(&auth, &payload.organizer_id)?;

                let result = state.chain_service.simulate_transaction_program_ix(SimulateTransactionRequest {
                    transaction_base64: payload.transaction_base64,
                    sig_verify: payload.sig_verify,
                    replace_recent_blockhash: payload.replace_recent_blockhash,
                }, &[$action]).await?;

                Ok(Json(PrimarySaleSimResponse {
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

primary_sale_handlers!(
    (buy_ticket, simulate_buy_ticket, "buy_ticket"),
    (
        issue_comp_ticket,
        simulate_issue_comp_ticket,
        "issue_comp_ticket"
    ),
);

fn require_primary_sale_editor(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}
