use std::{collections::VecDeque, sync::Arc, time::Duration};

use mongodb::bson::{Document, doc};
use tokio::{sync::RwLock, time::sleep};
use tracing::{error, info, warn};

use crate::{
    config::AppConfig,
    service::{kpi_service::KpiService, ops_service::OpsService},
};

#[derive(Clone)]
pub struct IndexerService {
    http: reqwest::Client,
    rpc_url: String,
    program_id: String,
    batch_size: u64,
    confirmation_depth: u64,
    poll_interval_ms: u64,
    start_slot: u64,
    backfill_end_slot: Option<u64>,
    mongo: mongodb::Client,
    kpi_service: KpiService,
    ops_service: Option<OpsService>,
    runtime: Arc<RwLock<IndexerRuntime>>,
}

#[derive(Clone, Default)]
pub struct IndexerStatusSnapshot {
    pub running: bool,
    pub last_poll_epoch: i64,
    pub last_processed_slot: i64,
    pub last_signature: Option<String>,
    pub backfill_active: bool,
    pub backfill_pending: usize,
}

#[derive(Clone)]
struct BackfillRequest {
    start_slot: u64,
    end_slot: u64,
}

#[derive(Clone, Default)]
struct IndexerRuntime {
    status: IndexerStatusSnapshot,
    queue: VecDeque<BackfillRequest>,
}

impl IndexerService {
    pub fn new(
        config: &AppConfig,
        http: reqwest::Client,
        mongo: mongodb::Client,
        ops_service: Option<OpsService>,
    ) -> Self {
        let kpi_service = KpiService::new(mongo.clone());
        Self {
            http,
            rpc_url: config.chain.rpc_url.clone(),
            program_id: config.chain.program_id.clone(),
            batch_size: config.indexer.batch_size,
            confirmation_depth: config.indexer.confirmation_depth,
            poll_interval_ms: config.indexer.poll_interval_ms,
            start_slot: config.indexer.start_slot,
            backfill_end_slot: config.indexer.backfill_end_slot,
            mongo,
            kpi_service,
            ops_service,
            runtime: Arc::new(RwLock::new(IndexerRuntime::default())),
        }
    }

    pub fn spawn(self) {
        tokio::spawn(async move {
            if let Err(err) = self.run().await {
                error!(error = %err, "indexer worker crashed");
            }
        });
    }

    pub async fn enqueue_backfill(&self, start_slot: u64, end_slot: u64) -> anyhow::Result<()> {
        if start_slot > end_slot {
            anyhow::bail!("start_slot must be <= end_slot");
        }
        let mut rt = self.runtime.write().await;
        rt.queue.push_back(BackfillRequest {
            start_slot,
            end_slot,
        });
        rt.status.backfill_pending = rt.queue.len();
        if let Some(ops) = &self.ops_service {
            ops.set_queue_lag(rt.status.backfill_pending).await;
        }
        Ok(())
    }

    pub async fn status(&self) -> IndexerStatusSnapshot {
        self.runtime.read().await.status.clone()
    }

