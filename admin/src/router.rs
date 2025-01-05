use fuente::mass::{templates::FuenteSidebarTemplate, AppLink};
use lucide_yew::{Bitcoin, House, Store, Truck, UserRound, Cog};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{CommerceDisplay, CommercesPage, CourierWhitelistPage, ExchangeRatePage, HomePage, SettingsPage, SettingsPageComponent};

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
        <div class="flex h-screen ">
            <HomeSidebar />
            <main class="flex-1">
                <div class="h-full no-scrollbar">
                    <div class="container mx-auto p-6">
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
                </div>
            </main>
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
