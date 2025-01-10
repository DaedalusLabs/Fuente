use std::rc::Rc;

use fuente::models::{
    OrderInvoiceState, OrderPaymentStatus, OrderStateIdb, OrderStatus, NOSTR_KIND_DRIVER_STATE,
    NOSTR_KIND_ORDER_STATE,
};
use nostr_minions::{
    browser_api::IdbStoreManager, key_manager::NostrIdStore, relay_pool::NostrProps,
};
use nostro2::{notes::NostrNote, relays::NostrSubscription};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveOrder {
    pub live_orders: Vec<(NostrNote, OrderInvoiceState)>,
    pub has_loaded: bool,
}

impl LiveOrder {}

pub enum LiveOrderAction {
    FinishedLoadingRelay,
    UpdateOrder(NostrNote, OrderInvoiceState),
    LoadOrders(Vec<(NostrNote, OrderInvoiceState)>),
    CompleteOrder(String),
}

impl Reducible for LiveOrder {
    type Action = LiveOrderAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            LiveOrderAction::UpdateOrder(order, state) => Rc::new(LiveOrder {
                has_loaded: self.has_loaded,
                live_orders: {
                    let mut orders = self.live_orders.clone();
                    orders.retain(|o| o.1.order_id() != state.order_id());
                    orders.push((order, state));
                    orders
                },
            }),
            LiveOrderAction::CompleteOrder(order_id) => Rc::new(LiveOrder {
                live_orders: {
                    let mut orders = self.live_orders.clone();
                    orders.retain(|o| o.1.order_id() != order_id);
                    orders
                },
                has_loaded: self.has_loaded,
            }),
            LiveOrderAction::FinishedLoadingRelay => Rc::new(LiveOrder {
                has_loaded: true,
                live_orders: self.live_orders.clone(),
            }),
            LiveOrderAction::LoadOrders(orders) => Rc::new(LiveOrder {
                has_loaded: self.has_loaded,
                live_orders: orders,
            }),
        }
    }
}

pub type LiveOrderStore = UseReducerHandle<LiveOrder>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct LiveOrderChildren {
    pub children: Children,
}

#[function_component(LiveOrderProvider)]
pub fn key_handler(props: &LiveOrderChildren) -> Html {
    let ctx = use_reducer(|| LiveOrder {
        has_loaded: false,
        live_orders: vec![],
    });

    let order_ctx = ctx.clone();
    use_effect_with((), move |_| {
        let order_ctx = order_ctx.clone();
        spawn_local(async move {
            match OrderStateIdb::retrieve_all_from_store().await {
                Ok(orders) => {
                    order_ctx.dispatch(LiveOrderAction::LoadOrders(
                        orders
                            .iter()
                            .filter_map(|o| Some((o.signed_note(), o.parse_order().ok()?)))
                            .collect(),
                    ));
                }
                Err(e) => {
                    gloo::console::error!("Failed to load live orders:", e);
                }
            }
        });
        || {}
    });

    html! {
        <ContextProvider<LiveOrderStore> context={ctx.clone()}>
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

    let subscriber = relay_ctx.subscribe.clone();
    let unique_notes = relay_ctx.unique_notes.clone();
    let keys_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let ctx_clone = ctx.clone();

    let id_handle = sub_id.clone();
    use_effect_with(keys_ctx.clone(), move |key_ctx| {
        if let Some(keys) = key_ctx.get_nostr_key() {
            spawn_local(async move {
                let last_sync_time = OrderStateIdb::last_saved_timestamp().await.unwrap_or(0);
                let mut filter = NostrSubscription {
                    kinds: Some(vec![NOSTR_KIND_ORDER_STATE]),
                    since: Some(last_sync_time as u64),
                    ..Default::default()
                };
                filter.add_tag("#p", keys.public_key().as_str());
                let sub: nostro2::relays::SubscribeEvent = filter.into();
                id_handle.set(sub.1.clone());
                subscriber.emit(sub);
            });
        }
        || {}
    });

    let keys_clone = keys_ctx.get_nostr_key().clone();
    let subscriber_clone = relay_ctx.subscribe;
    use_effect_with(ctx.live_orders.clone(), move |order| {
        if let Some((_note, state)) = order.last() {
            if let Some(_courier_note) = state.courier.clone() {
                let mut filter = NostrSubscription {
                    kinds: Some(vec![NOSTR_KIND_DRIVER_STATE]),
                    ..Default::default()
                };
                filter.add_tag("#p", keys_clone.as_ref().unwrap().public_key().as_str());
                subscriber_clone.emit(filter.into());
            }
        }
        || {}
    });

    use_effect_with(unique_notes, move |notes| {
        if let (Some(note), Some(keys)) = (notes.last(), keys_ctx.get_nostr_key()) {
            if note.kind == NOSTR_KIND_ORDER_STATE {
                if let Ok(decrypted) = keys.decrypt_nip_04_content(&note) {
                    if let Ok(order_note) = NostrNote::try_from(decrypted) {
                        if let Ok(order_status) = OrderInvoiceState::try_from(&order_note) {
                            match OrderStateIdb::new(order_note.clone()) {
                                Ok(order_idb) => {
                                    spawn_local(async move {
                                        order_idb
                                            .save()
                                            .await
                                            .expect("Failed to save order state idb");
                                    });
                                }
                                Err(e) => {
                                    gloo::console::error!(
                                        "Failed to create order state idb: {:?}",
                                        e
                                    );
                                }
                            }
                            match (&order_status.payment_status, &order_status.order_status) {
                                (OrderPaymentStatus::PaymentFailed, _) => {}
                                (_, OrderStatus::Canceled) => {
                                    // Save to IDB but don't complete the order immediately
                                    // Use SetOrder instead of CompleteOrder to keep the order in context
                                    ctx.dispatch(LiveOrderAction::UpdateOrder(
                                        order_note,
                                        order_status,
                                    ));
                                }
                                (OrderPaymentStatus::PaymentSuccess, OrderStatus::Completed) => {
                                    gloo::console::log!("Setting completed order for rating");
                                    ctx.dispatch(LiveOrderAction::UpdateOrder(
                                        order_note,
                                        order_status,
                                    ));
                                }
                                _ => {
                                    ctx.dispatch(LiveOrderAction::UpdateOrder(
                                        order_note,
                                        order_status,
                                    ));
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
                    ctx_clone.dispatch(LiveOrderAction::FinishedLoadingRelay);
                }
            }
        }
        || {}
    });

    html! {}
}
