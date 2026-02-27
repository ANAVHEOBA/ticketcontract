use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app::AppState,
    error::AppResult,
    module::{
        auth::{
            guard::{AuthContext, require_any_role},
            model::Role,
        },
        ops::schema::{AlertsResponse, AuditLogQuery, AuditLogsResponse, MetricsResponse},
    },
};

pub async fn metrics(
    auth: AuthContext,
    State(state): State<AppState>,
) -> AppResult<Json<MetricsResponse>> {
    require_any_role(&auth, &[Role::ProtocolAdmin])?;
    let metrics = state.ops_service.metrics_snapshot().await;
    Ok(Json(MetricsResponse { metrics }))
}

pub async fn alerts(
    auth: AuthContext,
    State(state): State<AppState>,
) -> AppResult<Json<AlertsResponse>> {
    require_any_role(&auth, &[Role::ProtocolAdmin])?;
    let indexer_status = match &state.indexer_service {
        Some(i) => Some(i.status().await),
        None => None,
    };
    let alerts = state.ops_service.alerts_snapshot(indexer_status).await;
    Ok(Json(AlertsResponse { alerts }))
}

pub async fn audit_logs(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<Json<AuditLogsResponse>> {
    require_any_role(&auth, &[Role::ProtocolAdmin])?;
    let logs = state.ops_service.list_audit_logs(query.limit).await?;
    Ok(Json(AuditLogsResponse { logs }))
}
