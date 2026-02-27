use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::app::AppState;
use crate::service::health_service::compute_readiness;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub env: String,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "ticketing-backend",
        env: state.config.app.env.clone(),
    })
}

pub async fn readiness(State(state): State<AppState>) -> Response {
    let readiness = compute_readiness(&state).await;
    let status = if readiness.ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(readiness)).into_response()
}
