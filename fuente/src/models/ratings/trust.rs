#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TrustRecord {
    pub order_id: String,
    pub pubkey: String,
    pub participant: crate::models::OrderParticipant,
    pub status: crate::models::OrderStatus,
    pub payment: crate::models::OrderPaymentStatus,
}
impl TryFrom<nostro2::notes::NostrNote> for TrustRecord {
    type Error = anyhow::Error;
    fn try_from(note: nostro2::notes::NostrNote) -> Result<Self, Self::Error> {
        let status_tag = note
            .tags
            .find_tags(nostro2::notes::NostrTag::Custom("status"));
        let order_status_str = status_tag.get(0).ok_or_else(|| {
            anyhow::anyhow!("No status tag found in note {}", note.id.as_ref().unwrap())
        })?;
        let status = crate::models::OrderStatus::try_from(order_status_str)?;
        let payment_status_str = status_tag.get(1).ok_or_else(|| {
            anyhow::anyhow!("No payment tag found in note {}", note.id.as_ref().unwrap())
        })?;
        let payment = crate::models::OrderPaymentStatus::try_from(payment_status_str)?;
        let participant_str = note.tags.find_first_parameter().ok_or_else(|| {
            anyhow::anyhow!("No participant found in note {}", note.id.as_ref().unwrap())
        })?;
        let (participant_str, order_id) = participant_str.split_once('-').ok_or_else(|| {
            anyhow::anyhow!("No participant found in note {}", note.id.as_ref().unwrap())
        })?;
        let participant = crate::models::OrderParticipant::try_from(participant_str)?;
        let pubkey = note.tags.find_first_tagged_pubkey().ok_or_else(|| {
            anyhow::anyhow!("No pubkey found in note {}", note.id.as_ref().unwrap())
        })?;
        Ok(TrustRecord {
            order_id: order_id.to_string(),
            pubkey,
            participant,
            status,
            payment,
        })
    }
}
impl TryFrom<&nostro2::notes::NostrNote> for TrustRecord {
    type Error = anyhow::Error;
    fn try_from(note: &nostro2::notes::NostrNote) -> Result<Self, Self::Error> {
        let status_tag = note
            .tags
            .find_tags(nostro2::notes::NostrTag::Custom("status"));
        let order_status_str = status_tag.get(0).ok_or_else(|| {
            anyhow::anyhow!("No status tag found in note {}", note.id.as_ref().unwrap())
        })?;
        let status = crate::models::OrderStatus::try_from(order_status_str)?;
        let payment_status_str = status_tag.get(1).ok_or_else(|| {
            anyhow::anyhow!("No payment tag found in note {}", note.id.as_ref().unwrap())
        })?;
        let payment = crate::models::OrderPaymentStatus::try_from(payment_status_str)?;
        let participant_str = note.tags.find_first_parameter().ok_or_else(|| {
            anyhow::anyhow!("No participant found in note {}", note.id.as_ref().unwrap())
        })?;
        let (participant_str, order_id) = participant_str.split_once('-').ok_or_else(|| {
            anyhow::anyhow!("No participant found in note {}", note.id.as_ref().unwrap())
        })?;
        let participant = crate::models::OrderParticipant::try_from(participant_str)?;
        let pubkey = note.tags.find_first_tagged_pubkey().ok_or_else(|| {
            anyhow::anyhow!("No pubkey found in note {}", note.id.as_ref().unwrap())
        })?;
        Ok(TrustRecord {
            order_id: order_id.to_string(),
            pubkey,
            participant,
            status,
            payment,
        })
    }
}
impl std::fmt::Display for TrustRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Order: {}, Participant: {:?}, Status: {:?}, Payment: {:?}",
            self.order_id, self.participant, self.status, self.payment
        )
    }
}
