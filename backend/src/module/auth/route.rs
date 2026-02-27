use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/nonce", post(controller::issue_nonce))
        .route("/auth/verify", post(controller::verify_signature))
        .route("/auth/provider/verify", post(controller::verify_provider))
        .route("/auth/me", get(controller::me))
        .route(
            "/auth/organizers/{organizer_id}/access",
            get(controller::organizer_access),
        )
}
