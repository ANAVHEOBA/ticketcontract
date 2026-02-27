use axum::{Router, routing::post};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/checkin/policy", post(controller::set_checkin_policy))
        .route("/checkin/ticket", post(controller::check_in_ticket))
        .route(
            "/checkin/ticket/simulate",
            post(controller::simulate_checkin),
        )
}
