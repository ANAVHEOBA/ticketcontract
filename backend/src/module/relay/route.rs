use axum::{Router, routing::post};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new().route("/relay/submit", post(controller::submit))
}
