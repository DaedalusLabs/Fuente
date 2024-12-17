type RatingMap = std::collections::HashMap<String, fuente::models::ParticipantRating>;

pub type LiveRatingMap = std::sync::Arc<tokio::sync::RwLock<RatingMap>>;

pub trait RatingManager {
    async fn add_trust_record(
        &self,
        record: fuente::models::TrustRecord,
    ) -> anyhow::Result<fuente::models::ParticipantRating>;
    async fn add_satisfaction_record(
        &self,
        record: fuente::models::SatisfactionRecord,
    ) -> anyhow::Result<Vec<fuente::models::ParticipantRating>>;
}
impl RatingManager for LiveRatingMap {
    async fn add_trust_record(
        &self,
        record: fuente::models::TrustRecord,
    ) -> anyhow::Result<fuente::models::ParticipantRating> {
        let mut map = self.write().await;
        let participant =
            map.entry(record.pubkey.clone())
                .or_insert_with(|| fuente::models::ParticipantRating {
                    pubkey: record.pubkey.clone(),
                    participant: record.participant.clone(),
                    trust_score: "0".to_string(),
                    satisfaction_score: "0".to_string(),
                    history: vec![],
                    satisfaction_history: vec![],
                });
        participant.add_record(record);
        Ok(participant.clone())
    }
    async fn add_satisfaction_record(
        &self,
        record: fuente::models::SatisfactionRecord,
    ) -> anyhow::Result<Vec<fuente::models::ParticipantRating>> {
        let mut map = self.write().await;
        let new_ratings = map
            .values_mut()
            .filter(|rating| {
                rating.history.iter().any(|r| r.order_id == record.order_id)
                    && rating.participant != record.participant
            })
            .map(|rating| {
                rating.add_satisfaction_record(record.clone());
                rating.clone()
            })
            .collect();
        Ok(new_ratings)
    }
}
