use crate::{contexts::CommerceDataStore, router::ConsumerRoute, contexts::FavoritesStore};
use fuente::mass::{
    AppLink, CommerceProfileCard, HeartIcon, HistoryIcon, HomeIcon, LoadingScreen, LookupIcon,
    MenuBarsIcon, ShoppingCartIcon, UserBadgeIcon,
};
use yew::prelude::*;
use fuente::models::FavoriteStore;
use nostr_minions::key_manager::NostrIdStore;
use crate::contexts::FavoritesAction;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>();
    if commerce_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    let commerce_ctx = commerce_ctx.unwrap();
    if !commerce_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    let businesses = commerce_ctx.commerces();
    let filter_state = use_state(|| CommerceFilter::All);
    html! {
        <div class="h-full w-full flex flex-col">
            <HomeHeader />
            <div class="flex flex-col flex-1 gap-8">
                <h2 class="text-3xl max-w-1/2 font-mplus text-fuente-dark px-4">{"Find your favorite stores!"}</h2>

                <div class="relative w-full max-w-sm mx-auto px-4">
                    <input
                      type={"search"}
                      placeholder={"Search..."}
                      class={"w-full pl-10 pr-4 py-2 text-sm bg-transparent border border-neutral-400
                      rounded-full focus:outline-none focus:border-fuente"}
                    />
                    <div class={"absolute inset-y-0 left-4 flex items-center pl-3 pointer-events-none"}>
                      <LookupIcon class={"h-4 w-4 stroke-neutral-600"} />
                    </div>
                </div>
                <CommerceFilters filter_handle={filter_state} />
                <div class="flex flex-1 flex-row overflow-x-scroll gap-8 pl-8 items-center">
                    {businesses.iter().map(|profile| {
                        let commerce_data = profile.profile().clone();
                        html! {
                            <AppLink<ConsumerRoute>
                                class="w-64"
                                selected_class=""
                                route={ConsumerRoute::Commerce { commerce_id: profile.id().to_string() }}>
                                <div class="relative">
                                <CommerceProfileCard commerce_data={commerce_data.clone()} />
                                <FavoriteButton 
                                    commerce_id={profile.id().to_string()}
                                    commerce_data={commerce_data} 
                                />
                            </div>
                            </AppLink<ConsumerRoute>>
                        }
                    }).collect::<Html>()}
                </div>
            </div>
            <HomeFooter />
        </div>
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

#[derive(Clone, PartialEq)]
pub enum CommerceFilter {
    All,
    Books,
    Tech,
    Clothing,
    Hardware,
    Pharmacy,
    Groceries,
    Music,
    Movies,
}
impl CommerceFilter {
    pub fn all_filters() -> Vec<CommerceFilter> {
        vec![
            CommerceFilter::All,
            CommerceFilter::Books,
            CommerceFilter::Tech,
            CommerceFilter::Clothing,
            CommerceFilter::Hardware,
            CommerceFilter::Pharmacy,
            CommerceFilter::Groceries,
            CommerceFilter::Music,
            CommerceFilter::Movies,
        ]
    }
}

impl ToString for CommerceFilter {
    fn to_string(&self) -> String {
        match self {
            CommerceFilter::All => "All".to_string(),
            CommerceFilter::Groceries => "Groceries".to_string(),
            CommerceFilter::Books => "Books".to_string(),
            CommerceFilter::Tech => "Tech".to_string(),
            CommerceFilter::Clothing => "Clothing".to_string(),
            CommerceFilter::Hardware => "Hardware".to_string(),
            CommerceFilter::Pharmacy => "Pharmacy".to_string(),
            CommerceFilter::Music => "Music".to_string(),
            CommerceFilter::Movies => "Movies".to_string(),

        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct CommerceFiltersProps {
    pub filter_handle: UseStateHandle<CommerceFilter>,
}

#[function_component(CommerceFilters)]
pub fn commerce_filters(props: &CommerceFiltersProps) -> Html {
    let current_filter = (*props.filter_handle).clone();
    let handle = props.filter_handle.clone();
    let selected_class = "text-fuente px-4 py-4 border-b-2 border-fuente";
    let unselected_class = "text-neutral-400 px-4 py-4";
    let filters = CommerceFilter::all_filters();
    html! {
        <div class="flex flex-row pl-4 overflow-x-scroll items-end text-xs font-bold whitespace-nowrap">
        {filters.iter().map(|filter| {
            let class = if *filter == current_filter {
                selected_class
            } else {
                unselected_class
            };
            let handle = handle.clone();
            let filter_clone = filter.clone();
            html! {
                <button
                    class={class}
                    onclick={Callback::from(move |_| handle.set(filter_clone.clone()))}>
                    {filter.to_string()}
                </button>
            }}).collect::<Html>()}
        </div>
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
                route={ConsumerRoute::Profile}>
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
