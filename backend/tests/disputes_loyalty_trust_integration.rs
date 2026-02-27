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
async fn scanner_cannot_refund_but_operator_can_flag_dispute() {
    let scanner_key = SigningKey::from_bytes(&[81u8; 32]);
    let scanner_wallet = bs58::encode(scanner_key.verifying_key().as_bytes()).into_string();
    let operator_key = SigningKey::from_bytes(&[82u8; 32]);
    let operator_wallet = bs58::encode(operator_key.verifying_key().as_bytes()).into_string();

    let rules = format!(
        r#"[
            {{"wallet":"{scanner_wallet}","role":"scanner","organizer_scopes":["org_123"]}},
            {{"wallet":"{operator_wallet}","role":"operator","organizer_scopes":["org_123"]}}
        ]"#
    );

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);

    let scanner_token = auth_token(&app, &scanner_key, &scanner_wallet).await;
    let refund_req = Request::builder()
        .method("POST")
        .uri("/api/v1/disputes/refund")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {scanner_token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_1",
                "ticket_id": "ticket_1",
                "dispute_id": "disp_1",
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();
    let refund_resp = app.clone().oneshot(refund_req).await.unwrap();
    assert_eq!(refund_resp.status(), StatusCode::FORBIDDEN);

    let operator_token = auth_token(&app, &operator_key, &operator_wallet).await;
    let dispute_req = Request::builder()
        .method("POST")
        .uri("/api/v1/disputes/flag")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {operator_token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_1",
                "ticket_id": "ticket_1",
                "dispute_id": "disp_1",
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();
    let dispute_resp = app.oneshot(dispute_req).await.unwrap();
    assert_eq!(dispute_resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn loyalty_trust_role_paths_enforced() {
    let scanner_key = SigningKey::from_bytes(&[83u8; 32]);
    let scanner_wallet = bs58::encode(scanner_key.verifying_key().as_bytes()).into_string();
    let operator_key = SigningKey::from_bytes(&[84u8; 32]);
    let operator_wallet = bs58::encode(operator_key.verifying_key().as_bytes()).into_string();
    let admin_key = SigningKey::from_bytes(&[85u8; 32]);
    let admin_wallet = bs58::encode(admin_key.verifying_key().as_bytes()).into_string();

    let rules = format!(
        r#"[
            {{"wallet":"{scanner_wallet}","role":"scanner","organizer_scopes":["org_123"]}},
            {{"wallet":"{operator_wallet}","role":"operator","organizer_scopes":["org_123"]}},
            {{"wallet":"{admin_wallet}","role":"protocol_admin","organizer_scopes":["org_123"]}}
        ]"#
    );

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);

    let scanner_token = auth_token(&app, &scanner_key, &scanner_wallet).await;
    let accrue_req = Request::builder()
        .method("POST")
        .uri("/api/v1/loyalty/accrue")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {scanner_token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_1",
                "wallet": scanner_wallet,
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();
    let accrue_resp = app.clone().oneshot(accrue_req).await.unwrap();
    assert_eq!(accrue_resp.status(), StatusCode::FORBIDDEN);

    let operator_token = auth_token(&app, &operator_key, &operator_wallet).await;
    let attendance_req = Request::builder()
        .method("POST")
        .uri("/api/v1/trust/attendance")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {operator_token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_1",
                "wallet": operator_wallet,
                "signal_id": "sig_1",
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();
    let attendance_resp = app.clone().oneshot(attendance_req).await.unwrap();
    assert_eq!(attendance_resp.status(), StatusCode::BAD_REQUEST);

    let admin_token = auth_token(&app, &admin_key, &admin_wallet).await;
    let schema_req = Request::builder()
        .method("POST")
        .uri("/api/v1/trust/schema-version")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {admin_token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "schema_version": 2,
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();
    let schema_resp = app.oneshot(schema_req).await.unwrap();
    assert_eq!(schema_resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn dispute_and_loyalty_reads_require_db() {
    let key = SigningKey::from_bytes(&[86u8; 32]);
    let wallet = bs58::encode(key.verifying_key().as_bytes()).into_string();
    let rules =
        format!(r#"[{{"wallet":"{wallet}","role":"operator","organizer_scopes":["org_123"]}}]"#);

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);
    let token = auth_token(&app, &key, &wallet).await;

    let queue_req = Request::builder()
        .method("GET")
        .uri("/api/v1/disputes/queue?organizer_id=org_123")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let queue_resp = app.clone().oneshot(queue_req).await.unwrap();
    assert_eq!(queue_resp.status(), StatusCode::SERVICE_UNAVAILABLE);

    let loyalty_req = Request::builder()
        .method("GET")
        .uri(&format!(
            "/api/v1/loyalty?wallet={wallet}&organizer_id=org_123"
        ))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let loyalty_resp = app.oneshot(loyalty_req).await.unwrap();
    assert_eq!(loyalty_resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}
