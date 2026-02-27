use mongodb::bson::{Document, doc};

use crate::error::ApiError;

use super::model::TicketRecord;

pub async fn find_ticket(
    mongo: &Option<mongodb::Client>,
    ticket_id: &str,
) -> Result<Option<TicketRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let filter = doc! { "ticket_id": ticket_id };
    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("tickets");
    let doc = collection
        .find_one(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    doc.map(document_to_ticket)
        .transpose()
        .map_err(ApiError::map_db_error)
}

fn document_to_ticket(doc: Document) -> anyhow::Result<TicketRecord> {
    Ok(TicketRecord {
        ticket_id: doc.get_str("ticket_id")?.to_string(),
        event_id: doc.get_str("event_id")?.to_string(),
        class_id: doc.get_str("class_id")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        owner_wallet: doc.get_str("owner_wallet").ok().map(ToString::to_string),
        status: doc.get_str("status").ok().map(ToString::to_string),
        metadata_uri: doc.get_str("metadata_uri").ok().map(ToString::to_string),
        metadata_version: doc.get_i64("metadata_version").ok().map(|v| v as u64),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}
