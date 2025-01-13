use std::{collections::HashSet, rc::Rc};

use fuente::models::{
    AdminConfigurationType, CommerceProfile, DriverProfile, DRIVER_HUB_PRIV_KEY,
    DRIVER_HUB_PUB_KEY, NOSTR_KIND_COMMERCE_PROFILE, NOSTR_KIND_COURIER_PROFILE,
    NOSTR_KIND_SERVER_CONFIG, TEST_PUB_KEY,
};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{
    keypair::NostrKeypair,
    notes::{NostrNote, NostrTag},
};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ServerConfigs {
    admin_whitelist: Vec<String>,
    commerce_whitelist: Vec<String>,
    commerces: Vec<NostrNote>,
    couriers: Vec<(NostrNote, DriverProfile)>,
    couriers_whitelist: Vec<String>,
    consumer_blacklist: Vec<String>,
    user_registrations: Vec<String>,
    exchange_rate: f64,
    loaded: bool,
}

impl ServerConfigs {
    pub fn find_courier(&self, pubkey: &str) -> Option<(NostrNote, DriverProfile)> {
        self.couriers
            .iter()
            .find(|note| note.0.pubkey == pubkey)
            .cloned()
    }
    pub fn get_exchange_rate(&self) -> f64 {
        self.exchange_rate
    }
    pub fn set_exchange_rate(&mut self, rate: f64) {
        self.exchange_rate = rate;
    }
    pub fn get_unregistered_commerces(&self) -> Vec<NostrNote> {
        let mut unregistered_users = vec![];
        for note in self.commerces.iter() {
            if !self.commerce_whitelist.contains(&note.pubkey) {
                unregistered_users.push(note.clone());
            }
        }
        unregistered_users
    }
    pub fn get_whitelisted_commerces(&self) -> Vec<NostrNote> {
        let mut commerces = vec![];
        for note in self.commerces.iter() {
            if self.commerce_whitelist.contains(&note.pubkey) {
                commerces.push(note.clone());
            }
        }
        commerces
    }
    pub fn get_whitelisted_couriers(&self) -> Vec<(NostrNote, DriverProfile)> {
        let mut couriers = vec![];
        for note in self.couriers.iter() {
            if self.couriers_whitelist.contains(&note.0.pubkey) {
                couriers.push(note.clone());
            }
        }
        couriers
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
    pub fn loading(&self) -> bool {
        !self.loaded
    }
}

pub enum ServerConfigsAction {
    FinishLoading,
    UpdateExchangeRate(f64),
    UpdateCommerceWhitelist(Vec<String>),
    UpdateCouriersWhitelist(Vec<String>),
    AddCommerce(NostrNote),
    AddCourier((NostrNote, DriverProfile)),
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
            ServerConfigsAction::AddCommerce(note) => {
                let mut new_state = (*self).clone();
                new_state.commerces.retain(|n| n.pubkey != note.pubkey);
                new_state.commerces.push(note);
                Rc::new(new_state)
            }
            ServerConfigsAction::AddCourier(note) => {
                let mut new_state = (*self).clone();
                new_state.couriers.retain(|n| n.0.pubkey != note.0.pubkey);
                new_state.couriers.push(note);
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
        commerces: vec![],
        couriers: vec![],
        consumer_blacklist: vec![],
        user_registrations: vec![],
        exchange_rate: 0.0,
        loaded: false,
    });

    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let subscription_id = use_state(|| HashSet::new());
    let sub_handler = subscription_id.clone();

    use_effect_with((), move |_| || {});
    let subscriber = relay_ctx.subscribe.clone();
    use_effect_with(user_ctx.get_nostr_key().clone(), move |key| {
        if let Some(_) = key {
            let mut courier_filter = nostro2::relays::NostrSubscription {
                kinds: Some(vec![NOSTR_KIND_COURIER_PROFILE]),
                ..Default::default()
            };
            courier_filter.add_tag("#p", DRIVER_HUB_PUB_KEY);
            let commerce_filter = nostro2::relays::NostrSubscription {
                kinds: Some(vec![NOSTR_KIND_COMMERCE_PROFILE]),
                ..Default::default()
            };
            let filter = nostro2::relays::NostrSubscription {
                kinds: Some(vec![NOSTR_KIND_SERVER_CONFIG]),
                authors: Some(vec![TEST_PUB_KEY.to_string()]),
                ..Default::default()
            };
            let subscription: nostro2::relays::SubscribeEvent = filter.into();
            let commerce_filter: nostro2::relays::SubscribeEvent = commerce_filter.into();
            let courier_filter: nostro2::relays::SubscribeEvent = courier_filter.into();
            let mut new_set = (*sub_handler).clone();
            new_set.insert(subscription.1.clone());
            new_set.insert(commerce_filter.1.clone());
            new_set.insert(courier_filter.1.clone());
            sub_handler.set(new_set);
            subscriber.emit(commerce_filter.into());
            subscriber.emit(courier_filter.into());
            subscriber.emit(subscription.into());
        }
        || {}
    });

    let key_clone = user_ctx.clone();
    let ctx_clone = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        let driver_hub_key =
            NostrKeypair::new(DRIVER_HUB_PRIV_KEY).expect("Failed to create user keys");
        if let (Some(note), Some(_key_clone)) = (notes.last(), key_clone.get_nostr_key()) {
            if note.kind == NOSTR_KIND_COMMERCE_PROFILE {
                if let Ok(_) = CommerceProfile::try_from(note.clone()) {
                    ctx_clone.dispatch(ServerConfigsAction::AddCommerce(note.clone()));
                }
            }
            if note.kind == NOSTR_KIND_COURIER_PROFILE {
                let cleartext = driver_hub_key
                    .decrypt_nip_04_content(&note)
                    .expect("Failed to decrypt");
                let giftwrapped_note = NostrNote::try_from(cleartext).expect("Failed to parse");

                if let Ok(profile) = DriverProfile::try_from(giftwrapped_note.content.clone()) {
                    ctx_clone.dispatch(ServerConfigsAction::AddCourier((
                        giftwrapped_note.clone(),
                        profile,
                    )));
                }
            }
            if note.kind == NOSTR_KIND_SERVER_CONFIG {
                if let Some(conf_type_str) = note.tags.find_tags(NostrTag::Parameterized).get(2) {
                    let conf_type = AdminConfigurationType::try_from(conf_type_str.as_str())
                        .expect("Failed to parse conf type");
                    match conf_type {
                        AdminConfigurationType::ExchangeRate => {
                            if let Ok(rate) = note.content.parse::<f64>() {
                                ctx_clone.dispatch(ServerConfigsAction::UpdateExchangeRate(rate));
                            }
                        }
                        AdminConfigurationType::CommerceWhitelist => {
                            if let Ok(whitelist) =
                                serde_json::from_str::<Vec<String>>(&note.content)
                            {
                                ctx_clone.dispatch(ServerConfigsAction::UpdateCommerceWhitelist(
                                    whitelist,
                                ));
                            }
                        }
                        AdminConfigurationType::CourierWhitelist => {
                            if let Ok(whitelist) =
                                serde_json::from_str::<Vec<String>>(&note.content)
                            {
                                ctx_clone.dispatch(ServerConfigsAction::UpdateCouriersWhitelist(
                                    whitelist,
                                ));
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
            let mut new_set = (*sub_id).clone();
            if new_set.remove(id) {
                if new_set.is_empty() {
                    ctx_clone.dispatch(ServerConfigsAction::FinishLoading);
                }
                sub_id.set(new_set);
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
