use crate::{contexts::commerce_data::CommerceDataStore, router::DriverRoute};
use fuente::mass::atoms::{
    forms::AppLink,
    layouts::LoadingScreen,
    svgs::{HistoryIcon, HomeIcon, MenuBarsIcon, UserBadgeIcon},
};
use yew::prelude::*;

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
    html! {
            <div class="flex flex-col flex-1 gap-8">
                <h2 class="text-3xl max-w-1/2 font-mplus text-fuente-dark px-4">{"Waiting for a delivery!"}</h2>
            </div>
    }
}