    async fn run(self) -> anyhow::Result<()> {
        info!("indexer worker started");
        let cursor_name = "program_signatures";
        self.set_running(true).await;

        if let Some((slot, sig)) = self.get_indexer_cursor(cursor_name).await? {
            let mut rt = self.runtime.write().await;
            rt.status.last_processed_slot = slot;
            rt.status.last_signature = sig;
        }

        loop {
            if let Some(job) = self.next_backfill_job().await {
                self.set_backfill_active(true).await;
                let _ = self
                    .run_backfill_range(job.start_slot, job.end_slot)
                    .await
                    .map_err(|e| warn!(error = %e, "backfill failed"));
                self.set_backfill_active(false).await;
            }

            let until = { self.runtime.read().await.status.last_signature.clone() };
            let signatures = self.fetch_signatures(None, until.as_deref()).await?;
            let mut updates = 0u64;
            for entry in signatures.iter() {
                let slot = entry
                    .get("slot")
                    .and_then(|v| v.as_u64())
                    .unwrap_or_default();
                if slot < self.start_slot || slot < self.confirmation_depth {
                    continue;
                }
                if let Some(end) = self.backfill_end_slot {
                    if slot > end {
                        continue;
                    }
                }

                let Some(signature) = entry.get("signature").and_then(|v| v.as_str()) else {
                    continue;
                };

                let payload = self.fetch_transaction(signature).await.unwrap_or_else(
                    |_| serde_json::json!({ "signature": signature, "slot": slot }),
                );

                self.insert_chain_event(signature, slot as i64, "transaction", &payload)
                    .await?;
                self.set_indexer_cursor(cursor_name, slot as i64, Some(signature.to_string()))
                    .await?;
                self.update_status(slot as i64, Some(signature.to_string()))
                    .await;
                updates += 1;
            }

            if updates > 0 {
                self.poll_program_accounts().await?;
                let _ = self.kpi_service.refresh_all().await;
            }

            self.mark_polled().await;
            sleep(Duration::from_millis(self.poll_interval_ms)).await;
        }
    }

    async fn run_backfill_range(&self, start_slot: u64, end_slot: u64) -> anyhow::Result<()> {
        let mut before: Option<String> = None;
        loop {
            let rows = self.fetch_signatures(before.as_deref(), None).await?;
            if rows.is_empty() {
                break;
            }

            let mut hit_lower_bound = false;
            for row in rows.iter() {
                let slot = row.get("slot").and_then(|v| v.as_u64()).unwrap_or(0);
                let Some(signature) = row.get("signature").and_then(|v| v.as_str()) else {
                    continue;
                };

                if slot < start_slot {
                    hit_lower_bound = true;
                    break;
                }
                if slot > end_slot {
                    continue;
                }

                let payload = self.fetch_transaction(signature).await.unwrap_or_else(
                    |_| serde_json::json!({ "signature": signature, "slot": slot }),
                );
                self.insert_chain_event(signature, slot as i64, "backfill_transaction", &payload)
                    .await?;
                self.update_status(slot as i64, Some(signature.to_string()))
                    .await;
            }

            before = rows
                .last()
                .and_then(|s| s.get("signature"))
                .and_then(|v| v.as_str())
                .map(ToString::to_string);

            if hit_lower_bound || before.is_none() {
                break;
            }
        }
        Ok(())
    }

