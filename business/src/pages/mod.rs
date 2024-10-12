pub mod history;
pub mod home;
pub mod new_user;
pub mod products;
pub mod profile;
pub mod orders;

use yew::prelude::*;
use fuente::mass::atoms::{forms::AppLink, svgs::BackArrowIcon};

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
                <BackArrowIcon class="w-8 h-8 stroke-black" />
            </AppLink<CommerceRoute>>
            <h3 class="flex-1 text-center text-lg font-semibold">{&props.title}</h3>
            <div class="h-8 w-8"></div>
        </div>
    }
}
