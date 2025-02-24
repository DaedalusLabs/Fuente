use std::collections::HashMap;

use lucide_yew::{Heart, House, Search, ShieldQuestion, ShoppingCart, UserRound, X};
use nostr_minions::key_manager::NostrIdStore;
use web_sys::wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

use fuente::{
    contexts::LanguageConfigsStore,
    mass::{AppLink, LoginPage},
};

use crate::{
    contexts::{CartStore, CommerceDataStore, ConsumerDataStore, RequireAuth},
    pages::{
        AllCommercesPage, CartPage, CheckoutPage, CommercePage, FavoritesPage, HistoryPage,
        HomePage, LiveOrderCheck, NewAddressPage, NewProfilePage, SettingsPageComponent,
        TrackPackagesPage,
    },
};

#[derive(Clone, Routable, PartialEq, Debug)]
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
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
}
impl TryFrom<&str> for ConsumerRoute {
    type Error = web_sys::wasm_bindgen::JsError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ConsumerRoute::from_path(value, &HashMap::new())
            .ok_or_else(|| web_sys::wasm_bindgen::JsError::new("Invalid route"))
    }
}

#[function_component(ConsumerPages)]
pub fn consumer_pages() -> Html {
    let location = use_location().expect("Location not found");
    let path: ConsumerRoute = match location.path().try_into() {
        Ok(path) => path,
        Err(_) => ConsumerRoute::Home,
    };
    let auth_context = use_context::<NostrIdStore>().expect("LoginStateStore not found");
    let consumer_store = use_context::<ConsumerDataStore>().expect("ConsumerDataStore not found");
    let has_profile = consumer_store.get_profile();
    let has_address = consumer_store.get_default_address();
    let navigator = use_navigator().expect("Navigator not found");
    let logged_in = auth_context.get_identity().cloned();
    html! {
        <div class="flex flex-col h-screen overflow-hidden">
            {if path != ConsumerRoute::Login && path != ConsumerRoute::Register {
                html! {
                    <FuenteHeader />
                }
            } else {
                html! {}
            }}
            <main class="flex-1 overflow-y-auto">
                <Switch<ConsumerRoute> render = { move |switch: ConsumerRoute| {
                    match switch {
                        // Public routes
                        ConsumerRoute::Home => html!{<HomePage />},
                        ConsumerRoute::BrowseStores => html!{<AllCommercesPage />},
                        ConsumerRoute::Commerce { commerce_id } => html!{
                            <CommercePage {commerce_id} />
                        },

                        // Protected routes - for now just render normally
                        ConsumerRoute::Cart => html!{
                            <RequireAuth>
                                <CartPage />
                            </RequireAuth>
                        },
                        ConsumerRoute::Checkout => html!{<CheckoutPage />},
                        ConsumerRoute::History => html!{<HistoryPage />},
                        ConsumerRoute::Settings => html!{
                            <RequireAuth>
                                <SettingsPageComponent />
                            </RequireAuth>
                        },
                        ConsumerRoute::Favorites => html!{<FavoritesPage />},
                        ConsumerRoute::Order { order_id: _ } => html!{<LiveOrderCheck />},
                        ConsumerRoute::TrackPackages => html!{<TrackPackagesPage />},
                        ConsumerRoute::Login => {
                            if logged_in.is_some() {
                                if has_profile.is_none() || has_address.is_none() {
                                    navigator.push(&ConsumerRoute::Register);
                                } else {
                                    navigator.push(&ConsumerRoute::Home);
                                }
                                html!{}
                            } else {
                                html!{
                                    <LoginPage />
                            }}
                        },
                        ConsumerRoute::Register => {
                            match (has_profile.as_ref(), has_address.as_ref()) {
                                (Some(_), Some(_)) => {
                                    navigator.push(&ConsumerRoute::Home);
                                    html!{}
                                },
                                (Some(_), None) => html!{
                                    <NewAddressPage />
                                },
                                (None, Some(_)) => html!{
                                    <NewProfilePage />
                                },
                                (None, None) => html!{
                                    <NewProfilePage />
                                }
                            }
                        },
                    }
                }}
                />
            </main>
            {if path != ConsumerRoute::Login && path != ConsumerRoute::Register {
                html! {
                    <FuenteFooter />
                }
            } else {
                html! {}
            }}
        </div>
    }
}

