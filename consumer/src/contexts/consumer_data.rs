use fuente::models::{
    ConsumerAddress, ConsumerAddressIdb, ConsumerProfile, ConsumerProfileIdb,
    NOSTR_KIND_CONSUMER_PROFILE,
};
use nostr_minions::{browser_api::IdbStoreManager, key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{
    notes::NostrNote,
    relays::{EndOfSubscriptionEvent, NostrSubscription, RelayEvent},
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumerData {
    has_loaded: (bool, bool),
    profile: Option<ConsumerProfileIdb>,
    addresses: Vec<ConsumerAddressIdb>,
}

impl ConsumerData {
    pub fn finished_loading(&self) -> bool {
        self.has_loaded == (true, true)
    }
    pub fn get_profile(&self) -> Option<ConsumerProfile> {
        match &self.profile {
            Some(profile) => Some(profile.profile()),
            None => None,
        }
    }
    pub fn get_addresses(&self) -> Vec<ConsumerAddress> {
        self.addresses.clone().iter().map(|a| a.address()).collect()
    }
    pub fn get_default_address(&self) -> Option<ConsumerAddress> {
        self.addresses
            .iter()
            .find(|a| a.is_default())
            .map(|a| a.address())
    }
    pub fn get_address_entrys(&self) -> Vec<ConsumerAddressIdb> {
        self.addresses.clone()
    }
}

pub enum ConsumerDataAction {
    FinishedLoadingDb,
    FinishedLoadingRelays,
    LoadProfile(ConsumerProfileIdb),
    LoadAddresses(Vec<ConsumerAddressIdb>),
    SetDefaultAddress(ConsumerAddressIdb),
    NewAddress(ConsumerAddressIdb),
    DeleteAddress(ConsumerAddressIdb),
    NewProfile(ConsumerProfileIdb),
    DeleteProfile(ConsumerProfileIdb),
}

impl Reducible for ConsumerData {
    type Action = ConsumerDataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ConsumerDataAction::LoadProfile(profile) => Rc::new(ConsumerData {
                has_loaded: self.has_loaded,
                profile: Some(profile),
                addresses: self.addresses.clone(),
            }),
            ConsumerDataAction::FinishedLoadingDb => Rc::new(ConsumerData {
                has_loaded: (true, true),
                profile: self.profile.clone(),
                addresses: self.addresses.clone(),
            }),
            ConsumerDataAction::LoadAddresses(addresses) => Rc::new(ConsumerData {
                has_loaded: self.has_loaded,
                profile: self.profile.clone(),
                addresses,
            }),
            ConsumerDataAction::FinishedLoadingRelays => Rc::new(ConsumerData {
                has_loaded: (self.has_loaded.0, true),
                profile: self.profile.clone(),
                addresses: self.addresses.clone(),
            }),
            ConsumerDataAction::NewAddress(address) => {
                let db_entry = address.clone();
                spawn_local(async move {
                    db_entry
                        .save_to_store()
                        .await
                        .expect("Failed to save address");
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: self.profile.clone(),
                    addresses: {
                        let mut addresses = self.addresses.clone();
                        addresses.push(address);
                        addresses
                    },
                })
            }
            ConsumerDataAction::DeleteAddress(address) => {
                let db_entry = address.clone();
                spawn_local(async move {
                    let _ = db_entry.delete_from_store().await;
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: self.profile.clone(),
                    addresses: {
                        let mut addresses = self.addresses.clone();
                        addresses.retain(|a| a.id() != address.id());
                        addresses
                    },
                })
            }
            ConsumerDataAction::SetDefaultAddress(address) => {
                let db_entry = address.clone();
                spawn_local(async move {
                    let _ = db_entry.set_as_default().await;
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: self.profile.clone(),
                    addresses: {
                        let mut addresses = self.addresses.clone();
                        addresses.iter_mut().for_each(|a| {
                            if a.id() == address.id() {
                                a.set_default(true);
                            } else {
                                a.set_default(false);
                            }
                        });
                        addresses
                    },
                })
            }
            ConsumerDataAction::NewProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    db_entry
                        .save_to_store()
                        .await
                        .expect("Failed to save profile");
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: Some(profile),
                    addresses: self.addresses.clone(),
                })
            }
            ConsumerDataAction::DeleteProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    let _ = db_entry.delete_from_store().await;
                });
                Rc::new(ConsumerData {
                    has_loaded: self.has_loaded,
                    profile: None,
                    addresses: self.addresses.clone(),
                })
            }
        }
    }
}

