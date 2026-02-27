use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{app::AppState, error::ApiError};

use super::model::{JwtClaims, Role};

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub wallet: String,
    pub role: Role,
    pub organizer_scopes: Vec<String>,
}

impl<S> FromRequestParts<S> for AuthContext
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        let token = header
            .strip_prefix("Bearer ")
            .or_else(|| header.strip_prefix("bearer "))
            .ok_or(ApiError::Unauthorized)?;

        let claims: JwtClaims = app_state.auth_service.decode_token(token)?;

        Ok(Self {
            wallet: claims.sub,
            role: claims.role,
            organizer_scopes: claims.organizer_scopes,
        })
    }
}

pub fn require_any_role(context: &AuthContext, allowed: &[Role]) -> Result<(), ApiError> {
    if allowed.iter().any(|role| role == &context.role) {
        return Ok(());
    }
    Err(ApiError::Forbidden)
}

pub fn require_organizer_scope(context: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    if context.role == Role::ProtocolAdmin {
        return Ok(());
    }

    if context
        .organizer_scopes
        .iter()
        .any(|scope| scope == organizer_id || scope == "*")
    {
        return Ok(());
    }

    Err(ApiError::Forbidden)
}
