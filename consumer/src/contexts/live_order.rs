use std::rc::Rc;

use fuente::models::{OrderInvoiceState, OrderPaymentStatus, OrderStatus, NOSTR_KIND_ORDER_STATE};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{notes::NostrNote, relays::NostrSubscription};
use yew::prelude::*;

use crate::pages::LiveOrderCheck;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveOrder {
    pub order: Option<(NostrNote, OrderInvoiceState)>,
    pub has_loaded: bool,
}

impl LiveOrder {}

pub enum LiveOrderAction {
    FinishedLoadingRelay,
    SetOrder(NostrNote, OrderInvoiceState),
    ClearOrder,
    CompleteOrder(String),
}

impl Reducible for LiveOrder {
    type Action = LiveOrderAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            LiveOrderAction::SetOrder(order, state) => Rc::new(LiveOrder {
                order: Some((order, state)),
                has_loaded: self.has_loaded,
            }),
            LiveOrderAction::ClearOrder => Rc::new(LiveOrder {
                order: None,
                has_loaded: self.has_loaded,
            }),
            LiveOrderAction::CompleteOrder(order_id) => {
                if let Some(order) = &self.order {
                    if order.1.order_id() == order_id {
                        Rc::new(LiveOrder {
                            order: None,
                            has_loaded: self.has_loaded,
                        })
                    } else {
                        self
                    }
                } else {
                    self
                }
            }
            LiveOrderAction::FinishedLoadingRelay => Rc::new(LiveOrder {
                order: self.order.clone(),
                has_loaded: true,
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
        order: None,
        has_loaded: false,
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

    let subscriber = relay_ctx.subscribe;
    let unique_notes = relay_ctx.unique_notes.clone();
    let keys_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let ctx_clone = ctx.clone();

    let id_handle = sub_id.clone();
    use_effect_with(keys_ctx.clone(), move |key_ctx| {
        if let Some(keys) = key_ctx.get_nostr_key() {
            let mut filter = NostrSubscription {
                kinds: Some(vec![NOSTR_KIND_ORDER_STATE]),
                ..Default::default()
            };
            filter.add_tag("#p", keys.public_key().as_str());
            let sub = filter.relay_subscription();
            id_handle.set(sub.1.clone());
            subscriber.emit(sub);
        }
        || {}
    });

    use_effect_with(unique_notes, move |notes| {
        if let (Some(note), Some(keys)) = (notes.last(), keys_ctx.get_nostr_key()) {
            if note.kind == NOSTR_KIND_ORDER_STATE {
                if let Ok(decrypted) = keys.decrypt_nip_04_content(&note) {
                    if let Ok(order_note) = NostrNote::try_from(decrypted) {
                        if let Ok(order_status) = OrderInvoiceState::try_from(&order_note) {
                            match (&order_status.payment_status, &order_status.order_status) {
                                (OrderPaymentStatus::PaymentFailed, _) => {}
                                (_, OrderStatus::Canceled) => {
                                    ctx.dispatch(LiveOrderAction::CompleteOrder(
                                        order_status.order_id(),
                                    ));
                                }
                                (_, OrderStatus::Completed) => {
                                    ctx.dispatch(LiveOrderAction::CompleteOrder(
                                        order_status.order_id(),
                                    ));
                                }
                                _ => {
                                    ctx.dispatch(LiveOrderAction::SetOrder(
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
            if let nostro2::relays::RelayEvent::EndOfSubscription(
                nostro2::relays::EndOfSubscriptionEvent(_, id),
            ) = event
            {
                if id == &(*sub_id) {
                    ctx_clone.dispatch(LiveOrderAction::FinishedLoadingRelay);
                }
            }
        }
        || {}
    });

    html! {}
}
