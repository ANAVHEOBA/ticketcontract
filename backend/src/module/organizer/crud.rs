use mongodb::bson::{Document, doc};

use crate::error::ApiError;

use super::model::OrganizerRecord;

pub async fn find_organizer(
    mongo: &Option<mongodb::Client>,
    organizer_id: &str,
) -> Result<Option<OrganizerRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let filter = doc! { "organizer_id": organizer_id };
    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("organizers");
    let doc = collection
        .find_one(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    doc.map(document_to_organizer)
        .transpose()
        .map_err(ApiError::map_db_error)
}

fn document_to_organizer(doc: Document) -> anyhow::Result<OrganizerRecord> {
    Ok(OrganizerRecord {
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        owner_wallet: doc.get_str("owner_wallet").ok().map(ToString::to_string),
        metadata_uri: doc.get_str("metadata_uri").ok().map(ToString::to_string),
        payout_wallet: doc.get_str("payout_wallet").ok().map(ToString::to_string),
        status: doc.get_str("status").ok().map(ToString::to_string),
        compliance_flags: doc.get_array("compliance_flags").ok().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        }),
        operators: doc.get_array("operators").ok().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        }),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}
