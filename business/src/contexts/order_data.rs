use fuente::models::{
    OrderInvoiceState, OrderPaymentStatus, OrderStateIdb, OrderStatus, NOSTR_KIND_ORDER_STATE,
    TEST_PUB_KEY,
};
use nostr_minions::{
    browser_api::IdbStoreManager, key_manager::NostrIdStore, relay_pool::NostrProps,
};
use nostro2::{
    notes::NostrNote,
    relays::{NostrSubscription, RelayEvent},
};
use std::rc::Rc;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderData {
    checked_db: bool,
    checked_relay: bool,
    live_orders: Vec<(OrderInvoiceState, NostrNote)>,
    order_history: Vec<OrderInvoiceState>,
}

impl OrderData {
    pub fn checked_db(&self) -> bool {
        self.checked_db
    }
    pub fn checked_relay(&self) -> bool {
        self.checked_relay
    }
    pub fn live_orders(&self) -> Vec<(OrderInvoiceState, NostrNote)> {
        self.live_orders.clone()
    }
    pub fn filter_by_payment_status(&self, status: OrderPaymentStatus) -> Vec<OrderInvoiceState> {
        self.live_orders()
            .iter()
            .filter(|o| o.0.payment_status == status)
            .map(|o| o.0.clone())
            .collect()
    }
    pub fn filter_by_order_status(
        &self,
        status: OrderStatus,
    ) -> Vec<(OrderInvoiceState, NostrNote)> {
        self.live_orders
            .iter()
            .filter(|o| o.0.order_status == status)
            .cloned()
            .collect()
    }
    pub fn order_history(&self) -> Vec<OrderInvoiceState> {
        self.order_history.clone()
    }
}

pub enum OrderDataAction {
    CheckedDb,
    CheckedRelay,
    UpdateCommerceOrder(NostrNote),
    LoadOrderHistory(Vec<OrderStateIdb>),
}

impl Reducible for OrderData {
    type Action = OrderDataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            OrderDataAction::CheckedDb => Rc::new(OrderData {
                checked_db: true,
                checked_relay: self.checked_relay,
                live_orders: self.live_orders.clone(),
                order_history: self.order_history.clone(),
            }),
            OrderDataAction::CheckedRelay => Rc::new(OrderData {
                checked_db: self.checked_db,
                checked_relay: true,
                live_orders: self.live_orders.clone(),
                order_history: self.order_history.clone(),
            }),
            OrderDataAction::UpdateCommerceOrder(order) => Rc::new(OrderData {
                checked_db: self.checked_db,
                checked_relay: self.checked_relay,
                order_history: self.order_history.clone(),
                live_orders: {
                    let mut orders = self.live_orders.clone();
                    if let Ok(state) = OrderInvoiceState::try_from(&order) {
                        //match state.order_status {
                        //    OrderStatus::Canceled | OrderStatus::Completed => {
                        let idb = OrderStateIdb::new(order.clone()).expect("Failed to create idb");
                        spawn_local(async move {
                            idb.save().await.expect("Failed to save order state idb");
                        });
                        //     }
                        //     _ => {}
                        // }
                        orders.retain(|o| o.0.order_id() != state.order_id());
                        orders.push((state, order));
                    }
                    orders
                },
            }),
            OrderDataAction::LoadOrderHistory(history) => Rc::new(OrderData {
                checked_db: self.checked_db,
                checked_relay: self.checked_relay,
                live_orders: {
                    let mut orders = history.clone();
                    orders.retain(|o| {
                        if let Ok(order) = o.parse_order() {
                            order.order_status != OrderStatus::Completed
                                && order.order_status != OrderStatus::Canceled
                        } else {
                            true
                        }
                    });
                    orders
                        .iter()
                        .filter_map(|o| Some((o.parse_order().ok()?, o.signed_note())))
                        .collect()
                },
                order_history: {
                    let mut orders = history.clone();
                    orders.retain(|o| {
                        if let Ok(order) = o.parse_order() {
                            order.order_status == OrderStatus::Completed
                                || order.order_status == OrderStatus::Canceled
                        } else {
                            false
                        }
                    });
                    orders
                        .iter()
                        .map(|o| o.parse_order().expect("could not parse order"))
                        .collect()
                },
            }),
        }
    }
}

pub type OrderDataStore = UseReducerHandle<OrderData>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct OrderDataChildren {
    pub children: Children,
}

#[function_component(OrderDataProvider)]
pub fn key_handler(props: &OrderDataChildren) -> Html {
    let ctx = use_reducer(|| OrderData {
        checked_relay: false,
        checked_db: false,
        live_orders: vec![],
        order_history: vec![],
    });

    let ctx_clone = ctx.clone();

    use_effect_with((), move |_| {
        spawn_local(async move {
            if let Ok(live_orders) = OrderStateIdb::retrieve_all_from_store().await {
                ctx_clone.dispatch(OrderDataAction::LoadOrderHistory(live_orders));
            };
            ctx_clone.dispatch(OrderDataAction::CheckedDb);
        });
    });

    html! {
        <ContextProvider<OrderDataStore> context={ctx}>
            {props.children.clone()}
            <OrderDataSync />
        </ContextProvider<OrderDataStore>>
    }
}

#[function_component(OrderDataSync)]
pub fn commerce_data_sync() -> Html {
    let ctx = use_context::<OrderDataStore>().expect("Commerce context not found");
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let sub_id = use_state(|| "".to_string());

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let relay_events = relay_ctx.relay_events.clone();

    let id_handle = sub_id.clone();
    use_effect_with(key_ctx, move |key_ctx| {
        if let Some(keys) = key_ctx.get_nostr_key() {
            spawn_local(async move {
                let last_save_time = OrderStateIdb::last_saved_timestamp().await.unwrap_or(0);
                // let unix_time = web_sys::js_sys::Date::new_0();
                // let twelve_hours_ago_unix = (unix_time.get_time() as u64  / 1000)- 43200;

                let mut filter = NostrSubscription {
                    kinds: Some(vec![NOSTR_KIND_ORDER_STATE]),
                    authors: Some(vec![TEST_PUB_KEY.to_string()]),
                    // since: Some(twelve_hours_ago_unix),
                    since: Some(last_save_time as u64),
                    ..Default::default()
                };
                filter.add_tag("#p", keys.public_key().as_str());
                let relay_sub: nostro2::relays::SubscribeEvent = filter.into();
                id_handle.set(relay_sub.1.clone());
                subscriber.emit(relay_sub.into());
            });
        }
        || {}
    });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvent::EndOfSubscription((_, id))) = events.last() {
            if id == &(*id_handle) {
                ctx_clone.dispatch(OrderDataAction::CheckedRelay);
            }
        }
        || {}
    });
    let ctx_clone = ctx.clone();
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let keys = key_ctx.get_nostr_key();
    use_effect_with(unique_notes, move |notes| {
        if let (Some(note), Some(keys)) = (notes.last(), keys) {
            if note.kind == NOSTR_KIND_ORDER_STATE {
                if let Ok(plaintext) = keys.decrypt_nip_04_content(&note) {
                    if let Ok(order) = plaintext.try_into() {
                        ctx_clone.dispatch(OrderDataAction::UpdateCommerceOrder(order));
                    }
                }
            }
        }
        || {}
    });

    html! {}
}
