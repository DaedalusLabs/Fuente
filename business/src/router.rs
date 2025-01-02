use fuente::mass::AppLink;
use lucide_yew::{BellPlus, History, House, Settings, Store};
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
        <div class="flex h-screen ">
            <HomeSidebar />
            <main class="flex-1">
                <div class="h-full no-scrollbar">
                    <div class="container mx-auto p-6">
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
                </div>
            </main>
        </div>
    }
}
#[function_component(HomeSidebar)]
pub fn home_footer() -> Html {
    html! {
        <aside class="flex w-16 flex-col items-center space-y-8 border-r border-fuente py-8">
            <img
                class={"min-w-10 min-h-10 max-w-10 max-h-10"}
                src={"/public/assets/img/logo.png"}
                alt="avatar" />
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
            // Add settings link (using MenuBarsIcon)
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Settings}>
                <Settings class="w-8 h-8 stroke-fuente" />
            </AppLink<CommerceRoute>>
        </aside>
    }
}