pub type ConsumerDataStore = UseReducerHandle<ConsumerData>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct ConsumerDataChildren {
    pub children: Children,
}

#[function_component(ConsumerDataProvider)]
pub fn key_handler(props: &ConsumerDataChildren) -> Html {
    let ctx = use_reducer(|| ConsumerData {
        has_loaded: (false, false),
        addresses: vec![],
        profile: None,
    });

    let ctx_clone = ctx.clone();
    let key_ctx = use_context::<NostrIdStore>().expect("User context not found");
    use_effect_with(key_ctx, |key_ctx| {
        if let Some(key) = key_ctx.get_nostr_key() {
            spawn_local(async move {
                if let Ok(profile) = ConsumerProfileIdb::retrieve_from_store(&JsValue::from_str(
                    &key.public_key(),
                ))
                .await
                {
                    ctx_clone.dispatch(ConsumerDataAction::LoadProfile(profile));
                }
                if let Ok(addresses) = ConsumerAddressIdb::retrieve_all_from_store().await {
                    ctx_clone.dispatch(ConsumerDataAction::LoadAddresses(addresses));
                }
                ctx_clone.dispatch(ConsumerDataAction::FinishedLoadingDb);
            });
        }
        || {}
    });

    html! {
        <ContextProvider<ConsumerDataStore> context={ctx}>
            {props.children.clone()}
            <ConsumerDataSync />
        </ContextProvider<ConsumerDataStore>>
    }
}
#[function_component(ConsumerDataSync)]
pub fn commerce_data_sync() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("User context not found");
    let ctx = use_context::<ConsumerDataStore>().expect("User context not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let relay_events = relay_ctx.relay_events.clone();

    let id_handle = sub_id.clone();
    use_effect_with(key_ctx.clone(), move |keys| {
        if let Some(keys) = keys.get_nostr_key() {
            if &(*id_handle) == "" {
                let pubkey = keys.public_key();
                spawn_local(async move {
                    let filter = NostrSubscription {
                        kinds: Some(vec![NOSTR_KIND_CONSUMER_PROFILE]),
                        authors: Some(vec![pubkey.clone()]),
                        ..Default::default()
                    }
                    .relay_subscription();
                    id_handle.set(filter.1.clone());
                    subscriber.emit(filter);
                });
            }
        }
        || {}
    });

    let ctx_clone = ctx.clone();
    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            match note.kind {
                NOSTR_KIND_CONSUMER_PROFILE => {
                    let decrypted_note_str = key_ctx
                        .get_nostr_key()
                        .expect("No keys found")
                        .decrypt_nip_04_content(note)
                        .expect("Failed to decrypt note");
                    let decrypted_note: NostrNote = serde_json::from_str(&decrypted_note_str)
                        .expect("Failed to deserialize note");
                    if let Ok(profile) = decrypted_note.try_into() {
                        ctx_clone.dispatch(ConsumerDataAction::LoadProfile(profile))
                    } else {
                        gloo::console::info!("Received spammy note");
                    }
                }
                _ => {}
            }
        }
        || {}
    });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvent::EndOfSubscription(EndOfSubscriptionEvent(_, id))) = events.last() {
            if id == &(*id_handle) {
                ctx_clone.dispatch(ConsumerDataAction::FinishedLoadingRelays);
            }
        }
        || {}
    });

    html! {}
}
