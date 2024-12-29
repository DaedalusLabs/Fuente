use crate::contexts::FavoritesAction;
use crate::{contexts::CommerceDataStore, contexts::FavoritesStore, router::ConsumerRoute};
use fuente::mass::templates::{FuenteBitcoinBanner, FuenteHotCategories, FuenteSalesPitch};
use fuente::models::FavoriteStore;
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{
        AppLink, CommerceProfileCard, HeartIcon, HistoryIcon, HomeIcon, LookupIcon, MenuBarsIcon,
        ShoppingCartIcon, UserBadgeIcon,
    },
};
use nostr_minions::key_manager::NostrIdStore;
use yew::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let translations = language_ctx.translations();
    let businesses = commerce_ctx.commerces();
    html! {
        <>
            <CommerceFilters />
            <FuenteHotCategories />
            <FuenteStoresBanner/>
            <FuenteBitcoinBanner />
            <FuenteSalesPitch />

        </>
    }
}
#[function_component(FuenteStoresBanner)]
pub fn stores_banner() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    let businesses = commerce_ctx.commerces();
    html! {
    <section class="container mx-auto bg-sky-200 rounded-2xl mt-10 py-10">
        <h2 class="text-fuente text-5xl font-semibold px-10 tracking-tighter">{"Our top stores"}</h2>

        <div class="flex justify-between items-center mt-10 px-10">
            <svg viewBox="0 0 64 64"  xmlns="http://www.w3.org/2000/svg" enable-background="new 0 0 64 64" class="w-16 h-16">
                <path d="M4-272.1c-13.2 0-23.9-10.7-23.9-23.9S-9.2-319.9 4-319.9s23.9 10.7 23.9 23.9S17.2-272.1 4-272.1zm0-45.2c-11.7 0-21.3 9.6-21.3 21.3s9.6 21.3 21.3 21.3 21.3-9.6 21.3-21.3-9.6-21.3-21.3-21.3z" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path><path d="M4.5-282.3-9.2-296l13.7-13.7 1.8 1.9L-5.4-296l11.7 11.8-1.8 1.9" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path><path d="M-7.3-297.4h24v2.8h-24z" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path>
            </svg>

            <div class="overflow-x-auto whitespace-nowrap">
                <div class="grid grid-flow-col auto-cols-max gap-10">
                    {businesses.iter().map(|profile| {
                        let commerce_data = profile.profile().clone();
                        html! {
                            <AppLink<ConsumerRoute>
                                class="border-2 border-fuente rounded-3xl block object-contain w-40 bg-white"
                                selected_class=""
                                route={ConsumerRoute::Commerce { commerce_id: profile.id().to_string() }}>
                                <CommerceProfileCard commerce_data={commerce_data.clone()} />
                            </AppLink<ConsumerRoute>>
                        }
                    }).collect::<Html>()}
                </div>
            </div>

            <svg viewBox="0 0 64 64"  xmlns="http://www.w3.org/2000/svg" enable-background="new 0 0 64 64" class="w-16 h-16">
                <path d="M4-272.1c-13.2 0-23.9-10.7-23.9-23.9S-9.2-319.9 4-319.9s23.9 10.7 23.9 23.9S17.2-272.1 4-272.1zm0-45.2c-11.7 0-21.3 9.6-21.3 21.3s9.6 21.3 21.3 21.3 21.3-9.6 21.3-21.3-9.6-21.3-21.3-21.3z" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path><path d="m3.5-282.3-1.8-1.9L13.4-296 1.7-307.8l1.8-1.9L17.2-296 3.5-282.3" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path><path d="M15.3-294.6h-24v-2.8h24z" transform="translate(28 328)" fill="#4167e8" class="fill-134563"></path>
            </svg>
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
                "top-4",
                "right-4",
                "p-2",
                "rounded-full",
                "hover:bg-gray-100",
                "transition-colors",
                if is_favorite { "text-red-500" } else { "text-gray-400" }
            )}
        >
            <HeartIcon class="w-6 h-6" />
        </button>
    }
}

#[function_component(CommerceFilters)]
pub fn commerce_filters() -> Html {
    html! {
        <nav class="hidden lg:flex max-w-6xl mx-auto">
            <div class="flex justify-between">
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
#[function_component(HomeHeader)]
pub fn home_header() -> Html {
    html! {
        <div class="w-full flex flex-row justify-between p-4 ">
            <AppLink<ConsumerRoute>
                class="" selected_class=""
                route={ConsumerRoute::Settings}>
                <MenuBarsIcon class="w-8 h-8 stroke-neutral-900" />
            </AppLink<ConsumerRoute>>
            <AppLink<ConsumerRoute>
                class="" selected_class=""
                route={ConsumerRoute::Cart}>
                <ShoppingCartIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<ConsumerRoute>>
        </div>
    }
}
#[function_component(HomeFooter)]
pub fn home_footer() -> Html {
    html! {
        <div class="w-full p-4 flex flex-row justify-between">
            <AppLink<ConsumerRoute>
                class="" selected_class=""
                route={ConsumerRoute::Home}>
                <HomeIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<ConsumerRoute>>
            <AppLink<ConsumerRoute>
                class="" selected_class=""
                route={ConsumerRoute::Favorites}>
                <HeartIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<ConsumerRoute>>
            <AppLink<ConsumerRoute>
                class="" selected_class=""
                route={ConsumerRoute::Settings}>
                <UserBadgeIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<ConsumerRoute>>
            <AppLink<ConsumerRoute>
                class="" selected_class=""
                route={ConsumerRoute::History}>
                <HistoryIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<ConsumerRoute>>
        </div>
    }
}
