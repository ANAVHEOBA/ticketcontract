use axum::{Router, routing::post};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/primary-sale/buy", post(controller::buy_ticket))
        .route(
            "/primary-sale/buy/simulate",
            post(controller::simulate_buy_ticket),
        )
        .route("/primary-sale/comp", post(controller::issue_comp_ticket))
        .route(
            "/primary-sale/comp/simulate",
            post(controller::simulate_issue_comp_ticket),
        )
}
