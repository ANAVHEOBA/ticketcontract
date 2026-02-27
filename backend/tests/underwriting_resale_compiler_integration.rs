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
async fn scanner_cannot_call_underwriting() {
    let scanner_key = SigningKey::from_bytes(&[101u8; 32]);
    let scanner_wallet = bs58::encode(scanner_key.verifying_key().as_bytes()).into_string();
    let rules = format!(
        r#"[{{"wallet":"{scanner_wallet}","role":"scanner","organizer_scopes":["org_123"]}}]"#
    );
    let app = build_router(AppState::bootstrap(test_config(rules)).await.unwrap());
    let token = auth_token(&app, &scanner_key, &scanner_wallet).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/underwriting/financing/proposal")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_123",
                "requested_advance_amount": 5_000_000u64,
                "projected_gross_revenue": 25_000_000u64,
                "tenor_days": 45
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn operator_underwriting_returns_db_unavailable_when_mongo_offline() {
    let op_key = SigningKey::from_bytes(&[102u8; 32]);
    let op_wallet = bs58::encode(op_key.verifying_key().as_bytes()).into_string();
    let rules =
        format!(r#"[{{"wallet":"{op_wallet}","role":"operator","organizer_scopes":["org_123"]}}]"#);
    let app = build_router(AppState::bootstrap(test_config(rules)).await.unwrap());
    let token = auth_token(&app, &op_key, &op_wallet).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/underwriting/financing/proposal")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_123",
                "requested_advance_amount": 5_000_000u64,
                "projected_gross_revenue": 25_000_000u64
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn scanner_cannot_call_resale_compiler() {
    let scanner_key = SigningKey::from_bytes(&[103u8; 32]);
    let scanner_wallet = bs58::encode(scanner_key.verifying_key().as_bytes()).into_string();
    let rules = format!(
        r#"[{{"wallet":"{scanner_wallet}","role":"scanner","organizer_scopes":["org_123"]}}]"#
    );
    let app = build_router(AppState::bootstrap(test_config(rules)).await.unwrap());
    let token = auth_token(&app, &scanner_key, &scanner_wallet).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/resale-compiler/simulate")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_123",
                "class_id": "class_abc"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn organizer_resale_compiler_returns_db_unavailable_when_mongo_offline() {
    let org_key = SigningKey::from_bytes(&[104u8; 32]);
    let org_wallet = bs58::encode(org_key.verifying_key().as_bytes()).into_string();
    let rules = format!(
        r#"[{{"wallet":"{org_wallet}","role":"organizer_admin","organizer_scopes":["org_123"]}}]"#
    );
    let app = build_router(AppState::bootstrap(test_config(rules)).await.unwrap());
    let token = auth_token(&app, &org_key, &org_wallet).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/resale-compiler/simulate")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_123",
                "goals": {
                  "liquidity_weight": 0.5,
                  "fairness_weight": 0.3,
                  "royalty_weight": 0.2
                },
                "candidates": [{
                  "candidate_id": "balanced",
                  "max_markup_bps": 2200,
                  "royalty_bps": 700
                }]
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}
