use mongodb::bson::{Document, doc};

use crate::error::ApiError;

use super::model::{ResalePolicyRecommendation, ResalePolicyRecord};

pub async fn find_policy(
    mongo: &Option<mongodb::Client>,
    event_id: &str,
    class_id: Option<&str>,
) -> Result<Option<ResalePolicyRecord>, ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let mut filter = doc! { "event_id": event_id };
    match class_id {
        Some(value) => {
            filter.insert("class_id", value);
        }
        None => {
            filter.insert("class_id", mongodb::bson::Bson::Null);
        }
    }

    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("resale_policies");
    let doc = collection
        .find_one(filter)
        .await
        .map_err(ApiError::map_db_error)?;

    doc.map(document_to_policy)
        .transpose()
        .map_err(ApiError::map_db_error)
}

pub async fn upsert_recommendation(
    mongo: &Option<mongodb::Client>,
    recommendation: &ResalePolicyRecommendation,
) -> Result<(), ApiError> {
    let client = mongo.as_ref().ok_or(ApiError::DatabaseUnavailable)?;

    let filter = doc! { "recommendation_id": &recommendation.recommendation_id };
    let update = doc! {
        "$set": {
            "recommendation_id": &recommendation.recommendation_id,
            "organizer_id": &recommendation.organizer_id,
            "event_id": &recommendation.event_id,
            "class_id": recommendation.class_id.clone(),
            "max_markup_bps": recommendation.max_markup_bps as i32,
            "royalty_bps": recommendation.royalty_bps as i32,
            "confidence": recommendation.confidence,
            "rationale": recommendation.rationale.clone(),
            "updated_at_epoch": recommendation.updated_at_epoch as i64,
        }
    };

    let collection = client
        .database("ticketing_backend")
        .collection::<Document>("resale_policy_recommendations");

    collection
        .update_one(filter, update)
        .upsert(true)
        .await
        .map_err(ApiError::map_db_error)?;

    Ok(())
}

fn document_to_policy(doc: Document) -> anyhow::Result<ResalePolicyRecord> {
    Ok(ResalePolicyRecord {
        policy_id: doc.get_str("policy_id")?.to_string(),
        organizer_id: doc.get_str("organizer_id")?.to_string(),
        event_id: doc.get_str("event_id")?.to_string(),
        class_id: doc.get_str("class_id").ok().map(ToString::to_string),
        max_markup_bps: doc.get_i32("max_markup_bps").ok().map(|v| v as u16),
        royalty_bps: doc.get_i32("royalty_bps").ok().map(|v| v as u16),
        whitelist_enabled: doc.get_bool("whitelist_enabled").ok(),
        blacklist_enabled: doc.get_bool("blacklist_enabled").ok(),
        status: doc.get_str("status").ok().map(ToString::to_string),
        updated_at_epoch: doc.get_i64("updated_at_epoch").ok().map(|v| v as u64),
    })
}
