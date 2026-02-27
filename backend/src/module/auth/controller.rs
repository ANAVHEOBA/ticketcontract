use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    app::AppState,
    error::{ApiError, AppResult},
};

use super::{
    guard::{AuthContext, require_any_role, require_organizer_scope},
    model::Role,
    schema::{
        MeResponse, NonceRequest, NonceResponse, OrganizerAccessResponse, ProviderVerifyRequest,
        VerifyRequest,
        VerifyResponse,
    },
};

pub async fn issue_nonce(
    State(state): State<AppState>,
    Json(payload): Json<NonceRequest>,
) -> AppResult<Json<NonceResponse>> {
    let response = state.auth_service.issue_nonce(payload.wallet).await?;
    Ok(Json(response))
}

pub async fn verify_signature(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> AppResult<Json<VerifyResponse>> {
    let response = state.auth_service.verify_and_issue_token(payload).await?;
    Ok(Json(response))
}

pub async fn verify_provider(
    State(state): State<AppState>,
    Json(payload): Json<ProviderVerifyRequest>,
) -> AppResult<Json<VerifyResponse>> {
    let response = state
        .auth_service
        .verify_provider_and_issue_token(payload)
        .await?;
    Ok(Json(response))
}

pub async fn me(auth: AuthContext) -> AppResult<Json<MeResponse>> {
    Ok(Json(MeResponse {
        wallet: auth.wallet,
        role: auth.role,
        organizer_scopes: auth.organizer_scopes,
    }))
}

pub async fn organizer_access(
    auth: AuthContext,
    Path(organizer_id): Path<String>,
) -> AppResult<Json<OrganizerAccessResponse>> {
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
    require_organizer_scope(&auth, &organizer_id)?;

    if auth.role == Role::Scanner
        || auth.role == Role::Operator
        || auth.role == Role::OrganizerAdmin
    {
        return Ok(Json(OrganizerAccessResponse {
            allowed: true,
            organizer_id,
            role: auth.role,
        }));
    }

    if auth.role == Role::ProtocolAdmin || auth.role == Role::Financier {
        return Ok(Json(OrganizerAccessResponse {
            allowed: true,
            organizer_id,
            role: auth.role,
        }));
    }

    Err(ApiError::Forbidden)
}
