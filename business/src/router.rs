use fuente::mass::{AppLink, templates::FuenteSidebarTemplate};
use lucide_yew::{BellPlus, History, Settings, Store};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{HistoryPage, HomePage, ProductsPage, SettingsPageComponent};

#[derive(Clone, Routable, PartialEq)]
pub enum CommerceRoute {
    #[at("/")]
    Home,
    #[at("/history")]
    History,
    #[at("/settings")]
    Settings,
    #[at("/products")]
    Products,
    #[at("/orders")]
    Orders,
}

#[function_component(CommercePages)]
pub fn consumer_pages() -> Html {
    html! {
        <div class="flex flex-col lg:flex-row h-screen overflow-hidden">
            <HomeSidebar />
            <Switch<CommerceRoute> render = { move |switch: CommerceRoute| {
                    match switch {
                        CommerceRoute::Home => html!{<HomePage />},
                        CommerceRoute::History => html!{<HistoryPage />},
                        CommerceRoute::Settings => html!{<SettingsPageComponent />},
                        CommerceRoute::Products => html!{<ProductsPage />},
                        CommerceRoute::Orders => html!{<></>},
                    }
                }}
            />
        </div>
    }
}
#[function_component(HomeSidebar)]
pub fn home_footer() -> Html {
    html! {
        <FuenteSidebarTemplate >
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Home}>
                <BellPlus class="w-8 h-8 stroke-fuente" />
            </AppLink<CommerceRoute>>
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Products}>
                <Store class="w-8 h-8 stroke-fuente" />
            </AppLink<CommerceRoute>>
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::History}>
                <History class="w-8 h-8 stroke-fuente" />
            </AppLink<CommerceRoute>>
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Settings}>
                <Settings class="w-8 h-8 stroke-fuente" />
            </AppLink<CommerceRoute>>
        </FuenteSidebarTemplate>
    }
}
