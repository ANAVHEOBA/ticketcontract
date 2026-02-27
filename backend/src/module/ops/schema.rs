use serde::{Deserialize, Serialize};

use crate::service::ops_service::{AlertSnapshot, MetricsSnapshot};

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub metrics: MetricsSnapshot,
}

#[derive(Debug, Serialize)]
pub struct AlertsResponse {
    pub alerts: Vec<AlertSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogsResponse {
    pub logs: Vec<serde_json::Value>,
}

fn default_limit() -> u64 {
    100
}
