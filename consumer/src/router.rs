use lucide_yew::{Heart, House, Menu, Search, ShoppingCart, UserRound, X};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

use fuente::{contexts::LanguageConfigsStore, mass::AppLink};

use crate::{
    contexts::{CartStore, CommerceDataStore},
    pages::{
        AllCommercesPage, CartPage, CheckoutPage, CommercePage, FavoritesPage, HistoryPage,
        HomePage, LiveOrderCheck, SettingsPageComponent, TrackPackagesPage,
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
    #[at("/checkout")]
    Checkout,
    #[at("/browse-stores")]
    BrowseStores,
    #[at("/commerce/:commerce_id")]
    Commerce { commerce_id: String },
    #[at("/order/:order_id")]
    Order { order_id: String },
    #[at("/track-packages")] // Add this new route
    TrackPackages,
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
                        ConsumerRoute::Checkout => html!{<CheckoutPage />},
                        ConsumerRoute::BrowseStores => html!{<AllCommercesPage />},
                        ConsumerRoute::Commerce { commerce_id } => html!{
                            <CommercePage {commerce_id} />
                        },
                        ConsumerRoute::Order { order_id: _ } => html!{
                            <LiveOrderCheck />
                        },
                        ConsumerRoute::TrackPackages => html!{<TrackPackagesPage />},
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
    let cart_ctx = use_context::<CartStore>().expect("CartContext not found");
    let cart_len = cart_ctx.cart().len();
    html! {
    <header class="container mx-auto py-10 flex justify-center lg:justify-between">
       <AppLink<ConsumerRoute>
           class="hidden lg:flex"
           selected_class=""
           route={ConsumerRoute::Home}>
               <img src="/public/assets/img/logo.jpg" alt="Logo Fuente" class="w-40 hidden lg:flex"/>
       </AppLink<ConsumerRoute>>

        <div class="flex flex-col lg:flex-row gap-4 flex-1 items-center justify-end w-full">
            <div class="relative flex items-center w-full max-w-sm mx-auto lg:ml-auto lg:mx-0 gap-4">
                <SearchBar />

                <AppLink<ConsumerRoute>
                    class="lg:hidden"
                    selected_class=""
                    route={ConsumerRoute::Home}>
                    <House class="bg-fuente h-14 w-14 p-2 rounded-xl text-white lg:hidden" />
                </AppLink<ConsumerRoute>>
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
                    {match cart_len {
                        0 => html! {<ShoppingCart class="h-10 w-10 text-fuente hover:cursor-pointer" />},
                        _ => html! {
                            <div class="relative">
                                <ShoppingCart class="h-10 w-10 text-fuente hover:cursor-pointer" />
                                <span class="absolute -top-2 -right-2 bg-red-500 text-[12px] text-white rounded-full w-5 h-5 p-1 font-bold flex justify-center items-center">
                                    {cart_len}
                                </span>
                            </div>
                        }
                    }}
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
#[function_component(SearchBar)]
pub fn commerce_filters() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    let businesses = use_state(|| vec![]);
    let is_open = use_state(|| false);
    let wrapper_ref = use_node_ref();
    let search_term = use_state(|| String::new());

    {
        let wrapper_clone = wrapper_ref.clone();
        let is_open_clone = is_open.clone();
        use_effect_with(wrapper_ref.clone(), |div_ref| {
            let div = div_ref
                .cast::<HtmlElement>()
                .expect("div_ref not attached to div element");

            let handler = Closure::wrap(Box::new(move |event: MouseEvent| {
                if let Some(wrapper) = wrapper_clone.get() {
                    if !wrapper.contains(Some(&event.target_unchecked_into())) {
                        is_open_clone.set(false);
                    }
                }
            }) as Box<dyn FnMut(_)>);
            let handler_ref = handler.as_ref().unchecked_ref();
            web_sys::window()
                .expect("Window not found")
                .add_event_listener_with_callback("mousedown", handler_ref)
                .expect("Failed to add event listener");

            move || {
                div.remove_event_listener_with_callback("click", handler.as_ref().unchecked_ref())
                    .unwrap();
            }
        });
    }

    let businesses_clone = businesses.clone();
    use_effect_with(search_term.clone(), move |search_term| {
        let search_term = search_term.clone();
        businesses_clone.set(
            commerce_ctx
                .commerces()
                .iter()
                .filter(|profile| {
                    profile
                        .profile()
                        .name
                        .to_lowercase()
                        .contains(&*search_term.to_lowercase())
                })
                .cloned()
                .collect(),
        );
        || {}
    });

    let set_search_term = {
        let search_term = search_term.clone();
        let is_open = is_open.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            search_term.set(value);
            is_open.set(true);
        })
    };
    let clear_search = {
        let search_term = search_term.clone();
        let is_open = is_open.clone();
        Callback::from(move |_| {
            search_term.set(String::new());
            is_open.set(false);
        })
    };
    html! {
    <div ref={wrapper_ref} class="relative w-full max-w-2xl mx-auto">
        <div class="relative">
            <input
                type="text"
                value={(*search_term).clone()}
                oninput={set_search_term}
                placeholder={translations["nav_search"].clone()}
                class="w-full pl-10 pr-10 py-3 border-2 border-fuente rounded-xl text-fuente placeholder:text-fuente"
            />
            {if search_term.is_empty() {
                html! {
                    <div class="absolute top-3 right-4 text-fuente pointer-events-none" >
                        <Search class="w-6 h-6 text-fuente" />
                    </div>
                }
            } else { html! {
                    <button
                        type="button"
                        onclick={clear_search}
                        class="absolute inset-y-0 right-0 flex items-center pr-3"
                        >
                        <X class="w-5 h-5 text-gray-400 hover:text-gray-600" />
                    </button>
            } }}
        </div>
        {if *is_open && businesses.len() > 0 {
            html! {
                <div class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg">
                    <ul class="py-1">
                        {businesses.iter().map(|profile| {
                            let commerce_data = profile.profile().clone();
                            let commerce_id = profile.id().to_string();
                            html! {
                                <li class="px-4 py-2 cursor-pointer hover:bg-gray-100 text-fuente font-semibold">
                                    <AppLink<ConsumerRoute>
                                        class="block"
                                        selected_class=""
                                        route={ConsumerRoute::Commerce { commerce_id: commerce_id.clone() }}>
                                        {commerce_data.name}
                                    </AppLink<ConsumerRoute>>
                                </li>
                            }
                        }).collect::<Html>()}
                    </ul>
                </div>
            }
        } else { html! {} }}
    </div>
    }
}
