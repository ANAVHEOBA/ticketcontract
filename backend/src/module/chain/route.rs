use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/chain/context", get(controller::context))
        .route("/chain/pda/derive", post(controller::derive_pda))
        .route("/chain/tx/simulate", post(controller::simulate_transaction))
        .route("/chain/tx/submit", post(controller::submit_transaction))
        .route("/chain/tx/confirm", post(controller::confirm_signature))
        .route(
            "/chain/tx/submit-and-confirm",
            post(controller::submit_and_confirm),
        )
}
