use crate::contexts::{FavoritesAction, RatingsStore};
use crate::{contexts::CommerceDataStore, contexts::FavoritesStore, router::ConsumerRoute};
use fuente::contexts::LanguageConfigsStore;
use fuente::mass::templates::{FuenteBenefits, FuenteBitcoinBanner, FuenteSalesPitch};
use fuente::mass::{AppLink, CommerceProfileCard};
use fuente::models::FavoriteStore;
use lucide_yew::{ArrowRight, Heart};
use nostr_minions::key_manager::NostrIdStore;
use yew::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let translations = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = translations.translations();
    html! {
        <main class="flex flex-col flex-1 overflow-hidden w-full mx-auto">
            <CommerceFilters />
            <div class="grid grid-cols-1 gap-4 overflow-y-auto">
                <div class="grid lg:grid-cols-2 gap-4 container mx-auto">
                    <FuenteStoresBanner/>
                    <div class="bg-fuente rounded-2xl p-5 flex flex-col h-fit w-full">
                        <div class="flex flex-col gap-2 lg:mb-4">
                            <h2 class="text-white text-4xl font-semibold tracking-tighter">{&translations["home_stores"]}</h2>
                            <AppLink<ConsumerRoute>
                                class=""
                                selected_class=""
                                route={ConsumerRoute::BrowseStores}>
                                <ArrowRight class="w-12 h-12 text-white rounded-full border-4 border-white" />
                            </AppLink<ConsumerRoute>>
                        </div>

                        <img src="/public/assets/img/store.png" alt="Store Image" class="object-contain w-72 mx-auto " />
                    </div>
                </div>
                <FuenteBitcoinBanner />
                <FuenteSalesPitch />
                <FuenteBenefits />
            </div>
        </main>
    }
}
#[function_component(FuenteStoresBanner)]
pub fn stores_banner() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    let ratings_ctx = use_context::<RatingsStore>().expect("RatingsStore not found");
    let languages = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = languages.translations();
    let businesses = commerce_ctx.commerces();

    html! {
        <section class="bg-sky-200 rounded-2xl py-10 mt-5 lg:mt-0">
            <div class="flex justify-between items-center w-full mx-auto">
                <h2 class="text-fuente text-5xl font-semibold tracking-tighter px-5">{&translations["home_top_stores"]}</h2>
            </div>

            <div class="flex justify-center lg:justify-between items-center mt-10 px-6">
                // <button onclick={scroll_left}>
                //     <ArrowLeft
                //         class="w-8 h-8 sm:w-10 sm:h-10 md:h-12 md:w-12 lg:h-16 lg:w-16 text-fuente rounded-full border-4 border-fuente m-2" />
                // </button>
                <div class="overflow-x-auto whitespace-nowrap no-scrollbar">
                    <div id="commerce_carousel" class="grid grid-flow-col auto-cols-max gap-10">
                        {businesses.iter().map(|profile| {
                            gloo::console::log!("Profile");
                            let commerce_data = profile.profile().clone();
                            let commerce_id = profile.id().to_string();
                            let rating = ratings_ctx.get_business_rating(&commerce_id);

                            html! {
                                <AppLink<ConsumerRoute>
                                    class="border-2 border-fuente rounded-3xl block object-contain w-40 bg-white h-40 overflow-clip"
                                    selected_class=""
                                    route={ConsumerRoute::Commerce { commerce_id: commerce_id.clone() }}>
                                    <div class="relative">
                                        <CommerceProfileCard commerce_data={commerce_data.clone()} {rating} />
                                        <FavoriteButton commerce_id={commerce_id} commerce_data={commerce_data} />
                                    </div>
                                </AppLink<ConsumerRoute>>
                            }
                        }).collect::<Html>()}
                    </div>
                </div>
                // <button  onclick={scroll_right}>
                // <ArrowRight
                //     class="w-8 h-8 sm:w-10 sm:h-10 md:h-12 md:w-12 lg:h-16 lg:w-16 text-fuente rounded-full border-4 border-fuente m-2" />
                // </button>
            </div>
        </section>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct HomeFavoriteButtonProps {
    pub commerce_id: String,
    pub commerce_data: fuente::models::CommerceProfile,
}

#[function_component(FavoriteButton)]
fn favorite_button(props: &HomeFavoriteButtonProps) -> Html {
    let favorites_ctx = use_context::<FavoritesStore>().expect("Favorites context not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    if key_ctx.get_nostr_key().is_none() {
        return html! {};
    }

    let is_favorite = favorites_ctx.is_favorite(&props.commerce_id);

    let onclick = {
        let commerce_id = props.commerce_id.clone();
        let favorites = favorites_ctx.clone();
        let user_id = key_ctx.get_nostr_key().unwrap().public_key();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if favorites.is_favorite(&commerce_id) {
                favorites.dispatch(FavoritesAction::RemoveFavorite(commerce_id.clone()));
            } else {
                let favorite = FavoriteStore::new(commerce_id.clone(), user_id.clone());
                favorites.dispatch(FavoritesAction::AddFavorite(favorite));
            }
        })
    };

    html! {
        <button
            {onclick}
            class={classes!(
                "absolute",
                "z-[500]",
                "top-0",
                "right-0",
                "p-2",
                "rounded-full",
                "hover:bg-gray-100",
                "transition-colors",
                if is_favorite { "text-red-500" } else { "text-gray-400" }
            )}
        >
            <Heart class="w-6 h-6" />
        </button>
    }
}

#[function_component(CommerceFilters)]
pub fn commerce_filters() -> Html {
    html! {
        <nav class="hidden lg:flex w-full mx-auto items-center justify-center mb-5">
            <div class="flex justify-evenly  lg:max-w-4xl xl:max-w-6xl w-full">
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Books"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Tech"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Clothing"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Hardware"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Pharmacy"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Groceries"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Music"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Movies"}</a>
                <a href="#" class="text-fuente-dark font-semibold text-xl">{"Furniture"}</a>
            </div>
        </nav>
    }
}
