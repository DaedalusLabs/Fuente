use fuente::mass::LoadingScreen;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    contexts::{CommerceDataStore, LiveOrderStore},
    pages::{
        CartPage, CommercePage, FavoritesPage, HistoryPage, HomePage, LiveOrderCheck, SettingsPageComponent
    },
};
use fuente::mass::AppLink;

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
    html! {
        <div class="min-h-screen flex flex-col">
        <FuenteHeader />
        <div class="flex-1" >
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
                        ConsumerRoute::Order { order_id: _ } => html!{
                            <LiveOrderCheck />
                        }
                    }
                }}
            />
        </div>
        <FuenteFooter />
        </div>
    }
}
#[function_component(FuenteHeader)]
pub fn header() -> Html {
    html! {
    <header class="container mx-auto py-10 flex justify-center lg:justify-between">
       <AppLink<ConsumerRoute>
           class="hidden lg:flex"
           selected_class=""
           route={ConsumerRoute::Home}>
               <img src="/templates/img/Logo Fuente.jpeg" alt="Logo Fuente" class="w-40"/>
       </AppLink<ConsumerRoute>>

        <div class="flex flex-col lg:flex-row gap-4 flex-1 items-center justify-end w-full">
            <div class="relative flex items-center w-full max-w-sm mx-auto lg:ml-auto lg:mx-0 gap-4">
                <div class="relative flex items-center w-full">
                    <input
                        type="text"
                        class="w-full pl-10 pr-10 py-3 border-2 border-fuente rounded-xl"
                    />
                        <svg
                            class="absolute right-4 h-6 w-6 text-fuente pointer-events-none"
                            fill="none"
                            height="24"
                            stroke="currentColor"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            viewBox="0 0 24 24"
                            width="24"
                            xmlns="http://www.w3.org/2000/svg"
                        >
                            <circle cx="11" cy="11" r="8" />
                            <line x1="21" x2="16.65" y1="21" y2="16.65" />
                        </svg>
                </div>

                <button class="lg:hidden">
                    <svg
                        viewBox="0 0 32 32"
                        xmlns="http://www.w3.org/2000/svg"
                        enable-background="new 0 0 32 32"
                        class="bg-fuente h-14 w-14 p-2 rounded-xl"
                    >
                        <path
                            d="M4 10h24a2 2 0 0 0 0-4H4a2 2 0 0 0 0 4zm24 4H4a2 2 0 0 0 0 4h24a2 2 0 0 0 0-4zm0 8H4a2 2 0 0 0 0 4h24a2 2 0 0 0 0-4z"
                            fill="#ffffff"
                            class="fill-000000">
                        </path>
                    </svg>
                </button>
            </div>
            <div class="flex gap-5">
                <AppLink<ConsumerRoute>
                    class=""
                    selected_class=""
                    route={ConsumerRoute::Settings}>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"
                        class="size-6 w-10 h-10 text-fuente hover:cursor-pointer">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" />
                    </svg>
                </AppLink<ConsumerRoute>>

                <AppLink<ConsumerRoute>
                    class=""
                    selected_class=""
                    route={ConsumerRoute::Cart}>
                    <svg viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-fuente hover:cursor-pointer">
                        <rect fill="none" height="256" width="256" />
                        <path d="M184,184H69.8L41.9,30.6A8,8,0,0,0,34.1,24H16" fill="none" stroke="currentColor"
                            stroke-linecap="round" stroke-linejoin="round" stroke-width="16" />
                        <circle cx="80" cy="204" fill="none" r="20" stroke="currentColor" stroke-linecap="round"
                            stroke-linejoin="round" stroke-width="16" />
                        <circle cx="184" cy="204" fill="none" r="20" stroke="currentColor" stroke-linecap="round"
                            stroke-linejoin="round" stroke-width="16" />
                        <path d="M62.5,144H188.1a15.9,15.9,0,0,0,15.7-13.1L216,64H48" fill="none" stroke="currentColor"
                            stroke-linecap="round" stroke-linejoin="round" stroke-width="16" />
                    </svg>
                </AppLink<ConsumerRoute>>

                <AppLink<ConsumerRoute>
                    class=""
                    selected_class=""
                    route={ConsumerRoute::Favorites}>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"
                        class="size-6 w-10 h-10 text-fuente hover:cursor-pointer">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12Z" />
                    </svg>
                </AppLink<ConsumerRoute>>
            </div>
        </div>
    </header>
    }
}
#[function_component(FuenteFooter)]
pub fn footer() -> Html {
    html! {
    <footer class="bg-fuente p-20 mt-40">
        <div class="container mx-auto flex justify-between items-center lg:gap-4 xl:gap-0">
            <a href="/templates//views/app/home.html">
                <h3 class="text-white font-bold text-5xl">{"Fuente"}</h3>
            </a>

            <div>
                <h3 class="text-xl text-white">{"About Fuente"}</h3>
                <div class="mt-5 space-y-3">
                    <p class="text-white font-light text-lg">{"> How to buy?"}</p>
                    <p class="text-white font-light text-lg">{"> How to sale?"}</p>
                    <p class="text-white font-light text-lg">{"> Why is secure?"}</p>
                </div>
            </div>

            <div>
                <h3 class="text-xl text-white">{"Benefits of Fuente"}</h3>
                <div class="mt-5 space-y-3">
                    <p class="text-white font-light text-lg">{"> Our benefits"}</p>
                    <p class="text-white font-light text-lg">{"> Shipping and collections of orders"}</p>
                    <p class="text-white font-light text-lg">{"> Payment methods"}</p>
                </div>
            </div>

            <div>
                <h3 class="text-xl text-white">{"Politics of Fuente"}</h3>
                <div class="mt-5 space-y-3">
                    <p class="text-white font-light text-lg">{"> Terms and conditions"}</p>
                    <p class="text-white font-light text-lg">{"> General policies"}</p>
                    <p class="text-white font-light text-lg">{"> Privacy Policy"}</p>
                    <p class="text-white font-light text-lg">{"> Return and exchanges"}</p>
                </div>
            </div>
        </div>
    </footer>
    }
}
