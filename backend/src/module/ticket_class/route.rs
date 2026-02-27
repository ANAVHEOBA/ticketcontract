use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/ticket-classes",
            get(controller::list_ticket_classes).post(controller::create_ticket_class),
        )
        .route(
            "/ticket-classes/simulate",
            post(controller::simulate_create_ticket_class),
        )
        .route(
            "/ticket-classes/update",
            post(controller::update_ticket_class),
        )
        .route(
            "/ticket-classes/update/simulate",
            post(controller::simulate_update_ticket_class),
        )
        .route(
            "/ticket-classes/reserve",
            post(controller::reserve_inventory),
        )
        .route(
            "/ticket-classes/reserve/simulate",
            post(controller::simulate_reserve_inventory),
        )
        .route(
            "/ticket-classes/{class_id}",
            get(controller::get_ticket_class),
        )
        .route(
            "/ticket-classes/{class_id}/analytics",
            get(controller::ticket_class_analytics),
        )
}
