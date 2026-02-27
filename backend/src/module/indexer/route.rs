use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::controller;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/indexer/status", get(controller::status))
        .route("/indexer/backfill", post(controller::backfill))
        .route("/indexer/kpis/refresh", post(controller::refresh_kpis))
        .route(
            "/kpis/event-sales/{event_id}",
            get(controller::event_sales_kpi),
        )
        .route(
            "/kpis/resale-health/{event_id}",
            get(controller::resale_health_kpi),
        )
        .route("/kpis/financing-cash", get(controller::financing_cash_kpi))
}
