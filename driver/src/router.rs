use fuente::mass::AppLink;
use lucide_yew::{History, House, Menu, UserRound};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{HistoryPage, HomePage, SettingsPageComponent};

#[derive(Clone, Routable, PartialEq)]
pub enum DriverRoute {
    #[at("/")]
    Home,
    #[at("/history")]
    History,
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
                        DriverRoute::History => html!{<HistoryPage />},
                        DriverRoute::Settings => html!{<SettingsPageComponent />},
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
            <SettingsToggleLink />
        </div>
    }
}
#[function_component(HomeFooter)]
pub fn home_footer() -> Html {
    html! {
        <div class="w-full p-4 flex flex-row justify-around">
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::Settings}>
                <UserRound class="w-8 h-8 stroke-neutral-400" />
            </AppLink<DriverRoute>>
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::Home}>
                <House class="w-8 h-8 stroke-neutral-400" />
            </AppLink<DriverRoute>>
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::History}>
                <History class="w-8 h-8 stroke-neutral-400" />
            </AppLink<DriverRoute>>
        </div>
    }
}
#[function_component(SettingsToggleLink)]
pub fn settings_toggle() -> Html {
    let navigator = use_navigator().unwrap();
    let current_route = use_route::<DriverRoute>().unwrap();

    let onclick = Callback::from(move |_| {
        if current_route == DriverRoute::Settings {
            navigator.push(&DriverRoute::Home)
        } else {
            navigator.push(&DriverRoute::Settings)
        }
    });

    html! {
        <button {onclick}>
            <Menu class="w-8 h-8 stroke-neutral-400" />
        </button>
    }
}
