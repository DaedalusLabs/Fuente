use lucide_yew::{Heart, Menu, Search, ShoppingCart, UserRound};
use yew::prelude::*;
use yew_router::prelude::*;

use fuente::mass::AppLink;

use crate::pages::{
    CartPage, CommercePage, FavoritesPage, HistoryPage, HomePage, LiveOrderCheck, SettingsPageComponent
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
               <img src="/templates/img/Logo Fuente.jpeg" alt="Logo Fuente" class="w-40 hidden lg:flex"/>
       </AppLink<ConsumerRoute>>

        <div class="flex flex-col lg:flex-row gap-4 flex-1 items-center justify-end w-full">
            <div class="relative flex items-center w-full max-w-sm mx-auto lg:ml-auto lg:mx-0 gap-4">
                <div class="relative flex items-center w-full">
                    <input
                        type="text"
                        class="w-full pl-10 pr-10 py-3 border-2 border-fuente rounded-xl"
                    />
                    <Search class="absolute right-4 h-6 w-6 text-fuente pointer-events-none" />
                </div>

                <button class="lg:hidden">
                    <Menu class="bg-fuente h-14 w-14 p-2 rounded-xl text-white" />
                </button>
            </div>
            <div class="flex gap-5">
                <AppLink<ConsumerRoute>
                    class=""
                    selected_class=""
                    route={ConsumerRoute::Settings}>
                    <UserRound class="size-6 w-10 h-10 text-fuente hover:cursor-pointer" />
                </AppLink<ConsumerRoute>>

                <AppLink<ConsumerRoute>
                    class=""
                    selected_class=""
                    route={ConsumerRoute::Cart}>
                    <ShoppingCart class="h-10 w-10 text-fuente hover:cursor-pointer" />
                </AppLink<ConsumerRoute>>

                <AppLink<ConsumerRoute>
                    class=""
                    selected_class=""
                    route={ConsumerRoute::Favorites}>
                    <Heart class="size-6 w-10 h-10 text-fuente hover:cursor-pointer" />
                </AppLink<ConsumerRoute>>
            </div>
        </div>
    </header>
    }
}
#[function_component(FuenteFooter)]
pub fn footer() -> Html {
    html! {
    <footer class="bg-fuente p-10 lg:p-20 mt-40">
        <div class="container mx-auto flex justify-between items-center gap-4 xl:gap-0">
            <a href="#">
                <h3 class="text-white font-bold text-4xl lg:text-5xl">{"Fuente"}</h3>
            </a>

            <Menu class="bg-fuente h-14 w-14 p-2 rounded-xl lg:hidden text-white" />

            <div class="hidden lg:block">
                <h3 class="text-xl text-white">{"About Fuente"}</h3>
                <div class="mt-5 space-y-3">
                    <p class="text-white font-light text-lg">{"> How to buy?"}</p>
                    <p class="text-white font-light text-lg">{"> How to sale?"}</p>
                    <p class="text-white font-light text-lg">{"> Why is secure?"}</p>
                </div>
            </div>

            <div class="hidden lg:block">
                <h3 class="text-xl text-white">{"Benefits of Fuente"}</h3>
                <div class="mt-5 space-y-3">
                    <p class="text-white font-light text-lg">{"> Our benefits"}</p>
                    <p class="text-white font-light text-lg">{"> Shipping and collections of orders"}</p>
                    <p class="text-white font-light text-lg">{"> Payment methods"}</p>
                </div>
            </div>

            <div class="hidden lg:block">
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
