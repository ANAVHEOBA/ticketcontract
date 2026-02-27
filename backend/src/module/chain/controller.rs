use axum::{Json, extract::State};

use crate::{
    app::AppState,
    error::AppResult,
    module::{
        auth::guard::{AuthContext, require_any_role},
        chain::schema::{
            ChainContextResponse, ConfirmSignatureRequest, ConfirmSignatureResponse,
            DerivePdaRequest, DerivePdaResponse, SimulateTransactionRequest,
            SimulateTransactionResponse, SubmitAndConfirmRequest, SubmitAndConfirmResponse,
            SubmitTransactionRequest, SubmitTransactionResponse,
        },
    },
};

use crate::module::auth::model::Role;

pub async fn context(State(state): State<AppState>) -> AppResult<Json<ChainContextResponse>> {
    let details = state.chain_service.context();
    Ok(Json(details))
}

pub async fn derive_pda(
    State(state): State<AppState>,
    Json(payload): Json<DerivePdaRequest>,
) -> AppResult<Json<DerivePdaResponse>> {
    let result = state.chain_service.derive_pda(payload)?;
    Ok(Json(result))
}

pub async fn simulate_transaction(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SimulateTransactionRequest>,
) -> AppResult<Json<SimulateTransactionResponse>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    let result = state.chain_service.simulate_transaction(payload).await?;
    Ok(Json(result))
}

pub async fn submit_transaction(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SubmitTransactionRequest>,
) -> AppResult<Json<SubmitTransactionResponse>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    let result = state.chain_service.submit_transaction(payload).await?;
    Ok(Json(result))
}

pub async fn confirm_signature(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<ConfirmSignatureRequest>,
) -> AppResult<Json<ConfirmSignatureResponse>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
            Role::Scanner,
        ],
    )?;

    let result = state.chain_service.confirm_signature(payload).await?;
    Ok(Json(result))
}

pub async fn submit_and_confirm(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<SubmitAndConfirmRequest>,
) -> AppResult<Json<SubmitAndConfirmResponse>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    let result = state.chain_service.submit_and_confirm(payload).await?;
    Ok(Json(result))
}
