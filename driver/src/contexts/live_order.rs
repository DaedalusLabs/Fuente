use std::rc::Rc;

use fuente::models::{
    OrderInvoiceState, OrderStateIdb, OrderStatus, DRIVER_HUB_PRIV_KEY, DRIVER_HUB_PUB_KEY, NOSTR_KIND_ORDER_STATE
};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{relays::NostrSubscription, keypair::NostrKeypair};
use yew::{platform::spawn_local, prelude::*};
use nostr_minions::browser_api::IdbStoreManager;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderHub {
    hub_keys: NostrKeypair,
    orders: Vec<OrderInvoiceState>,
    live_order: Option<OrderInvoiceState>,
}

impl OrderHub {
    pub fn get_orders(&self) -> Vec<OrderInvoiceState> {
        self.orders.clone()
    }
    pub fn has_live_order(&self) -> bool {
        self.live_order.is_some()
    }
    pub fn get_live_order(&self) -> Option<OrderInvoiceState> {
        self.live_order.clone()
    }
}

pub enum OrderHubAction {
    FinishedLoadingDb,
    FinishedLoadingRelays,
    LoadOrders(Vec<OrderInvoiceState>),
    NewOrder(OrderInvoiceState),
    LiveOrder(OrderInvoiceState),
    DeleteOrder(String),
    OrderCompleted(String),
}

impl Reducible for OrderHub {
    type Action = OrderHubAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            OrderHubAction::LoadOrders(orders) => Rc::new(OrderHub {
                hub_keys: self.hub_keys.clone(),
                live_order: self.live_order.clone(),
                orders,
            }),
            OrderHubAction::FinishedLoadingDb => Rc::new(OrderHub {
                hub_keys: self.hub_keys.clone(),
                orders: self.orders.clone(),
                live_order: self.live_order.clone(),
            }),
            OrderHubAction::FinishedLoadingRelays => Rc::new(OrderHub {
                hub_keys: self.hub_keys.clone(),
                orders: self.orders.clone(),
                live_order: self.live_order.clone(),
            }),
            OrderHubAction::NewOrder(order) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.id() != order.id());
                orders.push(order);
                Rc::new(OrderHub {
                    hub_keys: self.hub_keys.clone(),
                    orders,
                    live_order: self.live_order.clone(),
                })
            }
            OrderHubAction::LiveOrder(order) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.id() != order.id());
                Rc::new(OrderHub {
                    hub_keys: self.hub_keys.clone(),
                    orders,
                    live_order: Some(order),
                })
            }
            OrderHubAction::OrderCompleted(completed_id) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.id() != completed_id);
                Rc::new(OrderHub {
                    hub_keys: self.hub_keys.clone(),
                    orders: self.orders.clone(),
                    live_order: None,
                })
            }
            OrderHubAction::DeleteOrder(order) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.id() != order);
                Rc::new(OrderHub {
                    hub_keys: self.hub_keys.clone(),
                    orders,
                    live_order: self.live_order.clone(),
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
        hub_keys: NostrKeypair::new(DRIVER_HUB_PRIV_KEY).expect("Failed to create user keys"),
        orders: vec![],
        live_order: None,
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
        if let Some(_keys) = key_ctx.get_nostr_key() {
            let mut filter = NostrSubscription {
                kinds: Some(vec![NOSTR_KIND_ORDER_STATE]),
                ..Default::default()
            };
            filter.add_tag("#p", DRIVER_HUB_PUB_KEY);
            let sub = filter.relay_subscription();
            id_handle.set(sub.1.clone());
            subscriber.emit(sub);
        }
        || {}
    });

    let my_keys = keys_ctx.get_nostr_key().expect("No keys found");
    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            if note.kind == NOSTR_KIND_ORDER_STATE {
                if let Ok(decrypted) = hub_keys.decrypt_nip_04_content(&note) {
                    if let Ok(order_status) = OrderInvoiceState::try_from(decrypted) {
                        if let Some(signed_note) = order_status.get_courier() {
                            if signed_note.pubkey == my_keys.public_key() {
                                // First save to IndexedDB for any order belonging to this driver
                                let db_entry = OrderStateIdb::new(note.clone());
                                if let Ok(entry) = db_entry {
                                    spawn_local(async move {
                                        if let Err(e) = entry.save_to_store().await {
                                            gloo::console::error!("Failed to save order to IndexedDB:", e);
                                        } else {
                                            gloo::console::log!("Order saved to IndexedDB");
                                        }
                                    });
                                }
    
                                match order_status.get_order_status() {
                                    OrderStatus::Canceled => {
                                        gloo::console::info!(
                                            "Order Canceled: ",
                                            format!("{:?}", order_status.get_order_status())
                                        );
                                        ctx.dispatch(OrderHubAction::DeleteOrder(
                                            order_status.id(),
                                        ));
                                    }
                                    OrderStatus::Completed => {
                                        gloo::console::info!(
                                            "Order Completed: ",
                                            format!("{:?}", order_status.get_order_status())
                                        );
                                        ctx.dispatch(OrderHubAction::OrderCompleted(
                                            order_status.id(),
                                        ));
                                    }
                                    _ => {
                                        gloo::console::info!(
                                            "New LIVE Order: ",
                                            format!("{:?}", order_status.get_order_status())
                                        );
                                        ctx.dispatch(OrderHubAction::NewOrder(
                                            order_status.clone(),
                                        ));
                                        ctx.dispatch(OrderHubAction::LiveOrder(order_status));
                                    }
                                }
                            }
                        } else {
                            // No courier assigned means we can add it to pool
                            gloo::console::info!("New Order: ", format!("{:?}", order_status.id()));
                            ctx.dispatch(OrderHubAction::NewOrder(order_status));
                        }
                    }
                }
            }
        }
        || {}
    });

    html! {}
}
