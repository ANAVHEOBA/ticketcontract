use anyhow::{Context, bail};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub app: AppSettings,
    pub auth: AuthSettings,
    pub chain: ChainSettings,
    pub db: DbSettings,
    pub redis: RedisSettings,
    pub indexer: IndexerSettings,
    pub settlement: SettlementSettings,
    pub observability: ObservabilitySettings,
}

#[derive(Clone, Debug)]
pub struct AppSettings {
    pub env: String,
    pub port: u16,
    pub api_prefix: String,
    pub cors_origin: String,
    pub log_level: String,
}

#[derive(Clone, Debug)]
pub struct AuthSettings {
    pub jwt_secret: String,
    pub jwt_expires_seconds: u64,
    pub siws_nonce_ttl_seconds: u64,
    pub access_rules_json: Option<String>,
    pub google_client_id: Option<String>,
    pub allow_unlisted_wallets: bool,
}

#[derive(Clone, Debug)]
pub struct ChainSettings {
    pub cluster: String,
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment: String,
    pub program_id: String,
    pub anchor_wallet: String,
    pub idl_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DbSettings {
    pub database_url: String,
    pub pool_min: u32,
    pub pool_max: u32,
    pub db_required: bool,
}

#[derive(Clone, Debug)]
pub struct RedisSettings {
    pub redis_url: String,
    pub queue_prefix: String,
}

#[derive(Clone, Debug)]
pub struct IndexerSettings {
    pub enabled: bool,
    pub start_slot: u64,
    pub backfill_end_slot: Option<u64>,
    pub batch_size: u64,
    pub confirmation_depth: u64,
    pub poll_interval_ms: u64,
}

#[derive(Clone, Debug)]
pub struct ObservabilitySettings {
    pub sentry_dsn: Option<String>,
    pub otel_exporter_otlp_endpoint: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SettlementSettings {
    pub idempotency_ttl_seconds: u64,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_expires_seconds = match std::env::var("JWT_EXPIRES_SECONDS") {
            Ok(v) => parse_u64("JWT_EXPIRES_SECONDS", &v)?,
            Err(_) => parse_duration_alias_to_seconds(&required("JWT_EXPIRES_IN")?)?,
        };

        let queue_prefix = std::env::var("QUEUE_PREFIX")
            .or_else(|_| std::env::var("BULLMQ_PREFIX"))
            .context("missing QUEUE_PREFIX or BULLMQ_PREFIX")?;

        let app = AppSettings {
            env: std::env::var("APP_ENV")
                .or_else(|_| std::env::var("NODE_ENV"))
                .unwrap_or_else(|_| "development".to_string()),
            port: parse_u16("PORT", &required("PORT")?)?,
            api_prefix: required("API_PREFIX")?,
            cors_origin: required("CORS_ORIGIN")?,
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        };

        let auth = AuthSettings {
            jwt_secret: required("JWT_SECRET")?,
            jwt_expires_seconds,
            siws_nonce_ttl_seconds: parse_u64(
                "SIWS_NONCE_TTL_SECONDS",
                &required("SIWS_NONCE_TTL_SECONDS")?,
            )?,
            access_rules_json: optional_non_empty("AUTH_ACCESS_RULES_JSON"),
            google_client_id: optional_non_empty("GOOGLE_CLIENT_ID"),
            allow_unlisted_wallets: parse_bool_with_default(
                "AUTH_ALLOW_UNLISTED_WALLETS",
                app.env != "production",
            )?,
        };

        let chain = ChainSettings {
            cluster: required("SOLANA_CLUSTER")?,
            rpc_url: required("SOLANA_RPC_URL")?,
            ws_url: required("SOLANA_WS_URL")?,
            commitment: required("SOLANA_COMMITMENT")?,
            program_id: required("PROGRAM_ID")?,
            anchor_wallet: required("ANCHOR_WALLET")?,
            idl_path: optional_non_empty("ANCHOR_IDL_PATH"),
        };

        let db = DbSettings {
            database_url: required("DATABASE_URL")?,
            pool_min: parse_u32("DB_POOL_MIN", &required("DB_POOL_MIN")?)?,
            pool_max: parse_u32("DB_POOL_MAX", &required("DB_POOL_MAX")?)?,
            db_required: parse_bool_with_default("DB_REQUIRED", true)?,
        };

        let redis = RedisSettings {
            redis_url: required("REDIS_URL")?,
            queue_prefix,
        };

        let indexer = IndexerSettings {
            enabled: parse_bool_with_default("INDEXER_ENABLED", true)?,
            start_slot: parse_u64(
                "INDEXER_START_SLOT",
                &std::env::var("INDEXER_START_SLOT").unwrap_or_else(|_| "0".to_string()),
            )?,
            backfill_end_slot: optional_u64("INDEXER_BACKFILL_END_SLOT")?,
            batch_size: parse_u64(
                "INDEXER_BATCH_SIZE",
                &std::env::var("INDEXER_BATCH_SIZE").unwrap_or_else(|_| "500".to_string()),
            )?,
            confirmation_depth: parse_u64(
                "INDEXER_CONFIRMATION_DEPTH",
                &std::env::var("INDEXER_CONFIRMATION_DEPTH").unwrap_or_else(|_| "1".to_string()),
            )?,
            poll_interval_ms: parse_u64(
                "INDEXER_POLL_INTERVAL_MS",
                &std::env::var("INDEXER_POLL_INTERVAL_MS").unwrap_or_else(|_| "4000".to_string()),
            )?,
        };

        let observability = ObservabilitySettings {
            sentry_dsn: optional_non_empty("SENTRY_DSN"),
            otel_exporter_otlp_endpoint: optional_non_empty("OTEL_EXPORTER_OTLP_ENDPOINT"),
        };

        let settlement = SettlementSettings {
            idempotency_ttl_seconds: parse_u64(
                "IDEMPOTENCY_TTL_SECONDS",
                &std::env::var("IDEMPOTENCY_TTL_SECONDS").unwrap_or_else(|_| "86400".to_string()),
            )?,
        };

        Ok(Self {
            app,
            auth,
            chain,
            db,
            redis,
            indexer,
            settlement,
            observability,
        })
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    let value = std::env::var(key).with_context(|| format!("missing {key}"))?;
    if value.trim().is_empty() {
        bail!("{key} cannot be empty");
    }
    Ok(value)
}

fn optional_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn parse_u16(key: &str, value: &str) -> anyhow::Result<u16> {
    value
        .parse::<u16>()
        .with_context(|| format!("{key} must be a valid u16"))
}

fn parse_u32(key: &str, value: &str) -> anyhow::Result<u32> {
    value
        .parse::<u32>()
        .with_context(|| format!("{key} must be a valid u32"))
}

fn parse_u64(key: &str, value: &str) -> anyhow::Result<u64> {
    value
        .parse::<u64>()
        .with_context(|| format!("{key} must be a valid u64"))
}

fn parse_duration_alias_to_seconds(raw: &str) -> anyhow::Result<u64> {
    if let Ok(seconds) = raw.parse::<u64>() {
        return Ok(seconds);
    }

    if let Some(days) = raw.strip_suffix('d') {
        return parse_u64("JWT_EXPIRES_IN", days).map(|d| d * 24 * 60 * 60);
    }

    if let Some(hours) = raw.strip_suffix('h') {
        return parse_u64("JWT_EXPIRES_IN", hours).map(|h| h * 60 * 60);
    }

    if let Some(minutes) = raw.strip_suffix('m') {
        return parse_u64("JWT_EXPIRES_IN", minutes).map(|m| m * 60);
    }

    bail!("JWT_EXPIRES_IN must be numeric seconds or end with d/h/m (e.g. 7d)")
}

fn parse_bool_with_default(key: &str, default: bool) -> anyhow::Result<bool> {
    match std::env::var(key) {
        Ok(value) => value
            .parse::<bool>()
            .with_context(|| format!("{key} must be true or false")),
        Err(_) => Ok(default),
    }
}

fn optional_u64(key: &str) -> anyhow::Result<Option<u64>> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Ok(Some(parse_u64(key, &v)?)),
        _ => Ok(None),
    }
}
