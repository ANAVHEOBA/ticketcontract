use mongodb::bson::{Bson, Document, doc};

use crate::error::ApiError;

use super::model::EventRecord;

pub async fn find_event(
    mongo: &Option<mongodb::Client>,
    event_id: &str,
) -> Result<Option<EventRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let filter = doc! { "event_id": event_id };
    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("events");
    let doc = collection
        .find_one(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    doc.map(document_to_event)
        .transpose()
        .map_err(ApiError::map_db_error)
}

pub async fn list_events(
    mongo: &Option<mongodb::Client>,
    organizer_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<EventRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let mut filter = Document::new();
    if let Some(org) = organizer_id {
        filter.insert("organizer_id", org);
    }
    if let Some(value) = status {
        filter.insert("status", value);
    }

    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("events");
    let mut cursor = collection
        .find(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    let mut result = Vec::new();
    while cursor.advance().await.map_err(ApiError::map_db_error)? {
        let doc = cursor
            .deserialize_current()
            .map_err(ApiError::map_db_error)?;
        result.push(document_to_event(doc).map_err(ApiError::map_db_error)?);
    }

    Ok(result)
}

fn document_to_event(doc: Document) -> anyhow::Result<EventRecord> {
    let resale_policy_snapshot = doc
        .get("resale_policy_snapshot")
        .and_then(|b| bson_to_json(b).ok());

    Ok(EventRecord {
        event_id: doc.get_str("event_id")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        name: doc.get_str("name").ok().map(ToString::to_string),
        status: doc.get_str("status").ok().map(ToString::to_string),
        metadata_uri: doc.get_str("metadata_uri").ok().map(ToString::to_string),
        resale_policy_snapshot,
        starts_at_epoch: doc.get_i64("starts_at_epoch").ok().map(|v| v as u64),
        ends_at_epoch: doc.get_i64("ends_at_epoch").ok().map(|v| v as u64),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}

fn bson_to_json(value: &Bson) -> anyhow::Result<serde_json::Value> {
    let serialized = serde_json::to_value(value)?;
    Ok(serialized)
}
