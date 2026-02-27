use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(controller::health))
        .route("/ready", get(controller::readiness))
}
