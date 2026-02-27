use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tickets/{ticket_id}", get(controller::get_ticket))
        .route(
            "/ticket-state/metadata",
            post(controller::update_ticket_metadata),
        )
        .route(
            "/ticket-state/metadata/simulate",
            post(controller::simulate_update_ticket_metadata),
        )
        .route(
            "/ticket-state/transition",
            post(controller::transition_ticket_status),
        )
}
