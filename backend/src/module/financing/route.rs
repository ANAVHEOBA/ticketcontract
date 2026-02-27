use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/financing/offers", post(controller::create_offer))
        .route(
            "/financing/offers/simulate",
            post(controller::simulate_create_offer),
        )
        .route("/financing/offers/accept", post(controller::accept_offer))
        .route(
            "/financing/offers/accept/simulate",
            post(controller::simulate_accept_offer),
        )
        .route("/financing/offers/reject", post(controller::reject_offer))
        .route(
            "/financing/offers/reject/simulate",
            post(controller::simulate_reject_offer),
        )
        .route("/financing/disburse", post(controller::disburse_advance))
        .route(
            "/financing/disburse/simulate",
            post(controller::simulate_disburse_advance),
        )
        .route(
            "/financing/clawback",
            post(controller::clawback_disbursement),
        )
        .route(
            "/financing/clawback/simulate",
            post(controller::simulate_clawback_disbursement),
        )
        .route("/financing/freeze", post(controller::set_financing_freeze))
        .route(
            "/financing/freeze/simulate",
            post(controller::simulate_set_financing_freeze),
        )
        .route("/financing/offers/{offer_id}", get(controller::get_offer))
}
