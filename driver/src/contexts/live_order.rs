use std::rc::Rc;

use fuente::{
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    models::{
        nostr_kinds::NOSTR_KIND_ORDER_STATE, orders::OrderInvoiceState, DRIVER_HUB_PRIV_KEY,
        DRIVER_HUB_PUB_KEY,
    },
};
use nostro2::{
    relays::{NostrFilter, RelayEvents},
    userkeys::UserKeys,
};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderHub {
    has_loaded: (bool, bool),
    hub_keys: UserKeys,
    orders: Vec<OrderInvoiceState>,
}

impl OrderHub {}

pub enum OrderHubAction {
    FinishedLoadingDb,
    FinishedLoadingRelays,
    LoadOrders(Vec<OrderInvoiceState>),
    NewOrder(OrderInvoiceState),
    DeleteOrder(String),
}

impl Reducible for OrderHub {
    type Action = OrderHubAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            OrderHubAction::LoadOrders(orders) => Rc::new(OrderHub {
                has_loaded: self.has_loaded,
                hub_keys: self.hub_keys.clone(),
                orders,
            }),
            OrderHubAction::FinishedLoadingDb => Rc::new(OrderHub {
                has_loaded: (true, self.has_loaded.1),
                hub_keys: self.hub_keys.clone(),
                orders: self.orders.clone(),
            }),
            OrderHubAction::FinishedLoadingRelays => Rc::new(OrderHub {
                has_loaded: (self.has_loaded.0, true),
                hub_keys: self.hub_keys.clone(),
                orders: self.orders.clone(),
            }),
            OrderHubAction::NewOrder(order) => {
                let mut orders = self.orders.clone();
                orders.push(order);
                Rc::new(OrderHub {
                    has_loaded: self.has_loaded,
                    hub_keys: self.hub_keys.clone(),
                    orders,
                })
            }
            OrderHubAction::DeleteOrder(order) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.id() != order);
                Rc::new(OrderHub {
                    has_loaded: self.has_loaded,
                    hub_keys: self.hub_keys.clone(),
                    orders,
                })
            }
        }
    }
}

pub type OrderHubStore = UseReducerHandle<OrderHub>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct OrderHubChildren {
    pub children: Children,
}

#[function_component(OrderHubProvider)]
pub fn key_handler(props: &OrderHubChildren) -> Html {
    let ctx = use_reducer(|| OrderHub {
        has_loaded: (false, false),
        hub_keys: UserKeys::new(DRIVER_HUB_PRIV_KEY).expect("Failed to create user keys"),
        orders: vec![],
    });

    // let ctx_clone = ctx.clone();
    // use_effect_with((), move |_| {});

    html! {
        <ContextProvider<OrderHubStore> context={ctx}>
            {props.children.clone()}
            <OrderHubSync />
        </ContextProvider<OrderHubStore>>
    }
}

#[function_component(OrderHubSync)]
pub fn commerce_data_sync() -> Html {
    let ctx = use_context::<OrderHubStore>().expect("Commerce context not found");
    let hub_keys = ctx.hub_keys.clone();
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let keys_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");

    let id_handle = sub_id.clone();
    use_effect_with(keys_ctx.clone(), move |key_ctx| {
        let filter = NostrFilter::default()
            .new_kind(NOSTR_KIND_ORDER_STATE)
            .new_tag("p", vec![DRIVER_HUB_PUB_KEY.to_string()])
            .subscribe();
        id_handle.set(filter.id());
        subscriber.emit(filter);
        gloo::console::log!("Subscribed to order state");
        || {}
    });

    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            if note.get_kind() == NOSTR_KIND_ORDER_STATE {
                if let Ok(decrypted) = hub_keys.decrypt_nip_04_content(&note) {
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
