use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/disputes/refund", post(controller::refund_ticket))
        .route("/disputes/flag", post(controller::flag_dispute))
        .route("/disputes/chargeback", post(controller::flag_chargeback))
        .route("/disputes/queue", get(controller::query_dispute_queue))
}
