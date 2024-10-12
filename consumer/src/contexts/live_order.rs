use std::rc::Rc;

use fuente::{
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    models::{
        commerce::CommerceProfileIdb,
        nostr_kinds::{
            NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE, NOSTR_KIND_ORDER_STATE,
        },
        orders::OrderInvoiceState,
        products::ProductMenuIdb,
    },
};
use nostro2::{
    notes::SignedNote,
    relays::{NostrFilter, RelayEvents},
};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveOrder {}

impl LiveOrder {}

pub enum LiveOrderAction {}

impl Reducible for LiveOrder {
    type Action = LiveOrderAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        // match action {
        // }
        self
    }
}

pub type LiveOrderStore = UseReducerHandle<LiveOrder>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct LiveOrderChildren {
    pub children: Children,
}

#[function_component(LiveOrderProvider)]
pub fn key_handler(props: &LiveOrderChildren) -> Html {
    let ctx = use_reducer(|| LiveOrder {});

    let ctx_clone = ctx.clone();
    use_effect_with((), move |_| {});

    html! {
        <ContextProvider<LiveOrderStore> context={ctx}>
            {props.children.clone()}
            <LiveOrderSync />
        </ContextProvider<LiveOrderStore>>
    }
}

#[function_component(LiveOrderSync)]
pub fn commerce_data_sync() -> Html {
    let ctx = use_context::<LiveOrderStore>().expect("Commerce context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let keys_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");

    let id_handle = sub_id.clone();
    use_effect_with(keys_ctx.clone(), move |key_ctx| {
        if let Some(keys) = key_ctx.get_key() {
            let filter = NostrFilter::default()
                .new_kind(NOSTR_KIND_ORDER_STATE)
                .new_tag("p", vec![keys.get_public_key().to_string()])
                .subscribe();
            id_handle.set(filter.id());
            subscriber.emit(filter);
            gloo::console::log!("Subscribed to order state");
        }
        || {}
    });

    use_effect_with(unique_notes, move |notes| {
        if let (Some(note), Some(keys)) = (notes.last(), keys_ctx.get_key()) {
            if note.get_kind() == NOSTR_KIND_ORDER_STATE {
                if let Ok(decrypted) = keys.decrypt_nip_04_content(&note) {
                    if let Ok(order_status) = OrderInvoiceState::try_from(decrypted) {
                        gloo::console::log!("Received order state", format!("{:?}", order_status));
                    }
                }
            }
        }
        || {}
    });

    html! {}
}