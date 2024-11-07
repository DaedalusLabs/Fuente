use super::PageHeader;
use fuente::mass::{HeartIcon, SimpleFormButton};
use yew::prelude::*;

#[function_component(FavoritesPage)]
pub fn favorites_page() -> Html {
    html! {
        <div class="h-full w-full flex flex-col justify-between items-center">
            <PageHeader title={"Favorites".to_string()} />
            <div class="flex flex-1 flex-col items-center justify-center text-wrap">
                <HeartIcon class="w-32 h-32 stroke-neutral-200" />
                <h4 class="text-xl font-semibold mt-4">{"No favorites yet"}</h4>
                <p class="text-sm text-neutral-400 font-semibold mt-2 max-w-48  text-center text-wrap">
                    {"Hit the button below to create a new order!"}
                </p>
            </div>
            <SimpleFormButton >
                {"Create an Order"}
            </SimpleFormButton>
        </div>
    }
}
