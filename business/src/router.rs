use fuente::mass::{AppLink, CategoriesIcon, HistoryIcon, HomeIcon, MenuBarsIcon};
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
        <div class="w-dvw h-dvh flex flex-row">
            <div class="h-full border-r-2 border-black">
                <div class="w-fit h-full px-4 py-8 gap-8 items-center flex flex-col">
                    <img
                        class={"min-w-10 min-h-10 max-w-10 max-h-10"}
                        src={"/public/assets/img/logo.png"}
                        alt="avatar" />
                    <HomeSidebar />
                </div>
            </div>
            <div class="flex-1">
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
    }
}
#[function_component(HomeSidebar)]
pub fn home_footer() -> Html {
    html! {
        <div class="w-fit flex flex-col gap-8 items-center">
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Home}>
                <HomeIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<CommerceRoute>>
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Products}>
                <CategoriesIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<CommerceRoute>>
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::History}>
                <HistoryIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<CommerceRoute>>
            // Add settings link (using MenuBarsIcon)
            <AppLink<CommerceRoute>
                class="" selected_class=""
                route={CommerceRoute::Settings}>
                <MenuBarsIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<CommerceRoute>>
        </div>
    }
}
