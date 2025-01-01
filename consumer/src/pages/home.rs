use crate::contexts::{FavoritesAction, RatingsStore};
use crate::{contexts::CommerceDataStore, contexts::FavoritesStore, router::ConsumerRoute};
use fuente::mass::templates::{FuenteBenefits, FuenteBitcoinBanner, FuenteHotCategories, FuenteSalesPitch};
use fuente::mass::{AppLink, CommerceProfileCard};
use fuente::models::FavoriteStore;
use lucide_yew::{ArrowLeft, ArrowRight, Heart};
use nostr_minions::key_manager::NostrIdStore;
use yew::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div class="space-y-4">
            <CommerceFilters />
            <FuenteStoresBanner/>
            <FuenteHotCategories />
            <FuenteBitcoinBanner />
            <FuenteSalesPitch />
            <FuenteBenefits />
        </div>
    }
}
#[function_component(FuenteStoresBanner)]
pub fn stores_banner() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    let ratings_ctx = use_context::<RatingsStore>().expect("RatingsStore not found");
    let businesses = commerce_ctx.commerces();

    html! {
        <section class="container mx-auto bg-sky-200 rounded-2xl mt-10 py-10">
            <h2 class="text-fuente text-5xl font-semibold px-10 tracking-tighter">{"Our top stores"}</h2>

            <div class="flex justify-between items-center mt-10 px-10">
                <ArrowLeft class="w-16 h-16 text-fuente rounded-full border-4 border-fuente" />
                <div class="overflow-x-auto whitespace-nowrap">
                    <div class="grid grid-flow-col auto-cols-max gap-10">
                        {businesses.iter().map(|profile| {
                            let commerce_data = profile.profile().clone();
                            let commerce_id = profile.id().to_string();
                            let rating = ratings_ctx.get_business_rating(&commerce_id);

                            // debugging
                            gloo::console::log!("Rating for business:", commerce_id.clone(), format!("{:?}", rating));
                            
                            html! {
                                <AppLink<ConsumerRoute>
                                    class="border-2 border-fuente rounded-3xl block object-contain w-40 bg-white h-40 overflow-clip"
                                    selected_class=""
                                    route={ConsumerRoute::Commerce { commerce_id: commerce_id.clone() }}>
                                    <div class="relative">
                                        <CommerceProfileCard {commerce_data} {rating} />
                                    </div>
                                </AppLink<ConsumerRoute>>
                            }
                        }).collect::<Html>()}
                    </div>
                </div>
                <ArrowRight class="w-16 h-16 text-fuente rounded-full border-4 border-fuente" />
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
            <Heart class="w-6 h-6" />
        </button>
    }
}

#[function_component(CommerceFilters)]
pub fn commerce_filters() -> Html {
    html! {
        <nav class="hidden lg:flex lg:max-w-4xl xl:max-w-6xl mx-auto">
            <div class="flex justify-evenly w-full">
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
