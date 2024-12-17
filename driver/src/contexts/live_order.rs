use std::rc::Rc;

use fuente::models::{
    OrderInvoiceState, OrderStateIdb, OrderStatus, DRIVER_HUB_PRIV_KEY, DRIVER_HUB_PUB_KEY,
    NOSTR_KIND_ORDER_STATE,
};
use nostr_minions::browser_api::IdbStoreManager;
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::notes::NostrNote;
use nostro2::{keypair::NostrKeypair, relays::NostrSubscription};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderHub {
    hub_keys: NostrKeypair,
    orders: Vec<(OrderInvoiceState, NostrNote)>,
    live_order: Option<(OrderInvoiceState, NostrNote)>,
}

impl OrderHub {
    pub fn get_orders(&self) -> Vec<(OrderInvoiceState, NostrNote)> {
        self.orders.clone()
    }
    pub fn has_live_order(&self) -> bool {
        self.live_order.is_some()
    }
    pub fn get_live_order(&self) -> Option<(OrderInvoiceState, NostrNote)> {
        self.live_order.clone()
    }
}

pub enum OrderHubAction {
    FinishedLoadingDb,
    FinishedLoadingRelays,
    LoadOrders(Vec<(OrderInvoiceState, NostrNote)>),
    NewOrder((OrderInvoiceState, NostrNote)),
    LiveOrder((OrderInvoiceState, NostrNote)),
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
                orders.retain(|o| o.0.order_id() != order.0.order_id());
                orders.push(order);
                Rc::new(OrderHub {
                    hub_keys: self.hub_keys.clone(),
                    orders,
                    live_order: self.live_order.clone(),
                })
            }
            OrderHubAction::LiveOrder(order) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.0.order_id() != order.0.order_id());
                Rc::new(OrderHub {
                    hub_keys: self.hub_keys.clone(),
                    orders,
                    live_order: Some(order),
                })
            }
            OrderHubAction::OrderCompleted(completed_id) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.0.order_id() != completed_id);
                Rc::new(OrderHub {
                    hub_keys: self.hub_keys.clone(),
                    orders: self.orders.clone(),
                    live_order: None,
                })
            }
            OrderHubAction::DeleteOrder(order) => {
                let mut orders = self.orders.clone();
                orders.retain(|o| o.0.order_id() != order);
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
                    if let Ok(order_note) = NostrNote::try_from(decrypted) {
                        if let Ok(order_status) = OrderInvoiceState::try_from(&order_note) {
                            if let Some(signed_note) = order_status.courier.as_ref() {
                                if signed_note.pubkey == my_keys.public_key() {
                                    // First save to IndexedDB for any order belonging to this driver
                                    let db_entry = OrderStateIdb::new(order_note.clone());
                                    if let Ok(entry) = db_entry {
                                        spawn_local(async move {
                                            if let Err(e) = entry.save_to_store().await {
                                                gloo::console::error!(
                                                    "Failed to save order to IndexedDB:",
                                                    e
                                                );
                                            } else {
                                                gloo::console::log!("Order saved to IndexedDB");
                                            }
                                        });
                                    }

                                    match order_status.order_status {
                                        OrderStatus::Canceled => {
                                            gloo::console::info!(
                                                "Order Canceled: ",
                                                format!("{:?}", order_status.order_status)
                                            );
                                            ctx.dispatch(OrderHubAction::DeleteOrder(
                                                order_status.order_id(),
                                            ));
                                        }
                                        OrderStatus::Completed => {
                                            gloo::console::info!(
                                                "Order Completed: ",
                                                format!("{:?}", order_status.order_status)
                                            );
                                            ctx.dispatch(OrderHubAction::OrderCompleted(
                                                order_status.order_id(),
                                            ));
                                        }
                                        _ => {
                                            gloo::console::info!(
                                                "New LIVE Order: ",
                                                format!("{:?}", order_status.order_status)
                                            );
                                            ctx.dispatch(OrderHubAction::NewOrder((
                                                order_status.clone(),
                                                order_note.clone(),
                                            )));
                                            ctx.dispatch(OrderHubAction::LiveOrder((
                                                order_status,
                                                order_note,
                                            )));
                                        }
                                    }
                                }
                            } else {
                                // No courier assigned means we can add it to pool
                                gloo::console::info!(
                                    "New Order: ",
                                    format!("{:?}", order_status.order_id())
                                );
                                ctx.dispatch(OrderHubAction::NewOrder((order_status, order_note)));
                            }
                        }
                    }
                }
            }
        }
        || {}
    });

    html! {}
}
