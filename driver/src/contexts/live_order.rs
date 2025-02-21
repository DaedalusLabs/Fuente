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
    order_history: Vec<(OrderInvoiceState, NostrNote)>,
    live_orders: Vec<(OrderInvoiceState, NostrNote)>,
    assigned_order: Option<(OrderInvoiceState, NostrNote)>,
    has_loaded: bool,
}

impl OrderHub {
    pub fn live_orders(&self) -> Vec<(OrderInvoiceState, NostrNote)> {
        self.live_orders.clone()
    }
    pub fn order_history(&self) -> Vec<(OrderInvoiceState, NostrNote)> {
        self.order_history.clone()
    }
    pub fn has_live_order(&self) -> bool {
        self.assigned_order.is_some()
    }
    pub fn get_live_order(&self) -> Option<(OrderInvoiceState, NostrNote)> {
        self.assigned_order.clone()
    }
    pub fn finished_loading(&self) -> bool {
        self.has_loaded
    }
}

pub enum OrderHubAction {
    FinishedLoadingRelays,
    LoadOrders(Vec<(OrderInvoiceState, NostrNote)>),
    UpdateOrder((OrderInvoiceState, NostrNote)),
    AssignOrder((OrderInvoiceState, NostrNote)),
    OrderCompleted(String),
}

impl Reducible for OrderHub {
    type Action = OrderHubAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            OrderHubAction::LoadOrders(orders) => Rc::new({
                OrderHub {
                    live_orders: {
                        let mut orders = orders.clone();
                        orders.retain(|o| {
                            o.0.order_status != OrderStatus::Completed
                                && o.0.order_status != OrderStatus::Canceled
                        });
                        orders.clone()
                    },
                    order_history: {
                        let mut orders = orders.clone();
                        orders.retain(|o| {
                            o.0.order_status == OrderStatus::Completed
                                || o.0.order_status == OrderStatus::Canceled
                        });
                        orders.clone()
                    },
                    assigned_order: self.assigned_order.clone(),
                    has_loaded: self.has_loaded.clone(),
                    hub_keys: self.hub_keys.clone(),
                }
            }),
            OrderHubAction::FinishedLoadingRelays => Rc::new(OrderHub {
                has_loaded: true,
                assigned_order: self.assigned_order.clone(),
                live_orders: self.live_orders.clone(),
                order_history: self.order_history.clone(),
                hub_keys: self.hub_keys.clone(),
            }),
            OrderHubAction::UpdateOrder((order, note)) => {
                let mut live_orders = self.live_orders.clone();
                if let Some(index) = live_orders.iter().position(|o| o.0.order_id() == order.order_id()) {
                    live_orders[index] = (order, note);
                } else {
                    live_orders.push((order, note));
                }

                Rc::new(OrderHub {
                    live_orders,
                    assigned_order: self.assigned_order.clone(),
                    order_history: self.order_history.clone(),
                    has_loaded: self.has_loaded,
                    hub_keys: self.hub_keys.clone(),
                })
            },
            OrderHubAction::OrderCompleted(completed_id) => {
                let mut live_orders = self.live_orders.clone();
                let mut order_history = self.order_history.clone();

                if let Some(completed_order) = live_orders.iter().find(|o| o.0.order_id() == completed_id).cloned() {
                    order_history.push(completed_order);
                }
                live_orders.retain(|o| o.0.order_id() != completed_id);
                Rc::new(OrderHub {
                    live_orders,
                    assigned_order: None,
                    order_history,
                    has_loaded: self.has_loaded,
                    hub_keys: self.hub_keys.clone(),
                })
            }
            OrderHubAction::AssignOrder((order, note)) => Rc::new(OrderHub {
                live_orders: {
                    let mut orders = self.live_orders.clone();
                    orders.retain(|o| o.0.order_id() != order.order_id());
                    orders.push((order.clone(), note.clone()));
                    orders
                },
                order_history: self.order_history.clone(),
                assigned_order: Some((order, note)),
                has_loaded: self.has_loaded,
                hub_keys: self.hub_keys.clone(),
            }),
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
        has_loaded: false,
        live_orders: Vec::new(),
        order_history: Vec::new(),
        assigned_order: None,
    });

    let ctx_clone = ctx.clone();
    use_effect_with((), move |_| {
        spawn_local(async move {
            let idb = OrderStateIdb::retrieve_all_from_store()
                .await
                .expect("Failed to retrieve orders");
            let history = idb
                .iter()
                .filter_map(|order_entry| {
                    Some((order_entry.parse_order().ok()?, order_entry.signed_note()))
                })
                .collect();
            ctx_clone.dispatch(OrderHubAction::LoadOrders(history));
        });
        || {}
    });

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
    let ctx_clone = ctx.clone();
    use_effect_with(keys_ctx.clone(), move |key_ctx| {
        let id_handle = id_handle.clone();
        let subscriber = subscriber.clone();
        let key_ctx = key_ctx.clone();
        spawn_local(async move {
            let last_saved = OrderStateIdb::last_saved_timestamp().await.unwrap_or(0);
            if let Some(_keys) = key_ctx.get_nostr_key() {
                let mut filter = NostrSubscription {
                    kinds: Some(vec![NOSTR_KIND_ORDER_STATE]),
                    since: Some(last_saved as u64),
                    ..Default::default()
                };
                filter.add_tag("#p", DRIVER_HUB_PUB_KEY);
                let sub: nostro2::relays::SubscribeEvent = filter.into();
                id_handle.set(sub.1.clone());
                subscriber.emit(sub);
            }
        });
        || {}
    });

    let ctx_handle = ctx.clone();
    let nostr_keys = keys_ctx.get_nostr_key().expect("Nostr keys not found");
    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            if note.kind == NOSTR_KIND_ORDER_STATE {
                if let Ok(decrypted) = hub_keys.decrypt_nip_04_content(&note) {
                    if let Ok(order_note) = NostrNote::try_from(decrypted) {
                        if let Ok(order_status) = OrderInvoiceState::try_from(&order_note) {
                            let idb = OrderStateIdb::new(order_note.clone())
                                .expect("Failed to create idb");
                            spawn_local(async move {
                                idb.save().await.expect("Failed to save order state idb");
                            });
                            match order_status.order_status {
                                OrderStatus::Completed | OrderStatus::Canceled => {
                                    ctx_handle.dispatch(OrderHubAction::OrderCompleted(
                                        order_status.order_id(),
                                    ));
                                }
                                _ => {
                                    if let Some(courier) = order_status.courier.as_ref() {
                                        if nostr_keys.public_key() == courier.pubkey {
                                            ctx_handle.dispatch(OrderHubAction::AssignOrder((
                                                order_status,
                                                order_note,
                                            )));
                                        }
                                    } else {
                                        ctx_handle.dispatch(OrderHubAction::UpdateOrder((
                                            order_status,
                                            order_note,
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        || {}
    });

    use_effect_with(relay_ctx.relay_events.clone(), move |events| {
        if let Some(event) = events.last() {
            if let nostro2::relays::RelayEvent::EndOfSubscription((_, id)) = event {
                if id == &(*sub_id) {
                    ctx_clone.dispatch(OrderHubAction::FinishedLoadingRelays);
                }
            }
        }
        || {}
    });

    html! {}
}
