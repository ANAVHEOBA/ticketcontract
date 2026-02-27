use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use backend::{
    app::{AppState, build_router},
    config::{
        AppConfig, AppSettings, AuthSettings, ChainSettings, DbSettings, IndexerSettings,
        ObservabilitySettings, RedisSettings, SettlementSettings,
    },
};
use ed25519_dalek::{Signer, SigningKey};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

fn test_config(access_rules_json: String) -> AppConfig {
    AppConfig {
        app: AppSettings {
            env: "test".to_string(),
            port: 8080,
            api_prefix: "/api/v1".to_string(),
            cors_origin: "http://localhost:3000".to_string(),
            log_level: "debug".to_string(),
        },
        auth: AuthSettings {
            jwt_secret: "test-secret".to_string(),
            jwt_expires_seconds: 3600,
            siws_nonce_ttl_seconds: 300,
            access_rules_json: Some(access_rules_json),
            google_client_id: None,
        },
        chain: ChainSettings {
            cluster: "devnet".to_string(),
            rpc_url: "http://127.0.0.1:1".to_string(),
            ws_url: "wss://api.devnet.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            program_id: "Evb4oQnwYDZhabE5E5tuJPg2yvCMGgwMRgkxzzaUw5Rv".to_string(),
            anchor_wallet: "/tmp/id.json".to_string(),
            idl_path: Some("/tmp/does-not-exist.json".to_string()),
        },
        db: DbSettings {
            database_url: "mongodb://127.0.0.1:27017".to_string(),
            pool_min: 1,
            pool_max: 2,
            db_required: false,
        },
        redis: RedisSettings {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            queue_prefix: "ticketing".to_string(),
        },
        indexer: IndexerSettings {
            enabled: false,
            start_slot: 0,
            backfill_end_slot: None,
            batch_size: 100,
            confirmation_depth: 1,
            poll_interval_ms: 4000,
        },
        settlement: SettlementSettings {
            idempotency_ttl_seconds: 86400,
        },
        observability: ObservabilitySettings {
            sentry_dsn: None,
            otel_exporter_otlp_endpoint: None,
        },
    }
}

async fn auth_token(app: &Router, signing_key: &SigningKey, wallet: &str) -> String {
    let nonce_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/nonce")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"wallet":"{wallet}"}}"#)))
        .unwrap();
    let nonce_resp = app.clone().oneshot(nonce_req).await.unwrap();
    let nonce_body = nonce_resp.into_body().collect().await.unwrap().to_bytes();
    let nonce_json: Value = serde_json::from_slice(&nonce_body).unwrap();

    let nonce = nonce_json["nonce"].as_str().unwrap();
    let message = nonce_json["message"].as_str().unwrap();
    let signature = signing_key.sign(message.as_bytes());
    let signature_b58 = bs58::encode(signature.to_bytes()).into_string();

    let verify_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "wallet": wallet,
                "nonce": nonce,
                "signature": signature_b58,
            })
            .to_string(),
        ))
        .unwrap();
    let verify_resp = app.clone().oneshot(verify_req).await.unwrap();
    let verify_body = verify_resp.into_body().collect().await.unwrap().to_bytes();
    let verify_json: Value = serde_json::from_slice(&verify_body).unwrap();

    verify_json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn e2e_backend_flow_covers_docs_ops_and_core_paths() {
    let admin_key = SigningKey::from_bytes(&[111u8; 32]);
    let organizer_key = SigningKey::from_bytes(&[112u8; 32]);
    let admin_wallet = bs58::encode(admin_key.verifying_key().as_bytes()).into_string();
    let organizer_wallet = bs58::encode(organizer_key.verifying_key().as_bytes()).into_string();
    let rules = format!(
        r#"[
            {{"wallet":"{admin_wallet}","role":"protocol_admin","organizer_scopes":["org_123"]}},
            {{"wallet":"{organizer_wallet}","role":"organizer_admin","organizer_scopes":["org_123"]}}
        ]"#
    );

    let app = build_router(AppState::bootstrap(test_config(rules)).await.unwrap());

    let docs = Request::builder()
        .method("GET")
        .uri("/api/v1/docs/openapi.yaml")
        .body(Body::empty())
        .unwrap();
    let docs_resp = app.clone().oneshot(docs).await.unwrap();
    assert_eq!(docs_resp.status(), StatusCode::OK);

    let postman = Request::builder()
        .method("GET")
        .uri("/api/v1/docs/postman_collection.json")
        .body(Body::empty())
        .unwrap();
    let postman_resp = app.clone().oneshot(postman).await.unwrap();
    assert_eq!(postman_resp.status(), StatusCode::OK);

    let admin_token = auth_token(&app, &admin_key, &admin_wallet).await;
    let organizer_token = auth_token(&app, &organizer_key, &organizer_wallet).await;

    let event_create = Request::builder()
        .method("POST")
        .uri("/api/v1/events")
        .header("authorization", format!("Bearer {organizer_token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_123",
                "transaction_base64": "AQ=="
            })
            .to_string(),
        ))
        .unwrap();
    let event_resp = app.clone().oneshot(event_create).await.unwrap();
    assert_eq!(event_resp.status(), StatusCode::BAD_REQUEST);

    let underwriting = Request::builder()
        .method("POST")
        .uri("/api/v1/underwriting/financing/proposal")
        .header("authorization", format!("Bearer {organizer_token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_123",
                "requested_advance_amount": 5000000u64,
                "projected_gross_revenue": 25000000u64
            })
            .to_string(),
        ))
        .unwrap();
    let underwriting_resp = app.clone().oneshot(underwriting).await.unwrap();
    assert_eq!(underwriting_resp.status(), StatusCode::SERVICE_UNAVAILABLE);

    let resale_compiler = Request::builder()
        .method("POST")
        .uri("/api/v1/resale-compiler/simulate")
        .header("authorization", format!("Bearer {organizer_token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_123"
            })
            .to_string(),
        ))
        .unwrap();
    let resale_resp = app.clone().oneshot(resale_compiler).await.unwrap();
    assert_eq!(resale_resp.status(), StatusCode::SERVICE_UNAVAILABLE);

    let ops_metrics = Request::builder()
        .method("GET")
        .uri("/api/v1/ops/metrics")
        .header("authorization", format!("Bearer {admin_token}"))
        .body(Body::empty())
        .unwrap();
    let ops_metrics_resp = app.clone().oneshot(ops_metrics).await.unwrap();
    assert_eq!(ops_metrics_resp.status(), StatusCode::OK);
    let metrics_body = ops_metrics_resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let metrics_json: Value = serde_json::from_slice(&metrics_body).unwrap();
    assert!(metrics_json["metrics"]["tx_failed"].as_u64().unwrap_or(0) >= 1);

    let ops_alerts = Request::builder()
        .method("GET")
        .uri("/api/v1/ops/alerts")
        .header("authorization", format!("Bearer {admin_token}"))
        .body(Body::empty())
        .unwrap();
    let ops_alerts_resp = app.clone().oneshot(ops_alerts).await.unwrap();
    assert_eq!(ops_alerts_resp.status(), StatusCode::OK);
}
