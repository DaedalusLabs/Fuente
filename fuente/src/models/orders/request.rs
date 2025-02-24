#[cfg(target_arch = "wasm32")]
use nostr_minions::key_manager::UserIdentity;
#[cfg(target_arch = "wasm32")]
use nostro2::notes::NostrNote;

#[cfg(target_arch = "wasm32")]
use crate::models::NOSTR_KIND_SERVER_REQUEST;
use crate::models::{
    ConsumerAddress, ConsumerProfile, ProductOrder, NOSTR_KIND_CONSUMER_ORDER_REQUEST,
};

#[derive(Debug, Clone, PartialEq, Hash, Eq, serde::Serialize, serde::Deserialize)]
pub struct OrderRequest {
    pub commerce: String,
    pub profile: ConsumerProfile,
    pub address: ConsumerAddress,
    pub products: ProductOrder,
}
impl Default for OrderRequest {
    fn default() -> Self {
        Self {
            commerce: "".to_string(),
            profile: ConsumerProfile::default(),
            address: ConsumerAddress::default(),
            products: ProductOrder::default(),
        }
    }
}
impl ToString for OrderRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl TryFrom<nostro2::notes::NostrNote> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(note: nostro2::notes::NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderRequest = note.content.try_into()?;
        Ok(order)
    }
}
impl TryFrom<&nostro2::notes::NostrNote> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(note: &nostro2::notes::NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderRequest = note.content.clone().try_into()?;
        Ok(order)
    }
}
impl OrderRequest {
    pub fn new(
        commerce: String,
        profile: ConsumerProfile,
        address: ConsumerAddress,
        products: ProductOrder,
    ) -> Self {
        Self {
            commerce,
            profile,
            address,
            products,
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub async fn sign_request(&self, keys: &UserIdentity) -> NostrNote {
        let content = self.to_string();
        let note = NostrNote {
            pubkey: keys.get_pubkey().await.expect("no pubkey"),
            kind: NOSTR_KIND_CONSUMER_ORDER_REQUEST,
            content,
            ..Default::default()
        };
        keys.sign_nostr_note(note).await.expect("could not sign")
    }
    #[cfg(target_arch = "wasm32")]
    pub async fn giftwrapped_request(
        &self,
        keys: &UserIdentity,
        recipient: String,
    ) -> anyhow::Result<NostrNote> {
        let note = self.sign_request(keys).await;
        let content = note.to_string();
        let giftwrap = NostrNote {
            pubkey: keys.get_pubkey().await.expect("no pubkey"),
            kind: NOSTR_KIND_SERVER_REQUEST,
            content,
            ..Default::default()
        };
        keys.sign_nip44(giftwrap, recipient)
            .await
            .map_err(|_e| anyhow::anyhow!("could not sign cart"))
    }
}
