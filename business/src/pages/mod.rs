mod history;
mod home;
mod new_user;
mod orders;
mod products;
mod settings;
pub use history::*;
pub use home::*;
use lucide_yew::ArrowLeft;
pub use new_user::*;
pub use products::*;
pub use settings::*;

use fuente::mass::AppLink;
use yew::prelude::*;

use crate::router::CommerceRoute;

#[derive(Clone, PartialEq, Properties)]
pub struct PageHeaderProps {
    pub title: String,
}

#[function_component(PageHeader)]
pub fn page_header(props: &PageHeaderProps) -> Html {
    html! {
        <div class="w-full flex flex-row items-center justify-between pt-8 px-8">
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Home}>
                <ArrowLeft class="w-8 h-8 stroke-black" />
            </AppLink<CommerceRoute>>
            <h3 class="flex-1 text-center text-lg font-semibold">{&props.title}</h3>
            <div class="h-8 w-8"></div>
        </div>
    }
}
