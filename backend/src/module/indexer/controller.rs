use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde_json::json;

use crate::{
    app::AppState,
    error::{ApiError, AppResult},
    module::{
        auth::{
            guard::{AuthContext, require_any_role},
            model::Role,
        },
        indexer::schema::{
            BackfillRequest, BackfillResponse, FinancingKpiQuery, IndexerStatusResponse,
            KpiRefreshResponse,
        },
    },
};

pub async fn status(
    auth: AuthContext,
    State(state): State<AppState>,
) -> AppResult<Json<IndexerStatusResponse>> {
    require_any_role(&auth, &[Role::ProtocolAdmin])?;

    let enabled = state.config.indexer.enabled;
    if let Some(indexer) = &state.indexer_service {
        let s = indexer.status().await;
        return Ok(Json(IndexerStatusResponse {
            enabled,
            running: s.running,
            last_poll_epoch: s.last_poll_epoch,
            last_processed_slot: s.last_processed_slot,
            last_signature: s.last_signature,
            backfill_active: s.backfill_active,
            backfill_pending: s.backfill_pending,
        }));
    }

    Ok(Json(IndexerStatusResponse {
        enabled,
        running: false,
        last_poll_epoch: 0,
        last_processed_slot: 0,
        last_signature: None,
        backfill_active: false,
        backfill_pending: 0,
    }))
}

pub async fn backfill(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<BackfillRequest>,
) -> AppResult<Json<BackfillResponse>> {
    require_any_role(&auth, &[Role::ProtocolAdmin])?;
    let Some(indexer) = &state.indexer_service else {
        return Err(ApiError::BadRequest("indexer is not available".to_string()));
    };

    indexer
        .enqueue_backfill(payload.start_slot, payload.end_slot)
        .await
        .map_err(ApiError::map_chain_error)?;
    let _ = state
        .ops_service
        .audit(
            &auth.wallet,
            role_name(&auth.role),
            "indexer_backfill",
            Some(json!({
                "start_slot": payload.start_slot,
                "end_slot": payload.end_slot
            })),
        )
        .await;

    Ok(Json(BackfillResponse { queued: true }))
}

pub async fn refresh_kpis(
    auth: AuthContext,
    State(state): State<AppState>,
) -> AppResult<Json<KpiRefreshResponse>> {
    require_any_role(&auth, &[Role::ProtocolAdmin])?;
    let Some(kpi) = &state.kpi_service else {
        return Err(ApiError::DatabaseUnavailable);
    };

    kpi.refresh_all().await.map_err(ApiError::map_db_error)?;
    let _ = state
        .ops_service
        .audit(&auth.wallet, role_name(&auth.role), "refresh_kpis", None)
        .await;
    Ok(Json(KpiRefreshResponse { refreshed: true }))
}

pub async fn event_sales_kpi(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    let Some(kpi) = &state.kpi_service else {
        return Err(ApiError::DatabaseUnavailable);
    };

    let doc = kpi
        .get_event_sales(&event_id)
        .await
        .map_err(ApiError::map_db_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        mongodb::bson::from_bson(mongodb::bson::Bson::Document(doc))
            .map_err(ApiError::map_db_error)?,
    ))
}

fn role_name(role: &Role) -> &'static str {
    match role {
        Role::ProtocolAdmin => "protocol_admin",
        Role::OrganizerAdmin => "organizer_admin",
        Role::Operator => "operator",
        Role::Scanner => "scanner",
        Role::Financier => "financier",
    }
}

pub async fn resale_health_kpi(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    let Some(kpi) = &state.kpi_service else {
        return Err(ApiError::DatabaseUnavailable);
    };

    let doc = kpi
        .get_resale_health(&event_id)
        .await
        .map_err(ApiError::map_db_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        mongodb::bson::from_bson(mongodb::bson::Bson::Document(doc))
            .map_err(ApiError::map_db_error)?,
    ))
}

pub async fn financing_cash_kpi(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<FinancingKpiQuery>,
) -> AppResult<Json<serde_json::Value>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    let Some(kpi) = &state.kpi_service else {
        return Err(ApiError::DatabaseUnavailable);
    };

    let doc = kpi
        .get_financing_cash_position(&query.organizer_id, &query.event_id)
        .await
        .map_err(ApiError::map_db_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        mongodb::bson::from_bson(mongodb::bson::Bson::Document(doc))
            .map_err(ApiError::map_db_error)?,
    ))
}
