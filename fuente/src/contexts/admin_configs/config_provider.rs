use minions::relay_pool::NostrProps;
use nostro2::relays::EndOfSubscriptionEvent;
use std::rc::Rc;
use yew::{platform::spawn_local, prelude::*};

use crate::models::{AdminConfigurationType, NOSTR_KIND_SERVER_CONFIG};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminConfigs {
    has_loaded: bool,
    commerce_whitelist: Vec<String>,
    courier_whitelist: Vec<String>,
    exchange_rate: String,
}
impl AdminConfigs {
    pub fn is_loaded(&self) -> bool {
        self.has_loaded
    }
    pub fn get_exchange_rate(&self) -> f64 {
        self.exchange_rate.parse::<f64>().unwrap_or(0.0)
    }
    pub fn get_commerce_whitelist(&self) -> Vec<String> {
        self.commerce_whitelist.clone()
    }
    pub fn get_courier_whitelist(&self) -> Vec<String> {
        self.courier_whitelist.clone()
    }
}

pub enum AdminConfigsAction {
    FinishLoading,
    UpdateExchangeRate(String),
    UpdateCommerceWhitelist(Vec<String>),
    UpdateCourierWhitelist(Vec<String>),
}
impl Reducible for AdminConfigs {
    type Action = AdminConfigsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AdminConfigsAction::FinishLoading => Rc::new(AdminConfigs {
                has_loaded: true,
                ..(*self).clone()
            }),
            AdminConfigsAction::UpdateExchangeRate(rate) => Rc::new(AdminConfigs {
                exchange_rate: rate,
                ..(*self).clone()
            }),
            AdminConfigsAction::UpdateCommerceWhitelist(whitelist) => Rc::new(AdminConfigs {
                commerce_whitelist: whitelist,
                ..(*self).clone()
            }),
            AdminConfigsAction::UpdateCourierWhitelist(whitelist) => Rc::new(AdminConfigs {
                courier_whitelist: whitelist,
                ..(*self).clone()
            }),
        }
    }
}
pub type AdminConfigsStore = UseReducerHandle<AdminConfigs>;

#[function_component(AdminConfigsProvider)]
pub fn key_handler(props: &yew::html::ChildrenProps) -> Html {
    let ctx = use_reducer(|| AdminConfigs {
        has_loaded: false,
        commerce_whitelist: vec![],
        courier_whitelist: vec![],
        exchange_rate: "0".to_string(),
    });

    use_effect_with((), |_| {
        spawn_local(async move {});
        || {}
    });

    html! {
        <ContextProvider<AdminConfigsStore> context={ctx}>
            {props.children.clone()}
            <AdminConfigSync />
        </ContextProvider<AdminConfigsStore>>
    }
}

#[function_component(AdminConfigSync)]
fn admin_config_sync() -> Html {
    let ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");

    let subscriber = relay_ctx.subscribe.clone();
    let subscription_id = use_state(|| None::<String>);
    let id_handler = subscription_id.clone();
    use_effect_with((), move |_| {
        let filter = nostro2::relays::NostrSubscription {
            kinds: Some(vec![NOSTR_KIND_SERVER_CONFIG]),
            ..Default::default()
        }
        .relay_subscription();
        id_handler.set(Some(filter.1.clone()));
        subscriber.emit(filter);
        || {}
    });

    let ctx_handler = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            if note.get_kind() == NOSTR_KIND_SERVER_CONFIG {
                if let Some(conf_type_tags) = note.get_tags_by_id("d") {
                    if let Some(conf_type_str) = conf_type_tags.get(2) {
                        let conf_type = AdminConfigurationType::try_from(conf_type_str.as_str())
                            .expect("Failed to parse conf type");
                        match conf_type {
                            AdminConfigurationType::ExchangeRate => {
                                if let Ok(rate) = note.get_content().parse::<f64>() {
                                    ctx_handler.dispatch(AdminConfigsAction::UpdateExchangeRate(
                                        rate.to_string(),
                                    ));
                                    gloo::console::log!("Exchange rate updated");
                                }
                            }
                            AdminConfigurationType::CommerceWhitelist => {
                                if let Ok(whitelist) =
                                    serde_json::from_str::<Vec<String>>(&note.get_content())
                                {
                                    ctx_handler.dispatch(
                                        AdminConfigsAction::UpdateCommerceWhitelist(whitelist),
                                    );
                                    gloo::console::log!("Commerce whitelist updated");
                                }
                            }
                            AdminConfigurationType::CourierWhitelist => {
                                if let Ok(whitelist) =
                                    serde_json::from_str::<Vec<String>>(&note.get_content())
                                {
                                    ctx_handler.dispatch(
                                        AdminConfigsAction::UpdateCourierWhitelist(whitelist),
                                    );
                                    gloo::console::log!("Courier whitelist updated");
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        || {}
    });

    let relay_events = relay_ctx.relay_events.clone();
    let sub_id = subscription_id.clone();
    let ctx_clone = ctx.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(nostro2::relays::RelayEvent::EndOfSubscription(EndOfSubscriptionEvent(_, id))) =
            events.last()
        {
            if id == sub_id.as_ref().unwrap() {
                ctx_clone.dispatch(AdminConfigsAction::FinishLoading);
            }
        }
        || {}
    });

    html! {}
}
