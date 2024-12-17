use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};

use crate::models::{NOSTR_KIND_SERVER_REQUEST, TEST_PUB_KEY};

use super::{state::OrderStatus, OrderInvoiceState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdateRequest {
    pub order: NostrNote,
    pub status_update: OrderStatus,
}
impl TryFrom<NostrNote> for OrderUpdateRequest {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        let content = note.content;
        let update: OrderUpdateRequest = serde_json::from_str(content.as_str())?;
        Ok(update)
    }
}
impl OrderUpdateRequest {
    pub fn new(order: NostrNote, status_update: OrderStatus) -> Self {
        Self {
            order,
            status_update,
        }
    }
    pub fn invoice_state(&self) -> anyhow::Result<OrderInvoiceState> {
        let invoice_state = OrderInvoiceState::try_from(&self.order)?;
        Ok(invoice_state)
    }
    pub fn sign_update(&self, keys: &NostrKeypair, kind: u32) -> anyhow::Result<NostrNote> {
        let mut note = NostrNote {
            kind,
            content: serde_json::to_string(self)?,
            pubkey: keys.public_key(),
            ..Default::default()
        };
        keys.sign_nostr_event(&mut note);
        let mut giftwrap = NostrNote {
            kind: NOSTR_KIND_SERVER_REQUEST,
            content: note.to_string(),
            pubkey: keys.public_key(),
            ..Default::default()
        };
        keys.sign_nip_04_encrypted(&mut giftwrap, TEST_PUB_KEY.to_string())?;
        Ok(giftwrap)
    }
}
