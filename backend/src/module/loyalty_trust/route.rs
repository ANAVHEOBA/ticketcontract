use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/loyalty/accrue", post(controller::accrue_points))
        .route("/loyalty/redeem", post(controller::redeem_points))
        .route("/trust/purchase", post(controller::record_purchase_signal))
        .route(
            "/trust/attendance",
            post(controller::record_attendance_signal),
        )
        .route("/trust/abuse", post(controller::flag_trust_abuse))
        .route(
            "/trust/schema-version",
            post(controller::set_trust_schema_version),
        )
        .route("/loyalty", get(controller::get_loyalty))
        .route("/trust/signals", get(controller::get_trust_signals))
}
