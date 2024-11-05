use std::rc::Rc;

use fuente::{
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    models::{admin_configs::AdminConfigurationType, nostr_kinds::NOSTR_KIND_SERVER_CONFIG},
};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ServerConfigs {
    admin_whitelist: Vec<String>,
    commerce_whitelist: Vec<String>,
    couriers_whitelist: Vec<String>,
    consumer_blacklist: Vec<String>,
    user_registrations: Vec<String>,
    exchange_rate: f64,
    loaded: bool,
}

impl ServerConfigs {
    pub fn get_exchange_rate(&self) -> f64 {
        self.exchange_rate
    }
    pub fn set_exchange_rate(&mut self, rate: f64) {
        self.exchange_rate = rate;
    }
    pub fn check_commerce_whitelist(&self, pubkey: &str) -> bool {
        self.commerce_whitelist.contains(&pubkey.to_string())
    }
    pub fn get_commerce_whitelist(&self) -> Vec<String> {
        self.commerce_whitelist.clone()
    }
    pub fn check_courier_whitelist(&self, pubkey: &str) -> bool {
        self.couriers_whitelist.contains(&pubkey.to_string())
    }
    pub fn check_consumer_blacklist(&self, pubkey: &str) -> bool {
        self.consumer_blacklist.contains(&pubkey.to_string())
    }
    pub fn check_admin_whitelist(&self, pubkey: &str) -> bool {
        self.admin_whitelist.contains(&pubkey.to_string())
    }
    pub fn get_user_registrations(&self) -> Vec<String> {
        self.user_registrations.clone()
    }
    pub fn get_admin_whitelist(&self) -> Vec<String> {
        self.admin_whitelist.clone()
    }
    pub fn get_couriers_whitelist(&self) -> Vec<String> {
        self.couriers_whitelist.clone()
    }
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

pub enum ServerConfigsAction {
    FinishLoading,
    UpdateExchangeRate(f64),
    UpdateCommerceWhitelist(Vec<String>),
    UpdateCouriersWhitelist(Vec<String>),
}

impl Reducible for ServerConfigs {
    type Action = ServerConfigsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ServerConfigsAction::FinishLoading => {
                let mut new_state = (*self).clone();
                new_state.loaded = true;
                Rc::new(new_state)
            }
            ServerConfigsAction::UpdateExchangeRate(rate) => {
                let mut new_state = (*self).clone();
                new_state.set_exchange_rate(rate);
                Rc::new(new_state)
            }
            ServerConfigsAction::UpdateCommerceWhitelist(whitelist) => {
                let mut new_state = (*self).clone();
                new_state.commerce_whitelist = whitelist;
                Rc::new(new_state)
            }
            ServerConfigsAction::UpdateCouriersWhitelist(whitelist) => {
                let mut new_state = (*self).clone();
                new_state.couriers_whitelist = whitelist;
                Rc::new(new_state)
            }
        }
    }
}

pub type ServerConfigsStore = UseReducerHandle<ServerConfigs>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct ServerConfigsChildren {
    pub children: Children,
}

#[function_component(ServerConfigsProvider)]
pub fn key_handler(props: &ServerConfigsChildren) -> Html {
    let ctx = use_reducer(|| ServerConfigs {
        admin_whitelist: vec![],
        commerce_whitelist: vec![],
        couriers_whitelist: vec![],
        consumer_blacklist: vec![],
        user_registrations: vec![],
        exchange_rate: 0.0,
        loaded: false,
    });

    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let subscription_id = use_state(|| None::<String>);
    let sub_handler = subscription_id.clone();

    let subscriber = relay_ctx.subscribe.clone();
    use_effect_with(user_ctx.clone(), move |keys| {
        if let Some(keys) = keys.get_nostr_key() {
            let filter = nostro2::relays::NostrFilter::default()
                .new_kind(NOSTR_KIND_SERVER_CONFIG)
                .new_tag("p", vec![keys.get_public_key()]);
            let subscription = filter.subscribe();
            sub_handler.set(Some(subscription.id()));
            subscriber.emit(subscription);
        }
        || {}
    });

    let key_clone = user_ctx.clone();
    let ctx_clone = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let (Some(note), Some(key_clone)) = (notes.last(), key_clone.get_nostr_key()) {
            if note.get_kind() == NOSTR_KIND_SERVER_CONFIG {
                let conf_type_tags = note.get_tags_by_id("d").expect("Failed to get tags");
                let conf_type_str = conf_type_tags.get(2).expect("Failed to get tag").as_str();
                let conf_type = AdminConfigurationType::try_from(conf_type_str)
                    .expect("Failed to parse conf type");
                let decrypted_note = key_clone
                    .decrypt_nip_04_content(&note)
                    .expect("Failed to parse note");
                match conf_type {
                    AdminConfigurationType::ExchangeRate => {
                        ctx_clone.dispatch(ServerConfigsAction::UpdateExchangeRate(
                            decrypted_note
                                .parse()
                                .expect("Failed to parse exchange rate"),
                        ));
                    }
                    AdminConfigurationType::CommerceWhitelist => {
                        gloo::console::info!(
                            "Updating commerce whitelist",
                            format!("{:?}", decrypted_note)
                        );
                        ctx_clone.dispatch(ServerConfigsAction::UpdateCommerceWhitelist(
                            serde_json::from_str(&decrypted_note)
                                .expect("Failed to parse whitelist"),
                        ));
                    }
                    AdminConfigurationType::CourierWhitelist => {
                        gloo::console::info!(
                            "Updating couriers whitelist",
                            format!("{:?}", decrypted_note)
                        );
                        ctx_clone.dispatch(ServerConfigsAction::UpdateCouriersWhitelist(
                            serde_json::from_str(&decrypted_note)
                                .expect("Failed to parse whitelist"),
                        ));
                    }
                    _ => {}
                }
            }
        }
        || {}
    });
    let relay_events = relay_ctx.relay_events.clone();
    let sub_id = subscription_id.clone();
    let ctx_clone = ctx.clone();
    use_effect_with(relay_events, move |events| {
        if let Some(nostro2::relays::RelayEvents::EOSE(id)) = events.last() {
            if id == sub_id.as_ref().unwrap() {
                ctx_clone.dispatch(ServerConfigsAction::FinishLoading);
            }
        }
        || {}
    });

    html! {
        <ContextProvider<ServerConfigsStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<ServerConfigsStore>>


    }
}
