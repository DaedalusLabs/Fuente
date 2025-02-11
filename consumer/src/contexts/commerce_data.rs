use std::rc::Rc;

use fuente::{
    contexts::AdminConfigsStore,
    models::{
        CommerceProfile, CommerceProfileIdb, ProductItem, ProductMenuIdb,
        NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE,
    },
};
use nostr_minions::relay_pool::NostrProps;
use nostro2::relays::{NostrSubscription, RelayEvent};
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
    pub fn find_commerce_by_id(&self, id: &str) -> Option<CommerceProfileIdb> {
        self.commerces.iter().find(|p| p.id() == id).cloned()
    }
    pub fn find_product_list_by_id(&self, id: &str) -> Option<ProductMenuIdb> {
        self.products_lists.iter().find(|p| p.id() == id).cloned()
    }
    pub fn find_product(&self, commerce_id: &str, product_id: &str) -> Option<ProductItem> {
        self.products_lists
            .iter()
            .find(|p| p.id() == commerce_id)
            .and_then(|p| {
                p.menu().categories().iter().fold(None, |acc, c| {
                    acc.or_else(|| {
                        c.products()
                            .iter()
                            .find(|i| i.id() == product_id)
                            .map(|i| i.clone())
                    })
                })
            })
    }
}

pub trait CommerceDataExt {
    fn find_commerce_by_id(&self, id: &str) -> Option<CommerceProfile>;
    fn is_loaded(&self) -> bool;
}

impl CommerceDataExt for UseReducerHandle<CommerceData> {
    fn find_commerce_by_id(&self, id: &str) -> Option<CommerceProfile> {
        if !self.finished_loading() {
            gloo::console::warn!("Attempting to find commerce while data is still loading");
            return None;
        }

        self.commerces
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.profile())
            .cloned()
    }

    fn is_loaded(&self) -> bool {
        self.finished_loading()
    }
}

pub enum CommerceDataAction {
    FinishedLoadingRelays,
    UpdateCommerceProfile(CommerceProfileIdb),
    UpdateProductList(ProductMenuIdb),
    FilterWhiteList(Vec<String>),
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
            CommerceDataAction::FilterWhiteList(wl) => Rc::new({
                let mut self_clone = (*self).clone();
                self_clone
                    .commerces
                    .retain(|p| wl.contains(&p.signed_note().pubkey));
                self_clone
                    .products_lists
                    .retain(|p| wl.contains(&p.note().pubkey));
                self_clone
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
    use_effect_with(admin_configs.clone(), move |configs| {
        if configs.is_loaded() {
            let filter: nostro2::relays::SubscribeEvent = NostrSubscription {
                kinds: Some(vec![
                    NOSTR_KIND_COMMERCE_PROFILE,
                    NOSTR_KIND_COMMERCE_PRODUCTS,
                ]),
                ..Default::default()
            }
            .into();
            id_handle.set(filter.1.clone());
            subscriber.emit(filter);
        }
        || {}
    });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvent::EndOfSubscription((_, id))) = events.last() {
            if *id == *id_handle {
                ctx_clone.dispatch(CommerceDataAction::FinishedLoadingRelays);
            }
        }
        || {}
    });
    let ctx_clone = ctx.clone();

    use_effect_with(
        (admin_configs.get_commerce_whitelist(), unique_notes),
        move |(admin_wl, notes)| {
            if let Some(note) = notes.last() {
                match note.kind {
                    NOSTR_KIND_COMMERCE_PROFILE => {
                        if admin_wl.contains(&note.pubkey) {
                            match note.clone().try_into() {
                                Ok(profile) => {
                                    ctx_clone.dispatch(CommerceDataAction::UpdateCommerceProfile(
                                        profile,
                                    ));
                                }
                                Err(e) => {
                                    gloo::console::error!(
                                        "Error in commerce profile",
                                        format!("{:?}", e)
                                    );
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
        },
    );
    let ctx_clone = ctx.clone();
    use_effect_with(admin_configs, move |wl| {
        let wl = wl.get_commerce_whitelist();
        if !wl.is_empty() {
            ctx_clone.dispatch(CommerceDataAction::FilterWhiteList(wl));
        }
        || {}
    });
    html! {}
}
