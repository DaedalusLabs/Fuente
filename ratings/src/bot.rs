use crate::manager::RatingManager;

pub struct RatingBot {
    pub keys: nostro2::keypair::NostrKeypair,
    pub broadcaster: tokio::sync::broadcast::Sender<nostro2::relays::WebSocketMessage>,
    pub ratings: crate::manager::LiveRatingMap,
}
impl RatingBot {
    fn order_filter(&self) -> nostro2::relays::NostrSubscription {
        nostro2::relays::NostrSubscription {
            kinds: Some(vec![fuente::models::NOSTR_KIND_ORDER_STATE]),
            ..Default::default()
        }
    }
    fn satisfaction_filter(&self) -> nostro2::relays::NostrSubscription {
        let mut filter = nostro2::relays::NostrSubscription {
            kinds: Some(vec![fuente::models::NOSTR_KIND_SATISFACTION_EVENT]),
            ..Default::default()
        };
        filter.add_tag("#p", self.keys.public_key().as_str());
        filter
    }
    pub async fn listen_to_relays(
        &self,
        mut relay_pool: nostro2::relays::NostrRelayPool,
    ) -> anyhow::Result<()> {
        relay_pool.broadcaster.send(self.order_filter().into())?;
        relay_pool
            .broadcaster
            .send(self.satisfaction_filter().into())?;
        while let Some(signed_note) = relay_pool.reader.recv().await {
            if let nostro2::relays::RelayEvent::NewNote((_, _, note)) =
                signed_note.1
            {
                if let Err(e) = self.process_note(note).await {
                    tracing::error!("Error processing note: {:?}", e);
                }
            }
        }
        Err(anyhow::anyhow!("Relay pool closed"))
    }
    async fn process_note(&self, note: nostro2::notes::NostrNote) -> anyhow::Result<()> {
        match note.kind {
            fuente::models::NOSTR_KIND_ORDER_STATE => {
                if let Ok(record) = fuente::models::TrustRecord::try_from(&note) {
                    let new_rating = self.ratings.add_trust_record(record).await?;
                    tracing::info!("Updated rating for {}", new_rating.pubkey);
                    let mut new_note: nostro2::notes::NostrNote = new_rating.into();
                    new_note.tags.add_parameter_tag("rating");
                    self.keys.sign_nostr_event(&mut new_note);
                    self.broadcaster.send(new_note.into())?;
                };
            }
            fuente::models::NOSTR_KIND_SATISFACTION_EVENT => {
                if let Ok(record) = fuente::models::SatisfactionRecord::try_from(&note) {
                    let new_ratings = self.ratings.add_satisfaction_record(record).await?;
                    for rating in new_ratings {
                        tracing::info!("Updated rating for {}", rating.pubkey);
                        let mut new_note: nostro2::notes::NostrNote = rating.into();
                        new_note.tags.add_parameter_tag("rating");
                        self.keys.sign_nostr_event(&mut new_note);
                        self.broadcaster.send(new_note.into())?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
