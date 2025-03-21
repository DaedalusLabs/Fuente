use nostr_minions::relay_pool::NostrProps;
use nostro2::notes::NostrTag;
use std::rc::Rc;
use yew::prelude::*;

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
        let filter: nostro2::relays::SubscribeEvent = nostro2::relays::NostrSubscription {
            kinds: Some(vec![NOSTR_KIND_SERVER_CONFIG]),
            ..Default::default()
        }
        .into();
        id_handler.set(Some(filter.1.clone()));
        subscriber.emit(filter);
        || {}
    });

    let ctx_handler = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            if note.kind == NOSTR_KIND_SERVER_CONFIG {
                if let Some(conf_type_str) = note.tags.find_tags(NostrTag::Parameterized).get(2) {
                    let conf_type = AdminConfigurationType::try_from(conf_type_str.as_str())
                        .expect("Failed to parse conf type");
                    match conf_type {
                        AdminConfigurationType::ExchangeRate => {
                            if let Ok(rate) = note.content.parse::<f64>() {
                                ctx_handler.dispatch(AdminConfigsAction::UpdateExchangeRate(
                                    rate.to_string(),
                                ));
                            }
                        }
                        AdminConfigurationType::CommerceWhitelist => {
                            match serde_json::from_str::<Vec<String>>(&note.content) {
                                Ok(whitelist) => {
                                    ctx_handler.dispatch(
                                        AdminConfigsAction::UpdateCommerceWhitelist(whitelist),
                                    );
                                }
                                Err(e) => {
                                    gloo::console::error!(
                                        "Failed to parse commerce whitelist: {:?}",
                                        format!("{:?}", e)
                                    );
                                }
                            }
                        }
                        AdminConfigurationType::CourierWhitelist => {
                            // Process the whitelist normally
                            if let Ok(data) =
                                serde_json::from_str::<serde_json::Value>(&note.content)
                            {
                                // Handle JSON object format
                                if data.is_object() {
                                    // Process active list
                                    if let Some(active) =
                                        data.get("active").and_then(|v| v.as_array())
                                    {
                                        if let Ok(whitelist) = serde_json::from_value::<Vec<String>>(
                                            active.clone().into(),
                                        ) {
                                            ctx_handler.dispatch(
                                                AdminConfigsAction::UpdateCourierWhitelist(
                                                    whitelist,
                                                ),
                                            );
                                        }
                                    }
                                } else if let Ok(whitelist) =
                                    serde_json::from_str::<Vec<String>>(&note.content)
                                {
                                    // For simple array format, only update whitelist but preserve deleted keys
                                    ctx_handler.dispatch(
                                        AdminConfigsAction::UpdateCourierWhitelist(whitelist),
                                    );
                                }
                            }
                        }
                        _ => {}
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
        if let Some(nostro2::relays::RelayEvent::EndOfSubscription((_, id))) = events.last() {
            if id == sub_id.as_ref().unwrap() {
                ctx_clone.dispatch(AdminConfigsAction::FinishLoading);
            }
        }
        || {}
    });

    html! {}
}
