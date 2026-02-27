use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/events",
            get(controller::list_events).post(controller::create_event),
        )
        .route("/events/simulate", post(controller::simulate_create_event))
        .route("/events/update", post(controller::update_event))
        .route(
            "/events/update/simulate",
            post(controller::simulate_update_event),
        )
        .route("/events/freeze", post(controller::freeze_event))
        .route(
            "/events/freeze/simulate",
            post(controller::simulate_freeze_event),
        )
        .route("/events/cancel", post(controller::cancel_event))
        .route(
            "/events/cancel/simulate",
            post(controller::simulate_cancel_event),
        )
        .route("/events/pause", post(controller::pause_event))
        .route(
            "/events/pause/simulate",
            post(controller::simulate_pause_event),
        )
        .route("/events/close", post(controller::close_event))
        .route(
            "/events/close/simulate",
            post(controller::simulate_close_event),
        )
        .route(
            "/events/restrictions",
            post(controller::set_event_restrictions),
        )
        .route(
            "/events/restrictions/simulate",
            post(controller::simulate_set_event_restrictions),
        )
        .route(
            "/events/loyalty-multiplier",
            post(controller::set_event_loyalty_multiplier),
        )
        .route(
            "/events/loyalty-multiplier/simulate",
            post(controller::simulate_set_event_loyalty_multiplier),
        )
        .route("/events/{event_id}", get(controller::get_event))
}
