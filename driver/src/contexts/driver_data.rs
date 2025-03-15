use fuente::models::{DriverProfile, DriverProfileIdb, NOSTR_KIND_COURIER_PROFILE};
use nostr_minions::{
    browser_api::IdbStoreManager, key_manager::NostrIdStore, relay_pool::NostrProps,
};
use nostro2::{
    notes::NostrNote,
    relays::{NostrSubscription, RelayEvent},
};
use std::rc::Rc;
use web_sys::wasm_bindgen::JsValue;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriverData {
    has_loaded: (bool, bool),
    profile: Option<DriverProfileIdb>,
}

impl DriverData {
    pub fn finished_loading(&self) -> bool {
        self.has_loaded == (true, true)
    }
    pub fn get_profile(&self) -> Option<DriverProfile> {
        match &self.profile {
            Some(profile) => Some(profile.profile()),
            None => None,
        }
    }
    pub fn get_profile_note(&self) -> Option<NostrNote> {
        match &self.profile {
            Some(profile) => Some(profile.signed_note()),
            None => None,
        }
    }
}

pub enum DriverDataAction {
    FinishedLoadingDb,
    FinishedLoadingRelays,
    LoadProfile(DriverProfileIdb),
    NewProfile(DriverProfileIdb),
    DeleteProfile(DriverProfileIdb),
}

impl Reducible for DriverData {
    type Action = DriverDataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            DriverDataAction::LoadProfile(profile) => Rc::new(DriverData {
                has_loaded: self.has_loaded,
                profile: Some(profile),
            }),
            DriverDataAction::FinishedLoadingDb => Rc::new(DriverData {
                has_loaded: (true, true),
                profile: self.profile.clone(),
            }),
            DriverDataAction::FinishedLoadingRelays => Rc::new(DriverData {
                has_loaded: (self.has_loaded.0, true),
                profile: self.profile.clone(),
            }),
            DriverDataAction::NewProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    db_entry
                        .save_to_store()
                        .await
                        .expect("Failed to save profile");
                });
                Rc::new(DriverData {
                    has_loaded: self.has_loaded,
                    profile: Some(profile),
                })
            }
            DriverDataAction::DeleteProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    let _ = db_entry.delete_from_store().await;
                });
                Rc::new(DriverData {
                    has_loaded: self.has_loaded,
                    profile: None,
                })
            }
        }
    }
}

pub type DriverDataStore = UseReducerHandle<DriverData>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct DriverDataChildren {
    pub children: Children,
}

#[function_component(DriverDataProvider)]
pub fn key_handler(props: &DriverDataChildren) -> Html {
    let ctx = use_reducer(|| DriverData {
        has_loaded: (false, false),
        profile: None,
    });

    let ctx_clone = ctx.clone();
    let key_ctx = use_context::<NostrIdStore>().expect("User context not found");
    use_effect_with(key_ctx, |key_ctx| {
        if let Some(key) = key_ctx.get_pubkey() {
            spawn_local(async move {
                if let Ok(profile) =
                    DriverProfileIdb::retrieve_from_store(&JsValue::from_str(&key))
                        .await
                {
                    ctx_clone.dispatch(DriverDataAction::LoadProfile(profile));
                }
                ctx_clone.dispatch(DriverDataAction::FinishedLoadingDb);
            });
        }
        || {}
    });

    html! {
        <ContextProvider<DriverDataStore> context={ctx}>
            {props.children.clone()}
            <DriverDataSync />
        </ContextProvider<DriverDataStore>>
    }
}
#[function_component(DriverDataSync)]
pub fn commerce_data_sync() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("User context not found");
    let ctx = use_context::<DriverDataStore>().expect("User context not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let relay_events = relay_ctx.relay_events.clone();

    let id_handle = sub_id.clone();
    use_effect_with(key_ctx.clone(), move |keys| {
        if let Some(pubkey) = keys.get_pubkey() {
            if &(*id_handle) == "" {
                spawn_local(async move {
                    let filter: nostro2::relays::SubscribeEvent = NostrSubscription {
                        kinds: Some(vec![NOSTR_KIND_COURIER_PROFILE]),
                        authors: Some(vec![pubkey.clone()]),
                        ..Default::default()
                    }
                    .into();
                    id_handle.set(filter.1.clone());
                    subscriber.emit(filter);
                });
            }
        }
        || {}
    });

    // let unique_notes = relay_ctx.unique_notes.clone();
    // let ctx_clone = ctx.clone();
    // use_effect_with(unique_notes, move |notes| {
    //     if let Some(note) = notes.last() {
    //         match note.kind {
    //             _ => {}
    //         }
    //     }
    //     || {}
    // });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvent::EndOfSubscription((_, id))) = events.last() {
            if id == &(*id_handle) {
                ctx_clone.dispatch(DriverDataAction::FinishedLoadingRelays);
            }
        }
        || {}
    });

    html! {}
}
