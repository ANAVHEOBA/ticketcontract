use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::model::{AccessProfile, AccessRule, NonceRecord};

#[derive(Clone, Default)]
pub struct NonceStore {
    inner: Arc<RwLock<HashMap<String, NonceRecord>>>,
}

impl NonceStore {
    pub async fn upsert(&self, wallet: String, record: NonceRecord) {
        self.inner.write().await.insert(wallet, record);
    }

    pub async fn take(&self, wallet: &str) -> Option<NonceRecord> {
        self.inner.write().await.remove(wallet)
    }
}

#[derive(Clone, Default)]
pub struct AccessStore {
    inner: Arc<RwLock<HashMap<String, AccessProfile>>>,
}

impl AccessStore {
    pub async fn get(&self, wallet: &str) -> Option<AccessProfile> {
        self.inner.read().await.get(wallet).cloned()
    }

    pub async fn upsert(&self, profile: AccessProfile) {
        self.inner
            .write()
            .await
            .insert(profile.wallet.clone(), profile);
    }

    pub async fn seed_rules(&self, rules: &[AccessRule]) {
        let mut map = self.inner.write().await;
        for rule in rules {
            map.insert(
                rule.wallet.clone(),
                AccessProfile {
                    wallet: rule.wallet.clone(),
                    role: rule.role.clone(),
                    organizer_scopes: rule.organizer_scopes.clone(),
                },
            );
        }
    }
}
