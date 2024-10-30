use std::rc::Rc;

use fuente::{
    contexts::relay_pool::NostrProps,
    models::{
        commerce::CommerceProfileIdb,
        nostr_kinds::{NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE},
        products::ProductMenuIdb,
    },
};
use nostro2::relays::{NostrFilter, RelayEvents};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommerceData {
    has_loaded: (bool, bool),
    commerces: Vec<CommerceProfileIdb>,
    products_lists: Vec<ProductMenuIdb>,
}

impl CommerceData {
    pub fn finished_loading(&self) -> bool {
        self.has_loaded == (true, true)
    }
    pub fn commerces(&self) -> Vec<CommerceProfileIdb> {
        self.commerces.clone()
    }
    pub fn products_lists(&self) -> Vec<ProductMenuIdb> {
        self.products_lists.clone()
    }
    pub fn find_commerce(&self, id: &str) -> Option<CommerceProfileIdb> {
        self.commerces.iter().find(|p| p.id() == id).cloned()
    }
}

pub enum CommerceDataAction {
    FinishedLoadingRelays,
    FinishedLoadingDb,
    LoadCommerceData(Vec<CommerceProfileIdb>),
    LoadProductData(Vec<ProductMenuIdb>),
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
                spawn_local(async move {
                    profile.save().await.expect("Failed to save");
                });
                Rc::new(CommerceData {
                    has_loaded: self.has_loaded,
                    commerces,
                    products_lists: self.products_lists.clone(),
                })
            }
            CommerceDataAction::UpdateProductList(list) => {
                let mut products_lists = self.products_lists.clone();
                products_lists.retain(|p| p.id() != list.id());
                products_lists.push(list.clone());
                spawn_local(async move {
                    list.save().await.expect("Failed to save");
                });
                Rc::new(CommerceData {
                    has_loaded: self.has_loaded,
                    commerces: self.commerces.clone(),
                    products_lists,
                })
            }
            CommerceDataAction::LoadCommerceData(db_entries) => Rc::new(CommerceData {
                has_loaded: self.has_loaded,
                commerces: db_entries,
                products_lists: self.products_lists.clone(),
            }),
            CommerceDataAction::LoadProductData(db_entries) => Rc::new(CommerceData {
                has_loaded: self.has_loaded,
                commerces: self.commerces.clone(),
                products_lists: db_entries,
            }),
            CommerceDataAction::FinishedLoadingRelays => Rc::new(CommerceData {
                has_loaded: (self.has_loaded.0, true),
                commerces: self.commerces.clone(),
                products_lists: self.products_lists.clone(),
            }),
            CommerceDataAction::FinishedLoadingDb => Rc::new(CommerceData {
                has_loaded: (true, self.has_loaded.1),
                commerces: self.commerces.clone(),
                products_lists: self.products_lists.clone(),
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
        has_loaded: (true, true),
        commerces: vec![],
        products_lists: vec![],
    });

    let ctx_clone = ctx.clone();
    use_effect_with((), move |_| {
        spawn_local(async move {
            match CommerceProfileIdb::find_all().await {
                Ok(vec) => {
                    ctx_clone.dispatch(CommerceDataAction::LoadCommerceData(vec));
                }
                Err(e) => gloo::console::error!(format!("{:?}", e)),
            };
            match ProductMenuIdb::find_all().await {
                Ok(vec) => {
                    ctx_clone.dispatch(CommerceDataAction::LoadProductData(vec));
                }
                Err(e) => gloo::console::error!(format!("{:?}", e)),
            };
            ctx_clone.dispatch(CommerceDataAction::FinishedLoadingDb);
        });
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
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let relay_events = relay_ctx.relay_events.clone();

    let id_handle = sub_id.clone();
    use_effect_with((), move |_| {
        let filter = NostrFilter::default()
            .new_kinds(vec![
                NOSTR_KIND_COMMERCE_PROFILE,
                NOSTR_KIND_COMMERCE_PRODUCTS,
            ])
            .subscribe();
        id_handle.set(filter.id());
        subscriber.emit(filter);
        || {}
    });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvents::EOSE(id)) = events.last() {
            if id == &(*id_handle) {
                ctx_clone.dispatch(CommerceDataAction::FinishedLoadingRelays);
            }
        }
        || {}
    });
    let ctx_clone = ctx.clone();
    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            match note.get_kind() {
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
