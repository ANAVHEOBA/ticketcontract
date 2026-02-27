use mongodb::bson::doc;
use serde::Serialize;

use crate::app::AppState;

#[derive(Debug, Serialize)]
pub struct ReadinessStatus {
    pub ok: bool,
    pub checks: Checks,
}

#[derive(Debug, Serialize)]
pub struct Checks {
    pub database: bool,
    pub solana_rpc: bool,
}

pub async fn compute_readiness(state: &AppState) -> ReadinessStatus {
    let db_ok = check_db(state).await;
    let rpc_ok = check_chain_rpc(state).await;

    ReadinessStatus {
        ok: db_ok && rpc_ok,
        checks: Checks {
            database: db_ok,
            solana_rpc: rpc_ok,
        },
    }
}

async fn check_db(state: &AppState) -> bool {
    let Some(mongo) = &state.mongo else {
        return !state.config.db.db_required;
    };

    let result = mongo
        .database("admin")
        .run_command(doc! { "ping": 1 })
        .await;

    result.is_ok()
}

async fn check_chain_rpc(state: &AppState) -> bool {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getHealth"
    });

    let response = state
        .http
        .post(&state.config.chain.rpc_url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.json::<serde_json::Value>().await;
            match body {
                Ok(json) => {
                    json.get("result") == Some(&serde_json::Value::String("ok".to_string()))
                }
                Err(_) => false,
            }
        }
        _ => false,
    }
}
