use fuente::{mass::LoadingScreen, models::NOSTR_KIND_DRIVER_STATE};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::relays::NostrSubscription;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    contexts::{CommerceDataStore, LiveOrderStore},
    pages::{
        CartPage, CommercePage, FavoritesPage, HistoryPage, HomePage, LiveOrderCheck,
        SettingsPageComponent,
    },
};

#[derive(Clone, Routable, PartialEq)]
pub enum ConsumerRoute {
    #[at("/")]
    Home,
    #[at("/history")]
    History,
    #[at("/settings")]
    Settings,
    #[at("/favorites")]
    Favorites,
    #[at("/cart")]
    Cart,
    #[at("/commerce/:commerce_id")]
    Commerce { commerce_id: String },
    #[at("/order/:order_id")]
    Order { order_id: String },
}

#[function_component(ConsumerPages)]
pub fn consumer_pages() -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("No order context found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    if !order_ctx.has_loaded || !commerce_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }

    html! {
        <>
        <Switch<ConsumerRoute> render = { move |switch: ConsumerRoute| {
                match switch {
                    ConsumerRoute::Home => html!{<HomePage />},
                    ConsumerRoute::History => html!{<HistoryPage />},
                    ConsumerRoute::Settings => html!{<SettingsPageComponent />},
                    ConsumerRoute::Favorites => html!{<FavoritesPage />},
                    ConsumerRoute::Cart => html!{
                        <LiveOrderCheck />
                    },
                    ConsumerRoute::Commerce { commerce_id } => html!{
                        <CommercePage {commerce_id} />
                    },
                    ConsumerRoute::Order { order_id: _ } => html!{
                        <LiveOrderCheck />
                    }
                }
            }}
        />
        </>
    }
}
