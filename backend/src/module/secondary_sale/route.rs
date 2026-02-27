use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/secondary-sale/list", post(controller::list_ticket))
        .route(
            "/secondary-sale/list/simulate",
            post(controller::simulate_list_ticket),
        )
        .route("/secondary-sale/buy", post(controller::buy_resale_ticket))
        .route(
            "/secondary-sale/buy/simulate",
            post(controller::simulate_buy_resale_ticket),
        )
        .route("/secondary-sale/cancel", post(controller::cancel_listing))
        .route(
            "/secondary-sale/cancel/simulate",
            post(controller::simulate_cancel_listing),
        )
        .route("/secondary-sale/expire", post(controller::expire_listing))
        .route(
            "/secondary-sale/expire/simulate",
            post(controller::simulate_expire_listing),
        )
        .route(
            "/secondary-sale/listings/{listing_id}",
            get(controller::get_listing),
        )
}
