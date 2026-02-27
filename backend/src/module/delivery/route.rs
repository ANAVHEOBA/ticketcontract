use axum::{Router, routing::get};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/docs/openapi.yaml", get(controller::openapi_yaml))
        .route(
            "/docs/postman_collection.json",
            get(controller::postman_collection),
        )
        .route(
            "/docs/bruno_collection.json",
            get(controller::bruno_collection),
        )
}
