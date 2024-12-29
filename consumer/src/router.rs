use yew::prelude::*;
use yew_router::prelude::*;

use crate::{contexts::{CommerceDataStore, LiveOrderStore}, pages::{CartPage, CommercePage, FavoritesPage, HistoryPage, HomePage, LiveOrderCheck, SettingsPageComponent}};

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
}

#[function_component(ConsumerPages)]
pub fn consumer_pages() -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("No order context found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    if !order_ctx.has_loaded || !commerce_ctx.finished_loading() {
        return html! {<div>{"Loading..."}</div>};
    }
    if let Some(order) = order_ctx.order.as_ref() {
        return html! {<LiveOrderCheck />};
    }
    
    html! {
        <Switch<ConsumerRoute> render = { move |switch: ConsumerRoute| {
                match switch {
                    ConsumerRoute::Home => html!{<HomePage />},
                    ConsumerRoute::History => html!{<HistoryPage />},
                    ConsumerRoute::Settings => html!{<SettingsPageComponent />},
                    ConsumerRoute::Favorites => html!{<FavoritesPage />},
                    ConsumerRoute::Cart => html!{<CartPage />},
                    ConsumerRoute::Commerce { commerce_id } => html!{
                        <CommercePage {commerce_id} />
                    },
                }
            }}
        />
    }
}
