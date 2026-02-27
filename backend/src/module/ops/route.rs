use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ops/metrics", get(controller::metrics))
        .route("/ops/alerts", get(controller::alerts))
        .route("/ops/audit-logs", get(controller::audit_logs))
}
