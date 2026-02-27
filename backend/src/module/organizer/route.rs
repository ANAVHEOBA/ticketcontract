use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/organizers", post(controller::create_organizer))
        .route(
            "/organizers/simulate",
            post(controller::simulate_create_organizer),
        )
        .route("/organizers/update", post(controller::update_organizer))
        .route(
            "/organizers/update/simulate",
            post(controller::simulate_update_organizer),
        )
        .route("/organizers/status", post(controller::set_organizer_status))
        .route(
            "/organizers/status/simulate",
            post(controller::simulate_set_organizer_status),
        )
        .route(
            "/organizers/compliance-flags",
            post(controller::set_organizer_compliance_flags),
        )
        .route(
            "/organizers/compliance-flags/simulate",
            post(controller::simulate_set_organizer_compliance_flags),
        )
        .route(
            "/organizers/operators",
            post(controller::set_organizer_operator),
        )
        .route(
            "/organizers/operators/simulate",
            post(controller::simulate_set_organizer_operator),
        )
        .route("/organizers/{organizer_id}", get(controller::get_organizer))
}
