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
        secondary_sale::{
            crud,
            schema::{
                ListingReadResponse, SecondarySaleActionResponse, SecondarySaleSimRequest,
                SecondarySaleSimResponse, SecondarySaleTxRequest,
            },
        },
    },
};

macro_rules! secondary_sale_handlers {
    ($(($tx_fn:ident, $sim_fn:ident, $action:literal, $role_guard:ident)),* $(,)?) => {
        $(
            pub async fn $tx_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<SecondarySaleTxRequest>,
            ) -> AppResult<Json<SecondarySaleActionResponse>> {
                $role_guard(&auth, &payload.organizer_id)?;
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

                Ok(Json(SecondarySaleActionResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    class_id: payload.class_id,
                    ticket_id: payload.ticket_id,
                    listing_id: payload.listing_id,
                    signature: result.signature,
                    confirmation_status: result.confirmation_status,
                }))
            }

            pub async fn $sim_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<SecondarySaleSimRequest>,
            ) -> AppResult<Json<SecondarySaleSimResponse>> {
                $role_guard(&auth, &payload.organizer_id)?;
                let result = state
                    .chain_service
                    .simulate_transaction_program_ix(SimulateTransactionRequest {
                        transaction_base64: payload.transaction_base64,
                        sig_verify: payload.sig_verify,
                        replace_recent_blockhash: payload.replace_recent_blockhash,
                    }, &[$action])
                    .await?;

                Ok(Json(SecondarySaleSimResponse {
                    action: $action,
                    organizer_id: payload.organizer_id,
                    event_id: payload.event_id,
                    class_id: payload.class_id,
                    ticket_id: payload.ticket_id,
                    listing_id: payload.listing_id,
                    err: result.err,
                    logs: result.logs,
                    units_consumed: result.units_consumed,
                }))
            }
        )*
    };
}

secondary_sale_handlers!(
    (
        list_ticket,
        simulate_list_ticket,
        "list_ticket",
        require_seller_or_admin
    ),
    (
        buy_resale_ticket,
        simulate_buy_resale_ticket,
        "buy_resale_ticket",
        require_buyer_or_admin
    ),
    (
        cancel_listing,
        simulate_cancel_listing,
        "cancel_listing",
        require_seller_or_admin
    ),
    (
        expire_listing,
        simulate_expire_listing,
        "expire_listing",
        require_operator_or_admin
    ),
);

pub async fn get_listing(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(listing_id): Path<String>,
) -> AppResult<Json<ListingReadResponse>> {
    let listing = crud::find_listing(&state.mongo, &listing_id).await?;
    match listing {
        Some(record) => {
            require_listing_reader(&auth, &record.organizer_id)?;
            Ok(Json(ListingReadResponse { listing: record }))
        }
        None => Err(ApiError::NotFound),
    }
}

fn require_listing_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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

fn require_seller_or_admin(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_buyer_or_admin(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
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

fn require_operator_or_admin(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}
