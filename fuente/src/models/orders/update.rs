#[cfg(target_arch = "wasm32")]
use nostr_minions::key_manager::UserIdentity;
use nostro2::notes::NostrNote;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
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
    #[cfg(target_arch = "wasm32")]
    pub async fn sign_update(&self, keys: &UserIdentity, kind: u32) -> anyhow::Result<NostrNote> {
        let pubkey = keys
            .get_pubkey()
            .await
            .ok_or(anyhow::anyhow!("No pubkey"))?;
        let note = NostrNote {
            kind,
            content: serde_json::to_string(self)?,
            pubkey: pubkey.clone(),
            ..Default::default()
        };
        let note = keys
            .sign_nostr_note(note)
            .await
            .map_err(|_e| anyhow::anyhow!("Could not sign note"))?;
        let giftwrap = NostrNote {
            kind: NOSTR_KIND_SERVER_REQUEST,
            content: note.to_string(),
            pubkey,
            ..Default::default()
        };
        let giftwrap = keys
            .sign_nip44(giftwrap, TEST_PUB_KEY.to_string())
            .await
            .map_err(|_e| anyhow::anyhow!("Could not sign giftwrap"))?;
        Ok(giftwrap)
    }
}
