use std::rc::Rc;

use fuente::{
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    models::{
        admin_configs::{AdminConfiguration, AdminConfigurationType},
        nostr_kinds::NOSTR_KIND_SERVER_CONFIG,
    },
};
use nostro2::userkeys::UserKeys;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct ServerConfigs {
    admin_whitelist: Vec<String>,
    commerce_whitelist: Vec<String>,
    couriers_whitelist: Vec<String>,
    consumer_blacklist: Vec<String>,
    user_registrations: Vec<String>,
    exchange_rate: f64,
}

impl ServerConfigs {
    pub fn get_exchange_rate(&self) -> f64 {
        self.exchange_rate
    }
    pub fn set_exchange_rate(&mut self, rate: f64) {
        self.exchange_rate = rate;
    }
}

pub enum ServerConfigsAction {
    UpdateExchangeRate(f64),
}

impl Reducible for ServerConfigs {
    type Action = ServerConfigsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ServerConfigsAction::UpdateExchangeRate(rate) => {
                let mut new_state = (*self).clone();
                new_state.set_exchange_rate(rate);
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
    });

    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let keys = user_ctx.get_key().expect("No keys");

    let subscriber = relay_ctx.subscribe.clone();
    let pubkey = keys.get_public_key();
    use_effect_with((), move |_| {
        let filter = nostro2::relays::NostrFilter::default()
            .new_kind(NOSTR_KIND_SERVER_CONFIG)
            .new_tag("p", vec![pubkey]);
        subscriber.emit(filter.subscribe());
        || {}
    });

    let key_clone = keys.clone();
    let ctx_clone = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            let conf_type_tags = note.get_tags_by_id("d").expect("Failed to get tags");
            let conf_type_str = conf_type_tags.get(2).expect("Failed to get tag").as_str();
            let conf_type =
                AdminConfigurationType::try_from(conf_type_str).expect("Failed to parse conf type");
            match conf_type {
                AdminConfigurationType::ExchangeRate => {
                    let exchange_rate = key_clone
                        .decrypt_nip_04_content(&note)
                        .expect("Failed to parse note");
                    ctx_clone.dispatch(ServerConfigsAction::UpdateExchangeRate(
                        exchange_rate
                            .parse()
                            .expect("Failed to parse exchange rate"),
                    ));
                }
                _ => {}
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
