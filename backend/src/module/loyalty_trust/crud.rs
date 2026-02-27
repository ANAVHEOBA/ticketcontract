use mongodb::bson::Document;

use crate::error::ApiError;

use super::model::{LoyaltyLedgerRecord, TrustSignalRecord};

pub async fn list_loyalty(
    mongo: &Option<mongodb::Client>,
    wallet: &str,
    organizer_id: Option<&str>,
) -> Result<Vec<LoyaltyLedgerRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let mut filter = Document::new();
    filter.insert("wallet", wallet);
    if let Some(org) = organizer_id {
        filter.insert("organizer_id", org);
    }

    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("loyalty_ledger");
    let mut cursor = collection
        .find(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    let mut result = Vec::new();
    while cursor.advance().await.map_err(ApiError::map_db_error)? {
        let doc: Document = cursor
            .deserialize_current()
            .map_err(ApiError::map_db_error)?;
        result.push(document_to_loyalty(doc).map_err(ApiError::map_db_error)?);
    }

    Ok(result)
}

pub async fn list_trust_signals(
    mongo: &Option<mongodb::Client>,
    wallet: Option<&str>,
    organizer_id: Option<&str>,
    event_id: Option<&str>,
    limit: u64,
) -> Result<Vec<TrustSignalRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let mut filter = Document::new();
    if let Some(w) = wallet {
        filter.insert("wallet", w);
    }
    if let Some(org) = organizer_id {
        filter.insert("organizer_id", org);
    }
    if let Some(ev) = event_id {
        filter.insert("event_id", ev);
    }

    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("trust_signals");
    let mut cursor = collection
        .find(filter)
        .limit(limit as i64)
        .await
        .map_err(ApiError::map_db_error)?;

    let mut result = Vec::new();
    while cursor.advance().await.map_err(ApiError::map_db_error)? {
        let doc: Document = cursor
            .deserialize_current()
            .map_err(ApiError::map_db_error)?;
        result.push(document_to_trust(doc).map_err(ApiError::map_db_error)?);
    }

    Ok(result)
}

fn document_to_loyalty(doc: Document) -> anyhow::Result<LoyaltyLedgerRecord> {
    Ok(LoyaltyLedgerRecord {
        wallet: doc.get_str("wallet")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        event_id: doc.get_str("event_id").ok().map(ToString::to_string),
        points_balance: doc.get_i64("points_balance").ok(),
        points_earned: doc.get_i64("points_earned").ok(),
        points_redeemed: doc.get_i64("points_redeemed").ok(),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}

fn document_to_trust(doc: Document) -> anyhow::Result<TrustSignalRecord> {
    Ok(TrustSignalRecord {
        signal_id: doc.get_str("signal_id")?.to_string(),
        wallet: doc.get_str("wallet")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        event_id: doc.get_str("event_id")?.to_string(),
        signal_type: doc.get_str("signal_type")?.to_string(),
        schema_version: doc.get_i32("schema_version").ok().map(|v| v as u32),
        score_delta: doc.get_i32("score_delta").ok(),
        created_at_epoch: doc.get_i64("created_at_epoch").ok().map(|v| v as u64),
    })
}
