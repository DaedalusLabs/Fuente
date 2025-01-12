use fuente::mass::{templates::FuenteSidebarTemplate, AppLink};
use lucide_yew::{BellPlus, History, UserRound};
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
        <div class="flex flex-col lg:flex-row h-screen overflow-hidden">
            <NavigationBar/>
            <Switch<DriverRoute> render = { move |switch: DriverRoute| {
                    match switch {
                        DriverRoute::Home => html!{<HomePage />},
                        DriverRoute::History => html!{<HistoryPage />},
                        DriverRoute::Settings => html!{<SettingsPageComponent />},
                    }
                }}
            />
        </div>
    }
}
#[function_component(NavigationBar)]
pub fn home_footer() -> Html {
    html! {
        <FuenteSidebarTemplate>
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::Home}>
                <BellPlus class="w-8 h-8 text-fuente" />
            </AppLink<DriverRoute>>
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::Settings}>
                <UserRound class="w-8 h-8 text-fuente" />
            </AppLink<DriverRoute>>
            <AppLink<DriverRoute>
                class="" selected_class=""
                route={DriverRoute::History}>
                <History class="w-8 h-8 text-fuente" />
            </AppLink<DriverRoute>>
        </FuenteSidebarTemplate>
    }
}