#[function_component(LoginButton)]
fn login_button() -> Html {
    let navigator = use_navigator().expect("Navigator not found");

    html! {
        <button
            onclick={
                Callback::from(move |_| {
                    navigator.push(&ConsumerRoute::Login);
                })
            }
            class="bg-fuente text-white px-5 py-2 rounded-full text-center font-bold"
        >
            {"Login"}
        </button>
    }
}
#[function_component(FuenteHeader)]
pub fn header() -> Html {
    let cart_ctx = use_context::<CartStore>().expect("CartContext not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let cart_len = cart_ctx.cart().len();
    let is_authenticated = key_ctx.get_identity().is_some();

    html! {
        <header class="container mx-auto pt-5 lg:py-10 flex justify-center lg:justify-between">
           <AppLink<ConsumerRoute>
               class=""
               selected_class=""
               route={ConsumerRoute::Home}>
                   <img src="/public/assets/img/logo.jpg" alt="Logo Fuente" class="w-40 hidden lg:flex"/>
           </AppLink<ConsumerRoute>>

            <div class="flex flex-col lg:flex-row gap-4 flex-1 items-center justify-end w-full">
                <div class="relative flex items-center w-full max-w-sm mx-auto lg:ml-auto lg:mx-0 gap-4">
                    <SearchBar />

                    <AppLink<ConsumerRoute>
                        class=""
                        selected_class=""
                        route={ConsumerRoute::Home}>
                        <House class="bg-fuente h-14 w-14 p-2 rounded-xl text-white lg:hidden" />
                    </AppLink<ConsumerRoute>>
                </div>

                // Auth-required buttons section
                <div class="flex items-center justify-center gap-5">
                    {if is_authenticated {
                        html! {
                            <>
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
                                    <div class="relative">
                                        <ShoppingCart class="h-10 w-10 text-fuente hover:cursor-pointer" />
                                        {if cart_len > 0 {
                                            html! {
                                                <span class="absolute -top-2 -right-2 bg-red-500 text-[12px] text-white rounded-full w-5 h-5 p-1 font-bold flex justify-center items-center">
                                                    {cart_len}
                                                </span>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                </AppLink<ConsumerRoute>>

                                <AppLink<ConsumerRoute>
                                    class=""
                                    selected_class=""
                                    route={ConsumerRoute::Favorites}>
                                    <Heart class="size-6 w-10 h-10 text-fuente hover:cursor-pointer" />
                                </AppLink<ConsumerRoute>>
                            </>
                        }
                    } else {
                        html! {
                            <LoginButton />
                        }
                    }}
                </div>
            </div>
        </header>
    }
}
#[function_component(FuenteFooter)]
pub fn footer() -> Html {
    html! {
        <footer class="bg-fuente-dark p-2 lg:p-4">
            <div class="container mx-auto">
                <div class="flex flex-wrap justify-between items-center gap-4">
                    <div class="w-auto lg:mb-4  items-center">
                        <a href="/" class="inline-block text-center">
                            <h3 class="text-white font-bold text-2xl sm:text-3xl">{"Fuente"}</h3>
                        </a>
                    </div>
                    <ShieldQuestion class="bg-fuente h-8 w-8 p-2 rounded-xl lg:hidden text-white" />

                    <details class="group w-full sm:w-auto hidden lg:block">
                        <summary class="text-lg text-white cursor-pointer list-none">
                            {"About Fuente"}
                            <span class="ml-2 group-open:rotate-180 inline-block transition-transform">{"▼"}</span>
                        </summary>
                        <ul class="mt-2 space-y-1">
                            <li><a href="#" class="text-white text-sm hover:underline">{"How to buy?"}</a></li>
                            <li><a href="#" class="text-white text-sm hover:underline">{"How to sell?"}</a></li>
                            <li><a href="#" class="text-white text-sm hover:underline">{"Why is it secure?"}</a></li>
                        </ul>
                    </details>

                    <details class="group w-full sm:w-auto hidden lg:block">
                        <summary class="text-lg text-white cursor-pointer list-none">
                            {"Benefits"}
                            <span class="ml-2 group-open:rotate-180 inline-block transition-transform">{"▼"}</span>
                        </summary>
                        <ul class="mt-2 space-y-1">
                            <li><a href="#" class="text-white text-sm hover:underline">{"Our benefits"}</a></li>
                            <li><a href="#" class="text-white text-sm hover:underline">{"Shipping and collections"}</a></li>
                            <li><a href="#" class="text-white text-sm hover:underline">{"Payment methods"}</a></li>
                        </ul>
                    </details>

                    <details class="group w-full sm:w-auto hidden lg:block">
                        <summary class="text-lg text-white cursor-pointer list-none">
                            {"Policies"}
                            <span class="ml-2 group-open:rotate-180 inline-block transition-transform">{"▼"}</span>
                        </summary>
                        <ul class="mt-2 space-y-1">
                            <li><a href="#" class="text-white text-sm hover:underline">{"Terms and conditions"}</a></li>
                            <li><a href="#" class="text-white text-sm hover:underline">{"General policies"}</a></li>
                            <li><a href="#" class="text-white text-sm hover:underline">{"Privacy Policy"}</a></li>
                            <li><a href="#" class="text-white text-sm hover:underline">{"Returns and exchanges"}</a></li>
                        </ul>
                    </details>
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
                class="w-full pl-5 pr-10 py-3 border-2 border-fuente rounded-xl text-fuente placeholder:text-fuente"
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
                <div class="absolute z-10 w-full mt-1 bg-white border-2 border-gray-300 rounded-md shadow-lg">
                    <ul class="py-1">
                        {businesses.iter().map(|profile| {
                            let commerce_data = profile.profile().clone();
                            let commerce_id = profile.id().to_string();
                            let is_open_clone = is_open.clone();
                            html! {
                                <li class="px-4 py-2 cursor-pointer hover:bg-gray-100 text-fuente font-semibold"
                                    onclick={Callback::from(move |_| is_open_clone.set(false))}
                                >
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
