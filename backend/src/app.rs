use std::sync::Arc;

use anyhow::Context;
use axum::{
    Json, Router,
    http::{HeaderValue, Method, StatusCode, header},
    response::IntoResponse,
};
use serde::Serialize;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    config, module,
    service::{
        auth_service::AuthService, chain_service::ChainService,
        idempotency_service::IdempotencyService, indexer_service::IndexerService,
        kpi_service::KpiService, ops_service::OpsService,
        resale_compiler_service::ResaleCompilerService, underwriting_service::UnderwritingService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<config::AppConfig>,
    pub mongo: Option<mongodb::Client>,
    pub http: reqwest::Client,
    pub auth_service: AuthService,
    pub chain_service: ChainService,
    pub ops_service: OpsService,
    pub idempotency_service: IdempotencyService,
    pub indexer_service: Option<IndexerService>,
    pub kpi_service: Option<KpiService>,
    pub underwriting_service: Option<UnderwritingService>,
    pub resale_compiler_service: Option<ResaleCompilerService>,
}

impl AppState {
    pub async fn bootstrap(config: config::AppConfig) -> anyhow::Result<Self> {
        let mongo = match config::db::connect_mongo(&config.db.database_url).await {
            Ok(client) => Some(client),
            Err(err) if !config.db.db_required => {
                tracing::warn!(error = %err, "mongodb unavailable but DB_REQUIRED=false, continuing");
                None
            }
            Err(err) => return Err(err).context("failed connecting to mongodb"),
        };

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("failed to create http client")?;

        let auth_service = AuthService::new(&config, http.clone()).await?;
        let ops_service = OpsService::new(mongo.clone());
        let chain_service =
            ChainService::new(&config, http.clone(), Some(ops_service.clone())).await?;
        let idempotency_service =
            IdempotencyService::new(config.settlement.idempotency_ttl_seconds);
        let kpi_service = mongo.clone().map(KpiService::new);
        let underwriting_service = mongo.clone().map(UnderwritingService::new);
        let resale_compiler_service = mongo.clone().map(ResaleCompilerService::new);
        let mut indexer_service = None;

        if config.indexer.enabled {
            if let Some(mongo_client) = mongo.clone() {
                let indexer = IndexerService::new(
                    &config,
                    http.clone(),
                    mongo_client,
                    Some(ops_service.clone()),
                );
                indexer.clone().spawn();
                indexer_service = Some(indexer);
            } else {
                tracing::warn!("indexer enabled but mongo is unavailable; worker not started");
            }
        }

        Ok(Self {
            config: Arc::new(config),
            mongo,
            http,
            auth_service,
            chain_service,
            ops_service,
            idempotency_service,
            indexer_service,
            kpi_service,
            underwriting_service,
            resale_compiler_service,
        })
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors_origin = HeaderValue::from_str(&state.config.app.cors_origin)
        .unwrap_or_else(|_| HeaderValue::from_static("*"));

    let cors = CorsLayer::new()
        .allow_origin(cors_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let root = Router::new()
        .merge(module::health::route::router())
        .merge(module::auth::route::router())
        .merge(module::chain::route::router())
        .merge(module::delivery::route::router())
        .merge(module::protocol_admin::route::router())
        .merge(module::relay::route::router())
        .merge(module::organizer::route::router())
        .merge(module::event::route::router())
        .merge(module::ticket_class::route::router())
        .merge(module::primary_sale::route::router())
        .merge(module::ticket_state::route::router())
        .merge(module::resale_policy::route::router())
        .merge(module::secondary_sale::route::router())
        .merge(module::financing::route::router())
        .merge(module::settlement::route::router())
        .merge(module::checkin::route::router())
        .merge(module::disputes::route::router())
        .merge(module::loyalty_trust::route::router())
        .merge(module::indexer::route::router())
        .merge(module::underwriting::route::router())
        .merge(module::resale_compiler::route::router())
        .merge(module::ops::route::router())
        .route("/", axum::routing::get(root));

    Router::new()
        .merge(root.clone())
        .nest(&state.config.app.api_prefix, root)
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

#[derive(Serialize)]
struct RootResponse {
    status: &'static str,
    service: &'static str,
}

async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(RootResponse {
            status: "ok",
            service: "ticketing-backend",
        }),
    )
}
