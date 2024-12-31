use std::rc::Rc;

use fuente::{
    contexts::AdminConfigsStore,
    models::{
        CommerceProfile, CommerceProfileIdb, ProductMenuIdb, NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE
    },
};
use nostr_minions::relay_pool::NostrProps;
use nostro2::relays::{EndOfSubscriptionEvent, NostrSubscription, RelayEvent};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommerceData {
    has_loaded: bool,
    commerces: Vec<CommerceProfileIdb>,
    products_lists: Vec<ProductMenuIdb>,
}

impl CommerceData {
    pub fn finished_loading(&self) -> bool {
        self.has_loaded
    }
    pub fn commerces(&self) -> Vec<CommerceProfileIdb> {
        self.commerces.clone()
    }
    pub fn products_lists(&self) -> Vec<ProductMenuIdb> {
        self.products_lists.clone()
    }
}

pub trait CommerceDataExt {
    fn find_commerce_by_id(&self, id: &str) -> Option<CommerceProfile>;
    fn is_loading(&self) -> bool;
}

impl CommerceDataExt for UseReducerHandle<CommerceData> {
    fn find_commerce_by_id(&self, id: &str) -> Option<CommerceProfile> {
        if !self.finished_loading() {
            gloo::console::warn!("Attempting to find commerce while data is still loading");
            return None;
        }
        
        self.commerces.iter()
            .find(|p| p.id() == id)
            .map(|p| p.profile())
            .cloned()
    }

    fn is_loading(&self) -> bool {
        !self.finished_loading()
    }
}

pub enum CommerceDataAction {
    FinishedLoadingRelays,
    UpdateCommerceProfile(CommerceProfileIdb),
    UpdateProductList(ProductMenuIdb),
}

impl Reducible for CommerceData {
    type Action = CommerceDataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CommerceDataAction::UpdateCommerceProfile(profile) => {
                let mut commerces = self.commerces.clone();
                commerces.retain(|p| p.id() != profile.id());
                commerces.push(profile.clone());
                Rc::new(CommerceData {
                    commerces,
                    ..(*self).clone()
                })
            }
            CommerceDataAction::UpdateProductList(list) => {
                let mut products_lists = self.products_lists.clone();
                products_lists.retain(|p| p.id() != list.id());
                products_lists.push(list.clone());
                Rc::new(CommerceData {
                    products_lists,
                    ..(*self).clone()
                })
            }
            CommerceDataAction::FinishedLoadingRelays => Rc::new(CommerceData {
                has_loaded: true,
                ..(*self).clone()
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
        has_loaded: false,
        commerces: vec![],
        products_lists: vec![],
    });

    let ctx_clone = ctx.clone();
    let admin_configs = use_context::<AdminConfigsStore>().expect("NostrProps not found");
    let commerce_wl = admin_configs.get_commerce_whitelist();

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
    let admin_configs = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let relay_events = relay_ctx.relay_events.clone();

    let id_handle = sub_id.clone();
    use_effect_with((), move |_| {
        let filter = NostrSubscription {
            kinds: Some(vec![
                NOSTR_KIND_COMMERCE_PROFILE,
                NOSTR_KIND_COMMERCE_PRODUCTS,
            ]),
            ..Default::default()
        }
        .relay_subscription();
        id_handle.set(filter.1.clone());
        subscriber.emit(filter);
        || {}
    });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvent::EndOfSubscription(EndOfSubscriptionEvent(_, id))) = events.last() {
            if *id == *id_handle {
                ctx_clone.dispatch(CommerceDataAction::FinishedLoadingRelays);
            }
        }
        || {}
    });
    let ctx_clone = ctx.clone();
    let admin_wl = admin_configs.get_commerce_whitelist();
    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            match note.kind {
                NOSTR_KIND_COMMERCE_PROFILE => {
                    if admin_wl.contains(&note.pubkey) {
                        match note.clone().try_into() {
                            Ok(profile) => {
                                ctx_clone
                                    .dispatch(CommerceDataAction::UpdateCommerceProfile(profile));
                            }
                            Err(e) => {
                                gloo::console::error!("Error in commerce profile", format!("{:?}", e));
                            }
                        }
                    }
                }
                NOSTR_KIND_COMMERCE_PRODUCTS => {
                    if admin_wl.contains(&note.pubkey) {
                        if let Ok(products) = note.clone().try_into() {
                            ctx_clone.dispatch(CommerceDataAction::UpdateProductList(products));
                        }
                    }
                }
                _ => {}
            }
        }
        || {}
    });
    html! {}
}