    async fn poll_program_accounts(&self) -> anyhow::Result<()> {
        let current_slot = self.fetch_current_slot().await.unwrap_or_default();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getProgramAccounts",
            "params": [self.program_id, { "encoding": "base64" }]
        });

        let response = self.http.post(&self.rpc_url).json(&body).send().await?;
        let value = response.json::<serde_json::Value>().await?;
        let rows = value
            .get("result")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let col = self
            .mongo
            .database("ticketing_backend")
            .collection::<Document>("account_snapshots");

        for item in rows {
            let Some(pubkey) = item.get("pubkey").and_then(|v| v.as_str()) else {
                continue;
            };
            let account = item
                .get("account")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let lamports = account
                .get("lamports")
                .and_then(|v| v.as_i64())
                .unwrap_or_default();
            let owner = account
                .get("owner")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            col.update_one(
                doc! { "pubkey": pubkey },
                doc! {
                    "$set": {
                        "pubkey": pubkey,
                        "owner": owner,
                        "lamports": lamports,
                        "program_id": &self.program_id,
                        "slot": current_slot as i64,
                        "updated_at_epoch": now_epoch(),
                        "account": mongodb::bson::to_bson(&account)?
                    }
                },
            )
            .upsert(true)
            .await?;
        }

        Ok(())
    }

    async fn fetch_signatures(
        &self,
        before_signature: Option<&str>,
        until_signature: Option<&str>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let mut config = serde_json::json!({ "limit": self.batch_size });
        if let Some(before) = before_signature {
            config["before"] = serde_json::Value::String(before.to_string());
        }
        if let Some(until) = until_signature {
            config["until"] = serde_json::Value::String(until.to_string());
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": [self.program_id, config]
        });

        let response = self.http.post(&self.rpc_url).json(&body).send().await?;
        let value = response.json::<serde_json::Value>().await?;
        if let Some(err) = value.get("error") {
            warn!(error = %err, "getSignaturesForAddress failed");
            return Ok(Vec::new());
        }

        Ok(value
            .get("result")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    async fn fetch_transaction(&self, signature: &str) -> anyhow::Result<serde_json::Value> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [signature, { "encoding": "json", "maxSupportedTransactionVersion": 0 }]
        });

        let response = self.http.post(&self.rpc_url).json(&body).send().await?;
        let value = response.json::<serde_json::Value>().await?;
        Ok(value
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    async fn fetch_current_slot(&self) -> anyhow::Result<u64> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSlot",
            "params": []
        });
        let response = self.http.post(&self.rpc_url).json(&body).send().await?;
        let value = response.json::<serde_json::Value>().await?;
        Ok(value
            .get("result")
            .and_then(|v| v.as_u64())
            .unwrap_or_default())
    }

    async fn get_indexer_cursor(
        &self,
        cursor_name: &str,
    ) -> anyhow::Result<Option<(i64, Option<String>)>> {
        let col = self
            .mongo
            .database("ticketing_backend")
            .collection::<Document>("indexer_cursors");

        let doc = col.find_one(doc! { "cursor_name": cursor_name }).await?;
        Ok(doc.map(|d| {
            (
                d.get_i64("last_processed_slot").unwrap_or_default(),
                d.get_str("last_signature").ok().map(ToString::to_string),
            )
        }))
    }

    async fn set_indexer_cursor(
        &self,
        cursor_name: &str,
        slot: i64,
        signature: Option<String>,
    ) -> anyhow::Result<()> {
        let col = self
            .mongo
            .database("ticketing_backend")
            .collection::<Document>("indexer_cursors");

        col.update_one(
            doc! { "cursor_name": cursor_name },
            doc! {
                "$set": {
                    "cursor_name": cursor_name,
                    "last_processed_slot": slot,
                    "last_signature": signature,
                    "updated_at_epoch": now_epoch()
                }
            },
        )
        .upsert(true)
        .await?;

        Ok(())
    }

    async fn insert_chain_event(
        &self,
        signature: &str,
        slot: i64,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let col = self
            .mongo
            .database("ticketing_backend")
            .collection::<Document>("chain_events");

        col.update_one(
            doc! { "signature": signature },
            doc! {
                "$setOnInsert": {
                    "signature": signature,
                    "slot": slot,
                    "program_id": &self.program_id,
                    "event_type": event_type,
                    "payload": mongodb::bson::to_bson(payload)?,
                    "observed_at_epoch": now_epoch(),
                }
            },
        )
        .upsert(true)
        .await?;

        Ok(())
    }

    async fn set_running(&self, running: bool) {
        let mut rt = self.runtime.write().await;
        rt.status.running = running;
    }

    async fn set_backfill_active(&self, active: bool) {
        let mut rt = self.runtime.write().await;
        rt.status.backfill_active = active;
        rt.status.backfill_pending = rt.queue.len();
        if let Some(ops) = &self.ops_service {
            ops.set_queue_lag(rt.status.backfill_pending).await;
        }
    }

    async fn next_backfill_job(&self) -> Option<BackfillRequest> {
        let mut rt = self.runtime.write().await;
        let job = rt.queue.pop_front();
        rt.status.backfill_pending = rt.queue.len();
        if let Some(ops) = &self.ops_service {
            ops.set_queue_lag(rt.status.backfill_pending).await;
        }
        job
    }

    async fn mark_polled(&self) {
        let mut rt = self.runtime.write().await;
        rt.status.last_poll_epoch = now_epoch();
    }

    async fn update_status(&self, slot: i64, signature: Option<String>) {
        let mut rt = self.runtime.write().await;
        rt.status.last_processed_slot = slot;
        rt.status.last_signature = signature;
    }
}

fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
