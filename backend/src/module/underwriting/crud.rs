use mongodb::bson::doc;

use crate::error::ApiError;

use super::model::UnderwritingHistoryMetrics;

pub async fn load_history_metrics(
    mongo: &mongodb::Client,
    organizer_id: &str,
    event_id: &str,
) -> Result<UnderwritingHistoryMetrics, ApiError> {
    let db = mongo.database("ticketing_backend");
    let tickets = db.collection::<mongodb::bson::Document>("tickets");
    let disputes = db.collection::<mongodb::bson::Document>("disputes");
    let listings = db.collection::<mongodb::bson::Document>("listings");
    let trust = db.collection::<mongodb::bson::Document>("trust_signals");

    let sold_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
        "status": { "$in": ["sold", "checked_in", "active"] }
    };
    let checked_in_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
        "status": "checked_in"
    };
    let refunded_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
        "status": "refunded"
    };
    let disputes_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
    };
    let chargeback_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
        "chargeback": true
    };
    let listed_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
    };
    let completed_filter = doc! {
        "organizer_id": organizer_id,
        "event_id": event_id,
        "status": { "$in": ["sold", "completed"] }
    };

    let primary_sales_count = tickets
        .count_documents(sold_filter)
        .await
        .map_err(ApiError::map_db_error)?;
    let checked_in_count = tickets
        .count_documents(checked_in_filter)
        .await
        .map_err(ApiError::map_db_error)?;
    let refunded_count = tickets
        .count_documents(refunded_filter)
        .await
        .map_err(ApiError::map_db_error)?;
    let disputes_count = disputes
        .count_documents(disputes_filter)
        .await
        .map_err(ApiError::map_db_error)?;
    let chargebacks_count = disputes
        .count_documents(chargeback_filter)
        .await
        .map_err(ApiError::map_db_error)?;
    let resale_listed_count = listings
        .count_documents(listed_filter)
        .await
        .map_err(ApiError::map_db_error)?;
    let resale_completed_count = listings
        .count_documents(completed_filter)
        .await
        .map_err(ApiError::map_db_error)?;

    let mut trust_signal_total_delta = 0i64;
    let mut cursor = trust
        .find(doc! {
            "organizer_id": organizer_id,
            "event_id": event_id,
        })
        .await
        .map_err(ApiError::map_db_error)?;

    while cursor.advance().await.map_err(ApiError::map_db_error)? {
        let doc = cursor
            .deserialize_current()
            .map_err(ApiError::map_db_error)?;
        trust_signal_total_delta += doc.get_i32("score_delta").unwrap_or(0) as i64;
    }

    Ok(UnderwritingHistoryMetrics {
        organizer_id: organizer_id.to_string(),
        event_id: event_id.to_string(),
        primary_sales_count,
        checked_in_count,
        refunded_count,
        disputes_count,
        chargebacks_count,
        resale_listed_count,
        resale_completed_count,
        trust_signal_total_delta,
    })
}
