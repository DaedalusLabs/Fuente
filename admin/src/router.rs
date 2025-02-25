use fuente::mass::{templates::FuenteSidebarTemplate, AppLink};
use lucide_yew::{Bitcoin, Cog, House, Store, Truck};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{
    CommercesPage, CourierWhitelistPage, ExchangeRatePage, HomePage, SettingsPageComponent,
};

#[derive(Clone, Routable, PartialEq)]
pub enum AdminPanelRoute {
    #[at("/")]
    Home,
    #[at("/exchange")]
    Exchange,
    #[at("/commerces")]
    Commerces,
    #[at("/couriers")]
    Couriers,
    #[at("/consumers")]
    Consumers,
    #[at("/settings")]
    Settings,
}

#[function_component(AdminPanelPages)]
pub fn consumer_pages() -> Html {
    html! {
        <div class="flex flex-col lg:flex-row h-screen overflow-hidden">
            <HomeSidebar />
            <Switch<AdminPanelRoute> render = { move |switch: AdminPanelRoute| {
                    match switch {
                        AdminPanelRoute::Home => html!{
                            <HomePage />
                        },
                        AdminPanelRoute::Settings => html!{
                            <SettingsPageComponent />
                        },
                        AdminPanelRoute::Exchange => html! {
                            <ExchangeRatePage />
                        },
                        AdminPanelRoute::Commerces => html! {
                            <CommercesPage />
                        },
                        AdminPanelRoute::Couriers => html! {
                            <CourierWhitelistPage />
                        },
                        AdminPanelRoute::Consumers => html!{
                            <></>
                        },
                    }
                }}
            />
        </div>
    }
}
#[function_component(HomeSidebar)]
pub fn home_footer() -> Html {
    html! {
        <FuenteSidebarTemplate>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Home}>
                <House class="w-8 h-8 text-fuente" />
            </AppLink<AdminPanelRoute>>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Exchange}>
                <Bitcoin class="w-8 h-8 text-fuente" />
            </AppLink<AdminPanelRoute>>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Commerces}>
                <Store class="w-8 h-8 text-fuente" />
            </AppLink<AdminPanelRoute>>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Couriers}>
                <Truck class="w-8 h-8 text-fuente" />
            </AppLink<AdminPanelRoute>>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Settings}>
                <Cog class="w-8 h-8 text-fuente" />
            </AppLink<AdminPanelRoute>>
        </FuenteSidebarTemplate>
    }
}
