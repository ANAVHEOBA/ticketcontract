use mongodb::bson::{Document, doc};

use crate::error::ApiError;

use super::model::ListingRecord;

pub async fn find_listing(
    mongo: &Option<mongodb::Client>,
    listing_id: &str,
) -> Result<Option<ListingRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let filter = doc! { "listing_id": listing_id };
    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("listings");
    let doc = collection
        .find_one(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    doc.map(document_to_listing)
        .transpose()
        .map_err(ApiError::map_db_error)
}

fn document_to_listing(doc: Document) -> anyhow::Result<ListingRecord> {
    Ok(ListingRecord {
        listing_id: doc.get_str("listing_id")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        event_id: doc.get_str("event_id")?.to_string(),
        class_id: doc.get_str("class_id")?.to_string(),
        ticket_id: doc.get_str("ticket_id")?.to_string(),
        seller_wallet: doc.get_str("seller_wallet").ok().map(ToString::to_string),
        ask_price: doc.get_i64("ask_price").ok().map(|v| v as u64),
        status: doc.get_str("status").ok().map(ToString::to_string),
        expires_at_epoch: doc.get_i64("expires_at_epoch").ok().map(|v| v as u64),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}
