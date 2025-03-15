use std::rc::Rc;
use fuente::models::FavoriteStore;
use nostr_minions::{browser_api::IdbStoreManager, key_manager::NostrIdStore};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FavoritesData {
    has_loaded: bool,
    favorites: Vec<FavoriteStore>,
}

impl FavoritesData {
    pub fn is_loaded(&self) -> bool {
        self.has_loaded
    }

    pub fn get_favorites(&self) -> Vec<FavoriteStore> {
        self.favorites.clone()
    }

    pub fn is_favorite(&self, commerce_id: &str) -> bool {
        self.favorites.iter().any(|f| f.commerce_id == commerce_id)
    }
}

pub enum FavoritesAction {
    LoadFavorites(Vec<FavoriteStore>),
    AddFavorite(FavoriteStore),
    RemoveFavorite(String),
    SetLoaded,
}

impl Reducible for FavoritesData {
    type Action = FavoritesAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            FavoritesAction::LoadFavorites(favorites) => Rc::new(FavoritesData {
                has_loaded: self.has_loaded,
                favorites,
            }),
            FavoritesAction::AddFavorite(favorite) => {
                let db_entry = favorite.clone();
                spawn_local(async move {
                    if let Err(e) = db_entry.save_to_store().await {
                        gloo::console::error!("Failed to save favorite:", e);
                    }
                });
                
                let mut favorites = self.favorites.clone();
                favorites.push(favorite);
                
                Rc::new(FavoritesData {
                    has_loaded: self.has_loaded,
                    favorites,
                })
            }
            FavoritesAction::RemoveFavorite(commerce_id) => {
                let mut favorites = self.favorites.clone();
                if let Some(favorite) = favorites.iter().find(|f| f.commerce_id == commerce_id).cloned() {
                    spawn_local(async move {
                        if let Err(e) = favorite.delete_from_store().await {
                            gloo::console::error!("Failed to delete favorite:", e);
                        }
                    });
                }
                
                favorites.retain(|f| f.commerce_id != commerce_id);
                
                Rc::new(FavoritesData {
                    has_loaded: self.has_loaded,
                    favorites,
                })
            }
            FavoritesAction::SetLoaded => Rc::new(FavoritesData {
                has_loaded: true,
                favorites: self.favorites.clone(),
            }),
        }
    }
}

pub type FavoritesStore = UseReducerHandle<FavoritesData>;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct FavoritesChildren {
    pub children: Children,
}

#[function_component(FavoritesProvider)]
pub fn favorites_provider(props: &FavoritesChildren) -> Html {
    let ctx = use_reducer(|| FavoritesData {
        has_loaded: false,
        favorites: vec![],
    });

    let ctx_clone = ctx.clone();
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");

    use_effect_with(key_ctx, move |key_ctx| {
        if let Some(keys) = key_ctx.get_pubkey() {
            let ctx = ctx_clone.clone();
            spawn_local(async move {
                match FavoriteStore::retrieve_all_from_store().await {
                    Ok(favorites) => {
                        // Filter favorites for current user
                        let user_favorites = favorites
                            .into_iter()
                            .filter(|f| f.user_id == keys)
                            .collect();
                        ctx.dispatch(FavoritesAction::LoadFavorites(user_favorites));
                    }
                    Err(e) => gloo::console::error!("Failed to load favorites:", e),
                }
                ctx.dispatch(FavoritesAction::SetLoaded);
            });
        }
        || {}
    });

    html! {
        <ContextProvider<FavoritesStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<FavoritesStore>>
    }
}
