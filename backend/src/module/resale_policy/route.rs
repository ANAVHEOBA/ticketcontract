use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/resale-policy",
            get(controller::get_policy).post(controller::set_policy),
        )
        .route(
            "/resale-policy/simulate",
            post(controller::simulate_set_policy),
        )
        .route(
            "/resale-policy/recommendation",
            post(controller::write_recommendation),
        )
        .route(
            "/resale-policy/validate",
            post(controller::validate_policy_request),
        )
}
