use fuente::mass::atoms::{
    forms::AppLink,
    svgs::{HistoryIcon, HomeIcon, MenuBarsIcon, UserBadgeIcon},
};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::home::HomePage;

#[derive(Clone, Routable, PartialEq)]
pub enum DriverRoute {
    #[at("/")]
    Home,
    #[at("/history")]
    History,
    #[at("/profile")]
    Profile,
    #[at("/settings")]
    Settings,
}

#[function_component(DriverPages)]
pub fn consumer_pages() -> Html {
    html! {
        <div class="h-full w-full flex flex-col justify-between">
            <HomeHeader />
            <Switch<DriverRoute> render = { move |switch: DriverRoute| {
                    match switch {
                        DriverRoute::Home => html!{<HomePage />},
                        DriverRoute::History => html!{<></>},
                        DriverRoute::Profile => html!{<></>},
                        DriverRoute::Settings => html!{<></>},
                    }
                }}
            />
            <HomeFooter />
        </div>
    }
}
#[function_component(HomeHeader)]
pub fn home_header() -> Html {
    html! {
        <div class="w-full flex flex-row justify-between p-4 ">
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::Settings}>
                <MenuBarsIcon class="w-8 h-8 stroke-neutral-900" />
            </AppLink<DriverRoute>>
        </div>
    }
}
#[function_component(HomeFooter)]
pub fn home_footer() -> Html {
    html! {
        <div class="w-full p-4 flex flex-row justify-around">
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::Profile}>
                <UserBadgeIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<DriverRoute>>
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::Home}>
                <HomeIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<DriverRoute>>
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::History}>
                <HistoryIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<DriverRoute>>
        </div>
    }
}
