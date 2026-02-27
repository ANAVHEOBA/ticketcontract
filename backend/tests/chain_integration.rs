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
use solana_sdk::pubkey::Pubkey;
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
    assert_eq!(nonce_resp.status(), StatusCode::OK);
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
    assert_eq!(verify_resp.status(), StatusCode::OK);
    let verify_body = verify_resp.into_body().collect().await.unwrap().to_bytes();
    let verify_json: Value = serde_json::from_slice(&verify_body).unwrap();

    verify_json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn derive_pda_endpoint_returns_expected_address() {
    let signing_key = SigningKey::from_bytes(&[12u8; 32]);
    let wallet = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    let rules =
        format!(r#"[{{"wallet":"{wallet}","role":"operator","organizer_scopes":["org_123"]}}]"#);

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/chain/pda/derive")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "seeds": [
                    {"value": "event", "encoding": "utf8"},
                    {"value": "123", "encoding": "utf8"}
                ]
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let parsed: Value = serde_json::from_slice(&body).unwrap();

    let program = "Evb4oQnwYDZhabE5E5tuJPg2yvCMGgwMRgkxzzaUw5Rv"
        .parse::<Pubkey>()
        .unwrap();
    let (expected, _) = Pubkey::find_program_address(&[b"event", b"123"], &program);

    assert_eq!(parsed["pda"].as_str().unwrap(), expected.to_string());
}

#[tokio::test]
async fn simulate_endpoint_requires_auth_and_maps_chain_failure() {
    let signing_key = SigningKey::from_bytes(&[13u8; 32]);
    let wallet = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    let rules =
        format!(r#"[{{"wallet":"{wallet}","role":"operator","organizer_scopes":["org_123"]}}]"#);

    let state = AppState::bootstrap(test_config(rules)).await.unwrap();
    let app = build_router(state);

    let unauth_req = Request::builder()
        .method("POST")
        .uri("/api/v1/chain/tx/simulate")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "transaction_base64": "AQAB",
                "sig_verify": false,
                "replace_recent_blockhash": true
            })
            .to_string(),
        ))
        .unwrap();
    let unauth_resp = app.clone().oneshot(unauth_req).await.unwrap();
    assert_eq!(unauth_resp.status(), StatusCode::UNAUTHORIZED);

    let token = auth_token(&app, &signing_key, &wallet).await;

    let sim_req = Request::builder()
        .method("POST")
        .uri("/api/v1/chain/tx/simulate")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(
            json!({
                "transaction_base64": "AQAB",
                "sig_verify": false,
                "replace_recent_blockhash": true
            })
            .to_string(),
        ))
        .unwrap();

    let sim_resp = app.oneshot(sim_req).await.unwrap();
    assert_eq!(sim_resp.status(), StatusCode::BAD_GATEWAY);
}
