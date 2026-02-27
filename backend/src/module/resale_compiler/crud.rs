use mongodb::bson::doc;

use crate::error::ApiError;

use super::model::ResaleHistoryInputs;

pub async fn load_history_inputs(
    mongo: &mongodb::Client,
    organizer_id: &str,
    event_id: &str,
    class_id: Option<&str>,
) -> Result<ResaleHistoryInputs, ApiError> {
    let db = mongo.database("ticketing_backend");
    let listings = db.collection::<mongodb::bson::Document>("listings");

    let mut base_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
    };
    if let Some(cid) = class_id {
        base_filter.insert("class_id", cid);
    }

    let listed_count = listings
        .count_documents(base_filter.clone())
        .await
        .map_err(ApiError::map_db_error)?;

    let mut sold_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
        "status": { "$in": ["sold", "completed"] },
    };
    if let Some(cid) = class_id {
        sold_filter.insert("class_id", cid);
    }

    let mut cancelled_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
        "status": { "$in": ["cancelled", "expired"] },
    };
    if let Some(cid) = class_id {
        cancelled_filter.insert("class_id", cid);
    }

    let sold_count = listings
        .count_documents(sold_filter)
        .await
        .map_err(ApiError::map_db_error)?;
    let cancelled_count = listings
        .count_documents(cancelled_filter)
        .await
        .map_err(ApiError::map_db_error)?;

    Ok(ResaleHistoryInputs {
        organizer_id: organizer_id.to_string(),
        event_id: event_id.to_string(),
        class_id: class_id.map(ToString::to_string),
        listed_count,
        sold_count,
        cancelled_count,
    })
}
