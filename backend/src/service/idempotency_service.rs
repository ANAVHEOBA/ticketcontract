use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::RwLock;

#[derive(Clone)]
pub struct IdempotencyService {
    ttl_seconds: u64,
    store: Arc<RwLock<HashMap<String, IdempotentEntry>>>,
}

#[derive(Clone)]
struct IdempotentEntry {
    created_at_epoch: u64,
    payload: serde_json::Value,
}

impl IdempotencyService {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            ttl_seconds,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let now = epoch_now();
        let mut write = self.store.write().await;
        write.retain(|_, value| now.saturating_sub(value.created_at_epoch) <= self.ttl_seconds);
        write.get(key).map(|entry| entry.payload.clone())
    }

    pub async fn put(&self, key: String, payload: serde_json::Value) {
        let mut write = self.store.write().await;
        write.insert(
            key,
            IdempotentEntry {
                created_at_epoch: epoch_now(),
                payload,
            },
        );
    }
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
