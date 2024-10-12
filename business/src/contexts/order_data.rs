use fuente::{
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    models::{
        nostr_kinds::NOSTR_KIND_ORDER_STATE,
        orders::{OrderInvoiceState, OrderPaymentStatus, OrderStateIdb, OrderStatus},
        sync::LastSyncTime,
        TEST_PUB_KEY,
    },
};
use nostro2::relays::{NostrFilter, RelayEvents};
use std::rc::Rc;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderData {
    checked_db: bool,
    checked_relay: bool,
    live_orders: Vec<OrderInvoiceState>,
    order_history: Vec<OrderInvoiceState>,
}

impl OrderData {
    pub fn checked_db(&self) -> bool {
        self.checked_db
    }
    pub fn checked_relay(&self) -> bool {
        self.checked_relay
    }
    pub fn live_orders(&self) -> Vec<OrderInvoiceState> {
        self.live_orders.clone()
    }
    pub fn filter_by_payment_status(&self, status: OrderPaymentStatus) -> Vec<OrderInvoiceState> {
        self.live_orders
            .iter()
            .filter(|o| o.get_payment_status() == status)
            .cloned()
            .collect()
    }
    pub fn filter_by_order_status(&self, status: OrderStatus) -> Vec<OrderInvoiceState> {
        self.live_orders
            .iter()
            .filter(|o| o.get_order_status() == status)
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
    UpdateCommerceOrder(OrderInvoiceState),
    LoadOrderHistory(Vec<OrderInvoiceState>),
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
                    orders.retain(|o| o.id() != order.id());
                    orders.push(order);
                    orders
                },
            }),
            OrderDataAction::LoadOrderHistory(history) => Rc::new(OrderData {
                checked_db: self.checked_db,
                checked_relay: self.checked_relay,
                live_orders: self.live_orders.clone(),
                order_history: history,
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
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");

    use_effect_with(key_ctx.get_key(), move |user_key| {
        let user_key = user_key.clone();
        spawn_local(async move {
            if let Some(keys) = user_key {
                if let Ok(live_orders) = OrderStateIdb::find_history(&keys).await {
                    gloo::console::log!("Loaded live orders", live_orders.len());
                    ctx_clone.dispatch(OrderDataAction::LoadOrderHistory(live_orders));
                };
            }
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
        if let Some(keys) = key_ctx.get_key() {
            spawn_local(async move {
                // let last_sync_time = match LastSyncTime::get_last_sync_time().await {
                //     Ok(time) => time,
                //     Err(_) => 0,
                // };
                let filter = NostrFilter::default()
                    .new_kind(NOSTR_KIND_ORDER_STATE)
                    .new_author(TEST_PUB_KEY)
                    .new_tag("p", vec![keys.get_public_key()])
                    // .new_since(last_sync_time)
                    .subscribe();
                id_handle.set(filter.id());
                subscriber.emit(filter);
                LastSyncTime::update_sync_time(nostro2::utils::get_unix_timestamp())
                    .await
                    .expect("Failed to update sync time");
            });
        }
        || {}
    });

    let ctx_clone = ctx.clone();
    let id_handle = sub_id.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(RelayEvents::EOSE(id)) = events.last() {
            if id == &(*id_handle) {
                ctx_clone.dispatch(OrderDataAction::CheckedRelay);
            }
        }
        || {}
    });
    let ctx_clone = ctx.clone();
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let keys = key_ctx.get_key();
    use_effect_with(unique_notes, move |notes| {
        if let (Some(note), Some(keys)) = (notes.last(), keys) {
            if note.get_kind() == NOSTR_KIND_ORDER_STATE {
                if let Ok(plaintext) = keys.decrypt_nip_04_content(&note) {
                    if let Ok(order) = plaintext.try_into() {
                        ctx_clone.dispatch(OrderDataAction::UpdateCommerceOrder(order));
                        let db_entry = OrderStateIdb::new(note.clone());
                        spawn_local(async move {
                            db_entry
                                .expect("Failed to create order entry")
                                .save()
                                .await
                                .expect("Failed to save order entry");
                        })
                    }
                }
            }
        }
        || {}
    });

    html! {}
}
