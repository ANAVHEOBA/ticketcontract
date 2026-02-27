use axum::{
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
            rpc_url: "https://api.devnet.solana.com".to_string(),
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

#[tokio::test]
async fn nonce_verify_and_scope_guard_flow() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let wallet = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();

    let access_rules = format!(
        r#"[{{"wallet":"{wallet}","role":"organizer_admin","organizer_scopes":["org_123"]}}]"#
    );

    let state = AppState::bootstrap(test_config(access_rules))
        .await
        .unwrap();
    let app = build_router(state);

    let nonce_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/nonce")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"wallet":"{wallet}"}}"#)))
        .unwrap();
    let nonce_resp = app.clone().oneshot(nonce_req).await.unwrap();
    assert_eq!(nonce_resp.status(), StatusCode::OK);
    let nonce_body = nonce_resp.into_body().collect().await.unwrap().to_bytes();
    let nonce_json: serde_json::Value = serde_json::from_slice(&nonce_body).unwrap();

    let nonce = nonce_json["nonce"].as_str().unwrap();
    let message = nonce_json["message"].as_str().unwrap();
    let signature = signing_key.sign(message.as_bytes());
    let signature_b58 = bs58::encode(signature.to_bytes()).into_string();

    let verify_payload = serde_json::json!({
        "wallet": wallet,
        "nonce": nonce,
        "signature": signature_b58,
    });

    let verify_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(verify_payload.to_string()))
        .unwrap();
    let verify_resp = app.clone().oneshot(verify_req).await.unwrap();
    assert_eq!(verify_resp.status(), StatusCode::OK);
    let verify_body = verify_resp.into_body().collect().await.unwrap().to_bytes();
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_body).unwrap();
    let token = verify_json["access_token"].as_str().unwrap();

    let me_req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/me")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let me_resp = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(me_resp.status(), StatusCode::OK);

    let allowed_req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/organizers/org_123/access")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let allowed_resp = app.clone().oneshot(allowed_req).await.unwrap();
    assert_eq!(allowed_resp.status(), StatusCode::OK);

    let denied_req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/organizers/org_999/access")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let denied_resp = app.clone().oneshot(denied_req).await.unwrap();
    assert_eq!(denied_resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn verify_rejects_invalid_signature() {
    let signing_key = SigningKey::from_bytes(&[11u8; 32]);
    let wallet = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    let access_rules =
        format!(r#"[{{"wallet":"{wallet}","role":"scanner","organizer_scopes":["gate_a"]}}]"#);

    let state = AppState::bootstrap(test_config(access_rules))
        .await
        .unwrap();
    let app = build_router(state);

    let nonce_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/nonce")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"wallet":"{wallet}"}}"#)))
        .unwrap();
    let nonce_resp = app.clone().oneshot(nonce_req).await.unwrap();
    assert_eq!(nonce_resp.status(), StatusCode::OK);
    let nonce_body = nonce_resp.into_body().collect().await.unwrap().to_bytes();
    let nonce_json: serde_json::Value = serde_json::from_slice(&nonce_body).unwrap();

    let verify_payload = serde_json::json!({
        "wallet": wallet,
        "nonce": nonce_json["nonce"],
        "signature": bs58::encode([0u8; 64]).into_string(),
    });

    let verify_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(verify_payload.to_string()))
        .unwrap();
    let verify_resp = app.oneshot(verify_req).await.unwrap();
    assert_eq!(verify_resp.status(), StatusCode::UNAUTHORIZED);
}
