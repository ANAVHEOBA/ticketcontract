use mongodb::bson::{Document, doc};

use crate::error::ApiError;

use super::model::{TicketClassAnalytics, TicketClassRecord};

pub async fn find_ticket_class(
    mongo: &Option<mongodb::Client>,
    class_id: &str,
) -> Result<Option<TicketClassRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let filter = doc! { "class_id": class_id };
    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("ticket_classes");
    let doc = collection
        .find_one(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    doc.map(document_to_class)
        .transpose()
        .map_err(ApiError::map_db_error)
}

pub async fn list_ticket_classes(
    mongo: &Option<mongodb::Client>,
    organizer_id: Option<&str>,
    event_id: Option<&str>,
) -> Result<Vec<TicketClassRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let mut filter = Document::new();
    if let Some(org) = organizer_id {
        filter.insert("organizer_id", org);
    }
    if let Some(event) = event_id {
        filter.insert("event_id", event);
    }

    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("ticket_classes");
    let mut cursor = collection
        .find(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    let mut result = Vec::new();
    while cursor.advance().await.map_err(ApiError::map_db_error)? {
        let doc = cursor
            .deserialize_current()
            .map_err(ApiError::map_db_error)?;
        result.push(document_to_class(doc).map_err(ApiError::map_db_error)?);
    }

    Ok(result)
}

pub async fn class_analytics(
    mongo: &Option<mongodb::Client>,
    class_id: &str,
) -> Result<Option<TicketClassAnalytics>, ApiError> {
    let class = find_ticket_class(mongo, class_id).await?;
    let Some(class) = class else {
        return Ok(None);
    };

    let supply_total = class.supply_total.unwrap_or(0);
    let supply_reserved = class.supply_reserved.unwrap_or(0);
    let supply_sold = class.supply_sold.unwrap_or(0);
    let remaining = supply_total.saturating_sub(supply_reserved.saturating_add(supply_sold));
    let pacing_ratio = if supply_total == 0 {
        0.0
    } else {
        supply_sold as f64 / supply_total as f64
    };

    Ok(Some(TicketClassAnalytics {
        class_id: class.class_id,
        event_id: class.event_id,
        organizer_id: class.organizer_id,
        supply_total,
        supply_reserved,
        supply_sold,
        supply_remaining: remaining,
        pacing_ratio,
    }))
}

fn document_to_class(doc: Document) -> anyhow::Result<TicketClassRecord> {
    Ok(TicketClassRecord {
        class_id: doc.get_str("class_id")?.to_string(),
        event_id: doc.get_str("event_id")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        name: doc.get_str("name").ok().map(ToString::to_string),
        status: doc.get_str("status").ok().map(ToString::to_string),
        supply_total: doc.get_i64("supply_total").ok().map(|v| v as u64),
        supply_reserved: doc.get_i64("supply_reserved").ok().map(|v| v as u64),
        supply_sold: doc.get_i64("supply_sold").ok().map(|v| v as u64),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}
