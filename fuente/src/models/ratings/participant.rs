use nostro2::notes::NostrNote;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ParticipantRating {
    pub pubkey: String,
    pub trust_score: String,
    pub satisfaction_score: String,
    #[serde(skip)]
    pub participant: crate::models::OrderParticipant,
    #[serde(skip)]
    pub history: Vec<crate::models::TrustRecord>,
    #[serde(skip)]
    pub satisfaction_history: Vec<crate::models::SatisfactionRecord>,
}
impl Into<nostro2::notes::NostrNote> for ParticipantRating {
    fn into(self) -> nostro2::notes::NostrNote {
        let content = serde_json::to_string(&self).unwrap();
        nostro2::notes::NostrNote {
            content,
            kind: crate::models::NOSTR_KIND_PARTICIPANT_RATING,
            pubkey: self.pubkey,
            ..Default::default()
        }
    }
}

impl TryFrom<&NostrNote> for ParticipantRating {
    type Error = anyhow::Error;
    fn try_from(note: &NostrNote) -> Result<Self, Self::Error> {
        if note.kind != crate::models::NOSTR_KIND_PARTICIPANT_RATING {
            return Err(anyhow::anyhow!("Wrong kind"));
        }
        Ok(serde_json::from_str(&note.content)?)
    }
}

impl TryFrom<NostrNote> for ParticipantRating {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        Self::try_from(&note)
    }
}

impl ParticipantRating {
    pub fn add_record(&mut self, record: crate::models::TrustRecord) {
        self.history.push(record);
        self.calculate_trust_score();
    }
    pub fn add_satisfaction_record(&mut self, record: crate::models::SatisfactionRecord) {
        self.satisfaction_history.push(record);
        self.calculate_satisfaction_score();
    }
    pub fn calculate_satisfaction_score(&mut self) {
        let total_weight = self.satisfaction_history.len();
        let total_score = self.satisfaction_history.iter().fold(0.0, |acc, record| {
            acc + record.satisfaction.parse::<f32>().unwrap_or(0.0)
        });
        self.satisfaction_score = if total_weight > 0 {
            (total_score / total_weight as f32).to_string()
        } else {
            "0".to_string()
        };
    }
    fn calculate_trust_score(&mut self) {
        let mut score = self.trust_score.parse::<i32>().unwrap_or(0);
        let total_orders = self.history.len() as i32;
        let mut canceled_orders = 0;
        let mut completed_orders = 0;

        // Loop over each rating record to calculate score
        self.history.iter().for_each(|record| {
            match record.status {
                crate::models::OrderStatus::Completed => completed_orders += 1,
                crate::models::OrderStatus::Canceled => canceled_orders -= 1,
                _ => {} // Other statuses can be added if needed
            }
        });

        // Example: Canceled orders decrease the score, completed orders increase the score
        score += completed_orders; // +1 per completed order
        score -= canceled_orders; // -1 per canceled order

        // Optionally, you could normalize the score based on total orders
        // For example, scaling score based on the total number of orders
        if total_orders > 0 {
            score += total_orders as i32; // +1 per order, just an example of how you might scale the score
        }

        // Now, map the score to a 0-5 range
        // To do this, we need to normalize the score to the [0, 5] range.
        // Example: Let's say the maximum score we want for a user is 10 (this depends on your needs)
        let max_possible_score = total_orders as f32 + completed_orders as f32;
        let min_possible_score = 0.0; // Minimum score, for instance if all orders were canceled.

        // Normalize the score to the 0-5 range
        let normalized_score = if max_possible_score > 0.0 {
            5.0 * (score as f32 - min_possible_score) / (max_possible_score - min_possible_score)
        } else {
            0.0 // In case there's no data, return 0
        };

        // Limit the final score to be between 0 and 5
        self.trust_score = normalized_score.clamp(0.0, 5.0).to_string();
    }
}
