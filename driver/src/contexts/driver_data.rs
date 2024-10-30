use fuente::{
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    models::{
        driver::{DriverProfile, DriverProfileIdb},
        nostr_kinds::{
            NOSTR_KIND_CONSUMER_PROFILE, NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP,
            NOSTR_KIND_DRIVER_PROFILE,
        },
    },
};
use nostro2::{
    notes::SignedNote,
    relays::{NostrFilter, RelayEvents},
};
use std::rc::Rc;
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
    pub fn get_profile_note(&self) -> Option<SignedNote> {
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
                    db_entry.save().await.expect("Failed to save profile");
                });
                Rc::new(DriverData {
                    has_loaded: self.has_loaded,
                    profile: Some(profile),
                })
            }
            DriverDataAction::DeleteProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    let _ = db_entry.delete().await;
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
        if let Some(key) = key_ctx.get_key() {
            spawn_local(async move {
                if let Ok(profile) = DriverProfileIdb::find_profile(&key.get_public_key()).await {
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
        if let Some(keys) = keys.get_key() {
            if &(*id_handle) == "" {
                let pubkey = keys.get_public_key();
                spawn_local(async move {
                    let filter = NostrFilter::default()
                        .new_kind(NOSTR_KIND_DRIVER_PROFILE)
                        .new_author(&pubkey)
                        .subscribe();
                    id_handle.set(filter.id());
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
    //         match note.get_kind() {
    //             _ => {}
    //         }
    //     }
    //     || {}
    // });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvents::EOSE(id)) = events.last() {
            if id == &(*id_handle) {
                ctx_clone.dispatch(DriverDataAction::FinishedLoadingRelays);
            }
        }
        || {}
    });

    html! {}
}
