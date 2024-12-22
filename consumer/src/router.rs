use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{CartPage, CommercePage, FavoritesPage, HistoryPage, HomePage, SettingsPageComponent};

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
