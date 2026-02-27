use axum::{Router, routing::post};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/resale-compiler/simulate",
        post(controller::simulate_resale_policy),
    )
}
