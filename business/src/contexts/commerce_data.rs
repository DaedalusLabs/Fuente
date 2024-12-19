use fuente::models::{
    CommerceProfile, CommerceProfileIdb, ProductMenu, ProductMenuIdb, NOSTR_KIND_COMMERCE_PRODUCTS,
    NOSTR_KIND_COMMERCE_PROFILE, NOSTR_KIND_PRESIGNED_URL_REQ, NOSTR_KIND_PRESIGNED_URL_RESP,
};
use nostr_minions::{browser_api::IdbStoreManager, key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::relays::{EndOfSubscriptionEvent, NostrSubscription, RelayEvent};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommerceData {
    checked_db: bool,
    checked_relay: bool,
    profile: Option<CommerceProfileIdb>,
    menu: Option<ProductMenuIdb>,
}

impl CommerceData {
    pub fn checked_db(&self) -> bool {
        self.checked_db
    }
    pub fn checked_relay(&self) -> bool {
        self.checked_relay
    }
    pub fn profile(&self) -> Option<CommerceProfile> {
        if let Some(p) = &self.profile {
            Some(p.profile().clone())
        } else {
            None
        }
    }
    pub fn menu(&self) -> Option<ProductMenu> {
        if let Some(m) = &self.menu {
            Some(m.menu().clone())
        } else {
            None
        }
    }
}

pub enum CommerceDataAction {
    CheckedDb,
    CheckedRelay,
    LoadCommerceData(CommerceProfileIdb),
    LoadProductData(ProductMenuIdb),
    UpdateCommerceProfile(CommerceProfileIdb),
    UpdateProductList(ProductMenuIdb),
}

impl Reducible for CommerceData {
    type Action = CommerceDataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CommerceDataAction::UpdateCommerceProfile(profile) => {
                let db_entry = profile.clone();
                spawn_local(async move {
                    db_entry.save_to_store().await.expect("Failed to save");
                });
                Rc::new(CommerceData {
                    checked_db: self.checked_db,
                    checked_relay: self.checked_relay,
                    profile: Some(profile),
                    menu: self.menu.clone(),
                })
            }
            CommerceDataAction::UpdateProductList(list) => {
                let db_entry = list.clone();
                spawn_local(async move {
                    db_entry.save_to_store().await.expect("Failed to save");
                });
                Rc::new(CommerceData {
                    checked_db: self.checked_db,
                    checked_relay: self.checked_relay,
                    profile: self.profile.clone(),
                    menu: Some(list),
                })
            }
            CommerceDataAction::LoadCommerceData(db_entries) => Rc::new(CommerceData {
                checked_db: self.checked_db,
                checked_relay: self.checked_relay,
                profile: Some(db_entries),
                menu: self.menu.clone(),
            }),
            CommerceDataAction::LoadProductData(db_entries) => Rc::new(CommerceData {
                checked_db: self.checked_db,
                checked_relay: self.checked_relay,
                profile: self.profile.clone(),
                menu: Some(db_entries),
            }),
            CommerceDataAction::CheckedDb => Rc::new(CommerceData {
                checked_db: true,
                checked_relay: self.checked_relay,
                profile: self.profile.clone(),
                menu: self.menu.clone(),
            }),
            CommerceDataAction::CheckedRelay => Rc::new(CommerceData {
                checked_db: self.checked_db,
                checked_relay: true,
                profile: self.profile.clone(),
                menu: self.menu.clone(),
            }),
        }
    }
}

pub type CommerceDataStore = UseReducerHandle<CommerceData>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct CommerceDataChildren {
    pub children: Children,
}

#[function_component(CommerceDataProvider)]
pub fn key_handler(props: &CommerceDataChildren) -> Html {
    let ctx = use_reducer(|| CommerceData {
        checked_relay: false,
        checked_db: false,
        profile: None,
        menu: None,
    });

    let ctx_clone = ctx.clone();
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    use_effect_with(key_ctx, move |key_ctx| {
        if let Some(key) = key_ctx.get_nostr_key() {
            let pubkey = key.public_key().to_string();
            spawn_local(async move {
                if let Ok(profile) =
                    CommerceProfileIdb::retrieve_from_store(&JsValue::from_str(&pubkey)).await
                {
                    ctx_clone.dispatch(CommerceDataAction::LoadCommerceData(profile));
                }
                if let Ok(products) =
                    ProductMenuIdb::retrieve_from_store(&JsValue::from_str(&pubkey)).await
                {
                    ctx_clone.dispatch(CommerceDataAction::LoadProductData(products));
                }
                ctx_clone.dispatch(CommerceDataAction::CheckedDb);
            });
        }
        || {}
    });

    html! {
        <ContextProvider<CommerceDataStore> context={ctx}>
            {props.children.clone()}
            <CommerceDataSync />
        </ContextProvider<CommerceDataStore>>
    }
}

#[function_component(CommerceDataSync)]
pub fn commerce_data_sync() -> Html {
    let ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let relay_events = relay_ctx.relay_events.clone();

    let id_handle = sub_id.clone();
    use_effect_with(key_ctx, move |key_ctx| {
        if let Some(key) = key_ctx.get_nostr_key() {
            let filter = NostrSubscription {
                kinds: Some(vec![
                    NOSTR_KIND_COMMERCE_PROFILE,
                    NOSTR_KIND_COMMERCE_PRODUCTS,
                    NOSTR_KIND_PRESIGNED_URL_REQ,  // Add this
                    NOSTR_KIND_PRESIGNED_URL_RESP, // And this
                ]),
                authors: Some(vec![key.public_key()]),
                ..Default::default()
            }
            .relay_subscription();
            id_handle.set(filter.1.clone());
            subscriber.emit(filter);
        }
        || {}
    });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvent::EndOfSubscription(EndOfSubscriptionEvent(_, id))) = events.last() {
            if id == &(*id_handle) {
                ctx_clone.dispatch(CommerceDataAction::CheckedRelay);
            }
        }
        || {}
    });
    let ctx_clone = ctx.clone();
    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            gloo::console::log!("Note kind received:", note.kind);
            match note.kind {
                NOSTR_KIND_COMMERCE_PROFILE => {
                    if let Ok(profile) = note.clone().try_into() {
                        ctx_clone.dispatch(CommerceDataAction::UpdateCommerceProfile(profile));
                    }
                }
                NOSTR_KIND_COMMERCE_PRODUCTS => {
                    if let Ok(products) = note.clone().try_into() {
                        ctx_clone.dispatch(CommerceDataAction::UpdateProductList(products));
                    }
                }
                _ => {}
            }
        }
        || {}
    });

    html! {}
}
