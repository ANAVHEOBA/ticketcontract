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
async fn scanner_cannot_list_secondary_sale_ticket() {
    let signing_key = SigningKey::from_bytes(&[61u8; 32]);
    let wallet = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    let rules =
        format!(r#"[{{"wallet":"{wallet}","role":"scanner","organizer_scopes":["org_123"]}}]"#);

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);
    let token = auth_token(&app, &signing_key, &wallet).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/secondary-sale/list")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_1",
                "class_id": "class_1",
                "ticket_id": "ticket_1",
                "listing_id": "listing_1",
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn financier_can_create_offer_but_organizer_cannot_clawback() {
    let financier_key = SigningKey::from_bytes(&[62u8; 32]);
    let financier_wallet = bs58::encode(financier_key.verifying_key().as_bytes()).into_string();
    let organizer_key = SigningKey::from_bytes(&[63u8; 32]);
    let organizer_wallet = bs58::encode(organizer_key.verifying_key().as_bytes()).into_string();

    let rules = format!(
        r#"[
            {{"wallet":"{financier_wallet}","role":"financier","organizer_scopes":["org_123"]}},
            {{"wallet":"{organizer_wallet}","role":"organizer_admin","organizer_scopes":["org_123"]}}
        ]"#
    );

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);

    let financier_token = auth_token(&app, &financier_key, &financier_wallet).await;
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/v1/financing/offers")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {financier_token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_1",
                "offer_id": "off_1",
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::BAD_REQUEST);

    let organizer_token = auth_token(&app, &organizer_key, &organizer_wallet).await;
    let clawback_req = Request::builder()
        .method("POST")
        .uri("/api/v1/financing/clawback")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {organizer_token}"))
        .body(Body::from(
            json!({
                "organizer_id": "org_123",
                "event_id": "evt_1",
                "offer_id": "off_1",
                "transaction_base64": "AQAB"
            })
            .to_string(),
        ))
        .unwrap();
    let clawback_resp = app.oneshot(clawback_req).await.unwrap();
    assert_eq!(clawback_resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn secondary_sale_and_financing_reads_require_db() {
    let signing_key = SigningKey::from_bytes(&[64u8; 32]);
    let wallet = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    let rules =
        format!(r#"[{{"wallet":"{wallet}","role":"operator","organizer_scopes":["org_123"]}}]"#);

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);
    let token = auth_token(&app, &signing_key, &wallet).await;

    let listing_req = Request::builder()
        .method("GET")
        .uri("/api/v1/secondary-sale/listings/listing_1")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let listing_resp = app.clone().oneshot(listing_req).await.unwrap();
    assert_eq!(listing_resp.status(), StatusCode::SERVICE_UNAVAILABLE);

    let offer_req = Request::builder()
        .method("GET")
        .uri("/api/v1/financing/offers/off_1")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let offer_resp = app.oneshot(offer_req).await.unwrap();
    assert_eq!(offer_resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}
