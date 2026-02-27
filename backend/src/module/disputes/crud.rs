use mongodb::bson::Document;

use crate::error::ApiError;

use super::model::DisputeRecord;

pub async fn list_disputes(
    mongo: &Option<mongodb::Client>,
    organizer_id: Option<&str>,
    status: Option<&str>,
    limit: u64,
) -> Result<Vec<DisputeRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let mut filter = Document::new();
    if let Some(org) = organizer_id {
        filter.insert("organizer_id", org);
    }
    if let Some(s) = status {
        filter.insert("status", s);
    }

    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("disputes");
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
        result.push(document_to_dispute(doc).map_err(ApiError::map_db_error)?);
    }

    Ok(result)
}

fn document_to_dispute(doc: Document) -> anyhow::Result<DisputeRecord> {
    Ok(DisputeRecord {
        dispute_id: doc.get_str("dispute_id")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        event_id: doc.get_str("event_id")?.to_string(),
        ticket_id: doc.get_str("ticket_id")?.to_string(),
        status: doc.get_str("status").ok().map(ToString::to_string),
        reason: doc.get_str("reason").ok().map(ToString::to_string),
        chargeback: doc.get_bool("chargeback").ok(),
        created_at_epoch: doc.get_i64("created_at_epoch").ok().map(|v| v as u64),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}
