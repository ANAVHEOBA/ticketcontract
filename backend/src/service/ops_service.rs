use std::time::{SystemTime, UNIX_EPOCH};

use mongodb::bson::{Document, doc};
use tokio::sync::RwLock;

use crate::{error::ApiError, service::indexer_service::IndexerStatusSnapshot};

#[derive(Clone)]
pub struct OpsService {
    mongo: Option<mongodb::Client>,
    metrics: std::sync::Arc<RwLock<OpsMetrics>>,
    thresholds: OpsThresholds,
}

#[derive(Clone)]
pub struct OpsThresholds {
    pub failed_submissions_threshold: u64,
    pub indexer_lag_threshold_seconds: u64,
    pub queue_lag_threshold: usize,
}

#[derive(Default)]
struct OpsMetrics {
    tx_attempts: u64,
    tx_success: u64,
    tx_failed: u64,
    confirmation_latency_total_ms: u128,
    confirmation_latency_samples: u64,
    queue_lag: usize,
    protocol_paused: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub tx_attempts: u64,
    pub tx_success: u64,
    pub tx_failed: u64,
    pub tx_success_rate: f64,
    pub avg_confirmation_latency_ms: f64,
    pub queue_lag: usize,
    pub protocol_paused: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AlertSnapshot {
    pub active: bool,
    pub code: String,
    pub severity: String,
    pub message: String,
}

impl OpsService {
    pub fn new(mongo: Option<mongodb::Client>) -> Self {
        Self {
            mongo,
            metrics: std::sync::Arc::new(RwLock::new(OpsMetrics::default())),
            thresholds: OpsThresholds {
                failed_submissions_threshold: env_u64("ALERT_FAILED_SUBMISSIONS_THRESHOLD", 20),
                indexer_lag_threshold_seconds: env_u64("ALERT_INDEXER_LAG_THRESHOLD_SECONDS", 120),
                queue_lag_threshold: env_u64("ALERT_QUEUE_LAG_THRESHOLD", 200) as usize,
            },
        }
    }

    pub async fn record_tx_result(&self, success: bool, confirmation_latency_ms: Option<u128>) {
        let mut m = self.metrics.write().await;
        m.tx_attempts = m.tx_attempts.saturating_add(1);
        if success {
            m.tx_success = m.tx_success.saturating_add(1);
        } else {
            m.tx_failed = m.tx_failed.saturating_add(1);
        }
        if let Some(ms) = confirmation_latency_ms {
            m.confirmation_latency_total_ms = m.confirmation_latency_total_ms.saturating_add(ms);
            m.confirmation_latency_samples = m.confirmation_latency_samples.saturating_add(1);
        }
    }

    pub async fn set_queue_lag(&self, lag: usize) {
        self.metrics.write().await.queue_lag = lag;
    }

    pub async fn set_protocol_paused(&self, paused: bool) {
        self.metrics.write().await.protocol_paused = paused;
    }

    pub async fn metrics_snapshot(&self) -> MetricsSnapshot {
        let m = self.metrics.read().await;
        let tx_success_rate = if m.tx_attempts == 0 {
            0.0
        } else {
            m.tx_success as f64 / m.tx_attempts as f64
        };
        let avg_confirmation_latency_ms = if m.confirmation_latency_samples == 0 {
            0.0
        } else {
            m.confirmation_latency_total_ms as f64 / m.confirmation_latency_samples as f64
        };

        MetricsSnapshot {
            tx_attempts: m.tx_attempts,
            tx_success: m.tx_success,
            tx_failed: m.tx_failed,
            tx_success_rate,
            avg_confirmation_latency_ms,
            queue_lag: m.queue_lag,
            protocol_paused: m.protocol_paused,
        }
    }

    pub async fn alerts_snapshot(
        &self,
        indexer: Option<IndexerStatusSnapshot>,
    ) -> Vec<AlertSnapshot> {
        let m = self.metrics.read().await;
        let mut out = Vec::new();

        if m.tx_failed >= self.thresholds.failed_submissions_threshold {
            out.push(AlertSnapshot {
                active: true,
                code: "FAILED_SUBMISSIONS".to_string(),
                severity: "critical".to_string(),
                message: format!(
                    "failed submissions {} exceed threshold {}",
                    m.tx_failed, self.thresholds.failed_submissions_threshold
                ),
            });
        }

        if m.queue_lag >= self.thresholds.queue_lag_threshold {
            out.push(AlertSnapshot {
                active: true,
                code: "QUEUE_LAG".to_string(),
                severity: "warning".to_string(),
                message: format!(
                    "queue lag {} exceed threshold {}",
                    m.queue_lag, self.thresholds.queue_lag_threshold
                ),
            });
        }

        if let Some(indexer_status) = indexer {
            let now = now_epoch() as i64;
            let lag_seconds = now.saturating_sub(indexer_status.last_poll_epoch).max(0) as u64;
            if indexer_status.running
                && lag_seconds >= self.thresholds.indexer_lag_threshold_seconds
            {
                out.push(AlertSnapshot {
                    active: true,
                    code: "INDEXER_LAG".to_string(),
                    severity: "critical".to_string(),
                    message: format!(
                        "indexer lag {}s exceed threshold {}s",
                        lag_seconds, self.thresholds.indexer_lag_threshold_seconds
                    ),
                });
            }
        }

        if m.protocol_paused {
            out.push(AlertSnapshot {
                active: true,
                code: "PROTOCOL_PAUSED".to_string(),
                severity: "warning".to_string(),
                message: "protocol is currently paused".to_string(),
            });
        }

        out
    }

    pub async fn audit(
        &self,
        actor_wallet: &str,
        actor_role: &str,
        action: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), ApiError> {
        let Some(mongo) = &self.mongo else {
            return Ok(());
        };

        let col = mongo
            .database("ticketing_backend")
            .collection::<Document>("admin_audit_logs");
        col.insert_one(doc! {
            "actor_wallet": actor_wallet,
            "actor_role": actor_role,
            "action": action,
            "metadata": metadata.map(|m| mongodb::bson::to_bson(&m).unwrap_or(mongodb::bson::Bson::Null)),
            "created_at_epoch": now_epoch() as i64,
        })
        .await
        .map_err(ApiError::map_db_error)?;

        Ok(())
    }

    pub async fn list_audit_logs(&self, limit: u64) -> Result<Vec<serde_json::Value>, ApiError> {
        let Some(mongo) = &self.mongo else {
            return Err(ApiError::DatabaseUnavailable);
        };
        let col = mongo
            .database("ticketing_backend")
            .collection::<Document>("admin_audit_logs");

        let mut cursor = col
            .find(doc! {})
            .sort(doc! { "created_at_epoch": -1 })
            .limit(limit as i64)
            .await
            .map_err(ApiError::map_db_error)?;

        let mut out = Vec::new();
        while cursor.advance().await.map_err(ApiError::map_db_error)? {
            let doc = cursor
                .deserialize_current()
                .map_err(ApiError::map_db_error)?;
            let json =
                mongodb::bson::from_bson::<serde_json::Value>(mongodb::bson::Bson::Document(doc))
                    .map_err(ApiError::map_db_error)?;
            out.push(json);
        }
        Ok(out)
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
