use super::PageHeader;
use crate::contexts::{FavoritesStore, CommerceDataStore};
use crate::router::ConsumerRoute;
use fuente::mass::{HeartIcon, SimpleFormButton, CommerceProfileCard, AppLink};
use yew::prelude::*;
use nostr_minions::key_manager::NostrIdStore;
use crate::contexts::FavoritesAction;
use fuente::models::FavoriteStore;

#[function_component(FavoritesPage)]
pub fn favorites_page() -> Html {
    let favorites_ctx = use_context::<FavoritesStore>().expect("Favorites context not found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");
    
    // Get all favorites
    let favorites = favorites_ctx.get_favorites();
    
    // Get all businesses
    let commerces = commerce_ctx.commerces();
    
    // Filter businesses that are in favorites
    let favorite_businesses = commerces.iter()
        .filter(|commerce| {
            favorites.iter().any(|f| f.commerce_id == commerce.id())
        })
        .collect::<Vec<_>>();

    gloo::console::log!("Found favorite businesses:", favorite_businesses.len());
    
    if favorite_businesses.is_empty() {
        html! {
            <div class="h-full w-full flex flex-col justify-between items-center">
                <PageHeader title={"Favorites".to_string()} />
                <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                    <HeartIcon class="w-32 h-32 stroke-neutral-200" />
                    <h4 class="text-xl font-semibold mt-4">{"No favorites yet"}</h4>
                    <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48 text-center text-wrap">
                        {"Add your favorite stores to quickly find them here!"}
                    </p>
                </div>
                <AppLink<ConsumerRoute> 
                    route={ConsumerRoute::Home}
                    class="mb-8"
                    selected_class=""
                >
                    <SimpleFormButton>
                        {"Browse Stores"}
                    </SimpleFormButton>
                </AppLink<ConsumerRoute>>
            </div>
        }
    } else {
        html! {
            <div class="h-full w-full flex flex-col">
                <PageHeader title={"Favorites".to_string()} />
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-6 overflow-y-auto">
                    {favorite_businesses.iter().map(|commerce| {
                        let commerce_data = commerce.profile().clone();
                        let commerce_data_clone = commerce_data.clone();
                        html! {
                            <AppLink<ConsumerRoute>
                                route={ConsumerRoute::Commerce { 
                                    commerce_id: commerce.id().to_string() 
                                }}
                                class="w-full"
                                selected_class=""
                            >
                                <div class="relative">
                                    <CommerceProfileCard commerce_data={commerce_data_clone} />
                                    <FavoriteButton 
                                        commerce_id={commerce.id().to_string()}
                                        commerce_data={commerce_data} 
                                    />
                                </div>
                            </AppLink<ConsumerRoute>>
                        }
                    }).collect::<Html>()}
                </div>
            </div>
        }
    }
}

// Add the FavoriteButton component here as well since we're using it
#[derive(Properties, Clone, PartialEq)]
pub struct FavoriteButtonProps {
    pub commerce_id: String,
    pub commerce_data: fuente::models::CommerceProfile,
}

#[function_component(FavoriteButton)]
fn favorite_button(props: &FavoriteButtonProps) -> Html {
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