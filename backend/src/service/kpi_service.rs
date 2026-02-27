use mongodb::bson::{Document, doc};

#[derive(Clone)]
pub struct KpiService {
    mongo: mongodb::Client,
}

impl KpiService {
    pub fn new(mongo: mongodb::Client) -> Self {
        Self { mongo }
    }

    pub async fn refresh_all(&self) -> anyhow::Result<()> {
        self.refresh_event_sales().await?;
        self.refresh_resale_health().await?;
        self.refresh_financing_cash_position().await?;
        Ok(())
    }

    pub async fn refresh_event_sales(&self) -> anyhow::Result<()> {
        let db = self.mongo.database("ticketing_backend");
        let events = db.collection::<Document>("events");
        let kpi = db.collection::<Document>("kpi_event_sales");

        let mut cursor = events.find(doc! {}).await?;
        while cursor.advance().await? {
            let ev: Document = cursor.deserialize_current()?;
            let Some(event_id) = ev.get_str("event_id").ok() else {
                continue;
            };
            let organizer_id = ev.get_str("organizer_id").unwrap_or_default().to_string();

            let tickets_col = db.collection::<Document>("tickets");
            let tickets_count = tickets_col
                .count_documents(doc! { "event_id": event_id })
                .await? as i64;
            let sold = tickets_col
                .count_documents(doc! { "event_id": event_id, "status": "sold" })
                .await? as i64;
            let checked_in = tickets_col
                .count_documents(doc! { "event_id": event_id, "status": "checked_in" })
                .await? as i64;

            kpi.update_one(
                doc! { "event_id": event_id },
                doc! {
                    "$set": {
                        "event_id": event_id,
                        "organizer_id": organizer_id,
                        "tickets_count": tickets_count,
                        "tickets_sold": sold,
                        "tickets_checked_in": checked_in,
                        "updated_at_epoch": now_epoch(),
                    }
                },
            )
            .upsert(true)
            .await?;
        }

        Ok(())
    }

    pub async fn refresh_resale_health(&self) -> anyhow::Result<()> {
        let db = self.mongo.database("ticketing_backend");
        let listings = db.collection::<Document>("listings");
        let kpi = db.collection::<Document>("kpi_resale_health");

        let pipeline = vec![doc! {
            "$group": {
                "_id": "$event_id",
                "listings_total": { "$sum": 1 },
                "listings_active": { "$sum": { "$cond": [{ "$eq": ["$status", "active"] }, 1, 0] } },
                "listings_sold": { "$sum": { "$cond": [{ "$eq": ["$status", "sold"] }, 1, 0] } },
                "avg_ask_price": { "$avg": "$ask_price" }
            }
        }];

        let mut cursor = listings.aggregate(pipeline).await?;
        while cursor.advance().await? {
            let row: Document = cursor.deserialize_current()?;
            let Some(event_id) = row.get_str("_id").ok() else {
                continue;
            };

            kpi.update_one(
                doc! { "event_id": event_id },
                doc! {
                    "$set": {
                        "event_id": event_id,
                        "listings_total": row.get_i64("listings_total").unwrap_or(0),
                        "listings_active": row.get_i64("listings_active").unwrap_or(0),
                        "listings_sold": row.get_i64("listings_sold").unwrap_or(0),
                        "avg_ask_price": row.get("avg_ask_price").cloned().unwrap_or(mongodb::bson::Bson::Null),
                        "updated_at_epoch": now_epoch(),
                    }
                },
            )
            .upsert(true)
            .await?;
        }

        Ok(())
    }

    pub async fn refresh_financing_cash_position(&self) -> anyhow::Result<()> {
        let db = self.mongo.database("ticketing_backend");
        let financing = db.collection::<Document>("financing");
        let kpi = db.collection::<Document>("kpi_financing_cash_position");

        let mut cursor = financing.find(doc! {}).await?;
        while cursor.advance().await? {
            let offer: Document = cursor.deserialize_current()?;
            let organizer_id = offer
                .get_str("organizer_id")
                .unwrap_or_default()
                .to_string();
            let event_id = offer.get_str("event_id").unwrap_or_default().to_string();
            if organizer_id.is_empty() || event_id.is_empty() {
                continue;
            }

            let offers_count = financing
                .count_documents(doc! { "organizer_id": &organizer_id, "event_id": &event_id })
                .await? as i64;

            let disbursements = db.collection::<Document>("disbursements");
            let mut disb_cursor = disbursements
                .find(doc! { "organizer_id": &organizer_id, "event_id": &event_id })
                .await?;

            let mut disbursed_amount: i64 = 0;
            while disb_cursor.advance().await? {
                let row: Document = disb_cursor.deserialize_current()?;
                if row.get_str("status").ok() == Some("disbursed") {
                    disbursed_amount += row.get_i64("amount").unwrap_or(0);
                }
            }

            kpi.update_one(
                doc! { "organizer_id": &organizer_id, "event_id": &event_id },
                doc! {
                    "$set": {
                        "organizer_id": organizer_id,
                        "event_id": event_id,
                        "offers_count": offers_count,
                        "disbursed_amount": disbursed_amount,
                        "updated_at_epoch": now_epoch(),
                    }
                },
            )
            .upsert(true)
            .await?;
        }

        Ok(())
    }

    pub async fn get_event_sales(&self, event_id: &str) -> anyhow::Result<Option<Document>> {
        Ok(self
            .mongo
            .database("ticketing_backend")
            .collection::<Document>("kpi_event_sales")
            .find_one(doc! { "event_id": event_id })
            .await?)
    }

    pub async fn get_resale_health(&self, event_id: &str) -> anyhow::Result<Option<Document>> {
        Ok(self
            .mongo
            .database("ticketing_backend")
            .collection::<Document>("kpi_resale_health")
            .find_one(doc! { "event_id": event_id })
            .await?)
    }

    pub async fn get_financing_cash_position(
        &self,
        organizer_id: &str,
        event_id: &str,
    ) -> anyhow::Result<Option<Document>> {
        Ok(self
            .mongo
            .database("ticketing_backend")
            .collection::<Document>("kpi_financing_cash_position")
            .find_one(doc! { "organizer_id": organizer_id, "event_id": event_id })
            .await?)
    }
}

fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
