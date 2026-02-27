use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type AppResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("chain rpc unavailable: {0}")]
    ChainRpcUnavailable(String),
    #[error("database unavailable")]
    DatabaseUnavailable,
    #[error("internal server error")]
    Internal,
}

impl ApiError {
    pub fn map_chain_error<E>(err: E) -> Self
    where
        E: std::fmt::Display,
    {
        Self::ChainRpcUnavailable(err.to_string())
    }

    pub fn map_db_error<E>(_err: E) -> Self
    where
        E: std::fmt::Display,
    {
        Self::DatabaseUnavailable
    }
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "unauthorized".to_string(),
            ),
            Self::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "forbidden".to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "not found".to_string()),
            Self::ChainRpcUnavailable(msg) => (
                StatusCode::BAD_GATEWAY,
                "CHAIN_RPC_UNAVAILABLE",
                if msg.is_empty() {
                    "chain rpc unavailable".to_string()
                } else {
                    format!("chain rpc unavailable: {msg}")
                },
            ),
            Self::DatabaseUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "DATABASE_UNAVAILABLE",
                "database unavailable".to_string(),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "internal server error".to_string(),
            ),
        };

        (status, Json(ErrorBody { code, message })).into_response()
    }
}
