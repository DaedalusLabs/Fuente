use crate::contexts::FavoritesAction;
use crate::contexts::{CommerceDataStore, FavoritesStore};
use crate::router::ConsumerRoute;
use fuente::mass::templates::FavoritesPageTemplate;
use fuente::mass::{AppLink, CommerceProfileProps};
use fuente::models::FavoriteStore;
use lucide_yew::{Heart, Star};
use nostr_minions::key_manager::NostrIdStore;
use yew::prelude::*;

#[function_component(FavoritesPage)]
pub fn favorites_page() -> Html {
    let favorites_ctx = use_context::<FavoritesStore>().expect("Favorites context not found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Commerce context not found");

    // Get all favorites
    let favorites = favorites_ctx.get_favorites();

    // Get all businesses
    let commerces = commerce_ctx.commerces();

    // Filter businesses that are in favorites
    let favorite_businesses = commerces
        .iter()
        .filter(|commerce| favorites.iter().any(|f| f.commerce_id == commerce.id()))
        .collect::<Vec<_>>();

    html! {
        <FavoritesPageTemplate >
            <div class="w-full grid grid-col-flow gap-4 sm:gap-6 lg:gap-8 justify-center lg:justify-start">
                {favorite_businesses.iter().map(|commerce| {
                    html! {
                        <AppLink<ConsumerRoute>
                            route={ConsumerRoute::Commerce {
                                commerce_id: commerce.id().to_string()
                            }}
                            class="w-full"
                            selected_class=""
                        >
                        <div class="relative border-2 border-fuente rounded-2xl p-4 md:p-10 lg:p-16 shadow-md w-fit">
                            <FavoriteCommerceTemplate commerce_data={commerce.profile().clone()} />
                            <FavoriteButton commerce_id={commerce.id().to_string()} />
                        </div>
                        </AppLink<ConsumerRoute>>
                    }
                }).collect::<Html>()}
            </div>
        </FavoritesPageTemplate>
    }
}
#[function_component(FavoriteCommerceTemplate)]
pub fn favorite_commerce_template(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps {
        commerce_data,
        rating,
    } = props;
    html! {
       <div class="flex flex-col sm:flex-row items-center gap-4 sm:gap-6 lg:gap-8 w-fit">
           <img src={commerce_data.logo_url.clone()} alt="Company Image" class="w-24 h-24 sm:w-28 sm:h-28 lg:w-32 lg:h-32 object-cover rounded-2xl border-2 border-fuente"/>
           <div class="space-y-2 flex flex-col items-center sm:items-start text-center sm:text-left">
               <h3 class="text-gray-700 text-lg sm:text-xl lg:text-2xl font-bold tracking-wide uppercase">{&commerce_data.name}</h3>
               <p class="text-gray-600 font-normal text-sm sm:text-base lg:text-lg max-w-md">{&commerce_data.description}</p>
               {if let Some(rating) = rating {
                   html! {
                   <div class="flex items-center gap-2 mt-2">
                       <Star class="w-5 h-5 sm:w-6 sm:h-6 text-yellow-400" />
                       <p class="text-gray-700 font-medium text-sm sm:text-base lg:text-lg">{&rating.trust_score}</p>
                   </div>
                   }
               } else {
                   html! {}
               }}
           </div>
       </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct FavoriteButtonProps {
    pub commerce_id: String,
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
            <Heart class="w-6 h-6" />
        </button>
    }
}
