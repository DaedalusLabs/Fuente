#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SatisfactionRecord {
    pub order_id: String,
    pub participant: crate::models::OrderParticipant,
    pub satisfaction: String,
    #[serde(skip)]
    pub rater_pubkey: String,
}
impl TryFrom<&nostro2::notes::NostrNote> for SatisfactionRecord {
    type Error = anyhow::Error;
    fn try_from(note: &nostro2::notes::NostrNote) -> Result<Self, Self::Error> {
        let mut new_record = serde_json::from_str::<SatisfactionRecord>(&note.content)?;
        new_record.rater_pubkey = note.pubkey.clone();
        Ok(new_record)
    }
}
impl TryFrom<nostro2::notes::NostrNote> for SatisfactionRecord {
    type Error = anyhow::Error;
    fn try_from(note: nostro2::notes::NostrNote) -> Result<Self, Self::Error> {
        let mut new_record = serde_json::from_str::<SatisfactionRecord>(&note.content)?;
        new_record.rater_pubkey = note.pubkey;
        Ok(new_record)
    }
}
impl Into<nostro2::notes::NostrNote> for SatisfactionRecord {
    fn into(self) -> nostro2::notes::NostrNote {
        let content = serde_json::to_string(&self).unwrap();
        nostro2::notes::NostrNote {
            content,
            kind: crate::models::NOSTR_KIND_SATISFACTION_EVENT,
            pubkey: self.rater_pubkey,
            ..Default::default()
        }
    }
}
