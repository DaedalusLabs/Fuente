pub mod cart;
pub mod commerce;
pub mod favorites;
pub mod history;
pub mod home;
pub mod new_user;
pub mod profile;

use yew::prelude::*;
use fuente::mass::atoms::{forms::AppLink, svgs::BackArrowIcon};

use crate::router::ConsumerRoute;

#[derive(Clone, PartialEq, Properties)]
pub struct PageHeaderProps {
    pub title: String,
}

#[function_component(PageHeader)]
pub fn page_header(props: &PageHeaderProps) -> Html {
    html! {
        <div class="w-full flex flex-row items-center justify-between p-4">
            <AppLink<ConsumerRoute>
                class="" selected_class=""
                route={ConsumerRoute::Home}>
                <BackArrowIcon class="w-8 h-8 stroke-black" />
            </AppLink<ConsumerRoute>>
            <h3 class="flex-1 text-center text-2xl font-mplus text-fuente-dark">{&props.title}</h3>
            <div class="h-8 w-8"></div>
        </div>
    }
}
