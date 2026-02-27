use axum::{Router, routing::post};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/settlement/primary",
            post(controller::settle_primary_revenue),
        )
        .route(
            "/settlement/primary/simulate",
            post(controller::simulate_settle_primary_revenue),
        )
        .route(
            "/settlement/resale",
            post(controller::settle_resale_revenue),
        )
        .route(
            "/settlement/resale/simulate",
            post(controller::simulate_settle_resale_revenue),
        )
        .route(
            "/settlement/finalize",
            post(controller::finalize_settlement),
        )
        .route(
            "/settlement/finalize/simulate",
            post(controller::simulate_finalize_settlement),
        )
}
