use mongodb::bson::{Document, doc};

use crate::error::ApiError;

use super::model::FinancingOfferRecord;

pub async fn find_offer(
    mongo: &Option<mongodb::Client>,
    offer_id: &str,
) -> Result<Option<FinancingOfferRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let filter = doc! { "offer_id": offer_id };
    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("financing_offers");
    let doc = collection
        .find_one(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    doc.map(document_to_offer)
        .transpose()
        .map_err(ApiError::map_db_error)
}

fn document_to_offer(doc: Document) -> anyhow::Result<FinancingOfferRecord> {
    Ok(FinancingOfferRecord {
        offer_id: doc.get_str("offer_id")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        event_id: doc.get_str("event_id")?.to_string(),
        financier_wallet: doc
            .get_str("financier_wallet")
            .ok()
            .map(ToString::to_string),
        advance_bps: doc.get_i32("advance_bps").ok().map(|v| v as u16),
        fee_bps: doc.get_i32("fee_bps").ok().map(|v| v as u16),
        cap_amount: doc.get_i64("cap_amount").ok().map(|v| v as u64),
        status: doc.get_str("status").ok().map(ToString::to_string),
        freeze_enabled: doc.get_bool("freeze_enabled").ok(),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}
