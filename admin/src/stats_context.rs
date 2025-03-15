use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use fuente::models::{
    OrderStatus, PlatformStatIdb, NOSTR_KIND_CONSUMER_REGISTRY, NOSTR_KIND_ORDER_STATE,
};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{
    notes::{NostrNote, NostrTag},
    relays::SubscribeEvent,
};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct PlatformStats {
    loaded: bool,
    orders: HashMap<String, OrderStatus>,
    users: Vec<NostrNote>,
}

impl PlatformStats {
    pub fn count_completed_orders(&self) -> usize {
        self.orders
            .iter()
            .filter(|(_, status)| **status == OrderStatus::Completed)
            .count()
    }
    pub fn count_pending_orders(&self) -> usize {
        self.orders
            .iter()
            .filter(|(_, status)| **status == OrderStatus::Canceled)
            .count()
    }
    pub fn count_users(&self) -> usize {
        self.users.len()
    }
    pub fn loading(&self) -> bool {
        !self.loaded
    }
}

pub enum PlatformStatsAction {
    FinishLoading,
    LoadStats(Vec<NostrNote>, HashMap<String, OrderStatus>),
    AddOrder(NostrNote),
    AddUser(NostrNote),
}

impl Reducible for PlatformStats {
    type Action = PlatformStatsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            PlatformStatsAction::FinishLoading => Rc::new(Self {
                loaded: true,
                orders: self.orders.clone(),
                users: self.users.clone(),
            }),
            PlatformStatsAction::LoadStats(users, orders) => Rc::new(Self {
                loaded: self.loaded,
                orders,
                users,
            }),
            PlatformStatsAction::AddOrder(note) => {
                let mut orders = self.orders.clone();
                let order_id = note.tags.find_first_parameter().unwrap_or_default();
                let order_status_str = note
                    .tags
                    .find_tags(NostrTag::Custom("status"))
                    .first()
                    .cloned()
                    .unwrap_or_default();
                let order_status =
                    OrderStatus::try_from(&order_status_str).unwrap_or(OrderStatus::Pending);
                orders.insert(order_id, order_status);
                Rc::new(Self {
                    orders,
                    users: self.users.clone(),
                    loaded: self.loaded,
                })
            }
            PlatformStatsAction::AddUser(note) => {
                let mut users = self.users.clone();
                users.push(note);
                Rc::new(Self {
                    users,
                    orders: self.orders.clone(),
                    loaded: self.loaded,
                })
            }
        }
    }
}

pub type PlatformStatsStore = UseReducerHandle<PlatformStats>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct PlatformStatsChildren {
    pub children: Children,
}

#[function_component(PlatformStatsProvider)]
pub fn key_handler(props: &PlatformStatsChildren) -> Html {
    let ctx = use_reducer(|| PlatformStats {
        loaded: false,
        orders: HashMap::new(),
        users: vec![],
    });

    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let subscription_id = use_state(|| HashSet::new());
    let sub_handler = subscription_id.clone();

    let ctx_handle = ctx.clone();
    use_effect_with(user_ctx.get_identity().cloned(), move |keys| {
        if keys.is_some() {
            spawn_local(async move {
                let stats_history = PlatformStatIdb::find_history()
                    .await
                    .expect("Error finding history");
                let users = stats_history
                    .iter()
                    .filter(|note| note.kind == NOSTR_KIND_CONSUMER_REGISTRY)
                    .cloned()
                    .collect::<Vec<NostrNote>>();
                let mut orders = HashMap::new();
                stats_history
                    .iter()
                    .filter(|note| note.kind == NOSTR_KIND_ORDER_STATE)
                    .for_each(|note| {
                        let order_id = note.tags.find_first_parameter().unwrap_or_default();
                        let order_status_str = note
                            .tags
                            .find_tags(NostrTag::Custom("status"))
                            .first()
                            .cloned()
                            .unwrap_or_default();
                        let order_status = OrderStatus::try_from(&order_status_str)
                            .unwrap_or(OrderStatus::Pending);
                        orders.insert(order_id, order_status);
                    });
                ctx_handle.dispatch(PlatformStatsAction::LoadStats(users, orders));
            });
        }
        || {}
    });
    let subscriber = relay_ctx.subscribe.clone();
    use_effect_with(user_ctx.get_identity().cloned(), move |key| {
        if key.is_some() {
            spawn_local(async move {
                let last_save_time = PlatformStatIdb::last_saved_timestamp().await.unwrap_or(0);
                let users_filter: SubscribeEvent = nostro2::relays::NostrSubscription {
                    kinds: Some(vec![NOSTR_KIND_CONSUMER_REGISTRY]),
                    since: Some(last_save_time as u64),
                    ..Default::default()
                }
                .into();
                let order_filter: SubscribeEvent = nostro2::relays::NostrSubscription {
                    kinds: Some(vec![NOSTR_KIND_ORDER_STATE]),
                    since: Some(last_save_time as u64),
                    ..Default::default()
                }
                .into();
                let mut sub_id = HashSet::new();
                sub_id.insert(users_filter.1.clone());
                sub_id.insert(order_filter.1.clone());
                sub_handler.set(sub_id);
                subscriber.emit(users_filter.into());
                subscriber.emit(order_filter.into());
            });
        }
        || {}
    });

    let key_clone = user_ctx.clone();
    let ctx_clone = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let (Some(note), true) = (notes.last(), key_clone.get_identity().is_some()) {
            match note.kind {
                NOSTR_KIND_CONSUMER_REGISTRY => {
                    let platform_idb = PlatformStatIdb::new(note.clone());
                    let note_clone = note.clone();
                    spawn_local(async move {
                        if let Ok(_) = platform_idb
                            .expect("Error creating platform idb")
                            .save()
                            .await
                        {
                            ctx_clone.dispatch(PlatformStatsAction::AddUser(note_clone));
                        }
                    });
                }
                NOSTR_KIND_ORDER_STATE => {
                    let platform_idb = PlatformStatIdb::new(note.clone());
                    let note_clone = note.clone();
                    spawn_local(async move {
                        if let Ok(_) = platform_idb
                            .expect("Error creating platform idb")
                            .save()
                            .await
                        {
                            ctx_clone.dispatch(PlatformStatsAction::AddOrder(note_clone));
                        }
                    });
                }
                _ => {}
            }
        }
        || {}
    });
    let relay_events = relay_ctx.relay_events.clone();
    let sub_id = subscription_id.clone();
    let ctx_clone = ctx.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(nostro2::relays::RelayEvent::EndOfSubscription((_, id))) = events.last() {
            let mut new_set = (*sub_id).clone();
            if new_set.remove(id) {
                gloo::console::log!("Finished subscription");
                if new_set.is_empty() {
                    ctx_clone.dispatch(PlatformStatsAction::FinishLoading);
                    gloo::console::log!("Finished loading");
                }
                sub_id.set(new_set);
            }
        }
        || {}
    });

    html! {
        <ContextProvider<PlatformStatsStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<PlatformStatsStore>>
    }
}
