use fuente::mass::{AppLink, HomeIcon, MotoIcon, StoreIcon, UserBadgeIcon};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::CommerceDisplay;

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
        <div class="w-full h-full flex flex-row">
            <div class="h-full border-r-2 border-black">
                <div class="w-fit h-full px-4 py-8 gap-8 items-center flex flex-col">
                    <img
                        class={"min-w-10 min-h-10 max-w-10 max-h-10"}
                        src={"/public/assets/img/logo.png"}
                        alt="avatar" />
                    <HomeSidebar />
                </div>
            </div>
            <div class="flex flex-1 pl-8 pt-8 overflow-auto no-scrollbar">
                <Switch<AdminPanelRoute> render = { move |switch: AdminPanelRoute| {
                        match switch {
                            AdminPanelRoute::Home => html!{<>
                                <h1>{"Home"}</h1>
                                </>},
                            AdminPanelRoute::Settings => html!{<>
                                <h1>{"Settings"}</h1>
                                </>},
                            AdminPanelRoute::Exchange => html!{<>
                                <h1>{"Exchange"}</h1>
                                </>},
                            AdminPanelRoute::Commerces => html!{<>
                                <h1>{"Commerces"}</h1>
                                <CommerceDisplay />
                                </>},
                            AdminPanelRoute::Couriers => html!{<>
                                <h1>{"Couriers"}</h1>
                                </>},
                            AdminPanelRoute::Consumers => html!{<>
                                <h1>{"Consumers"}</h1>
                                </>},
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
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Home}>
                <HomeIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<AdminPanelRoute>>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Commerces}>
                <StoreIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<AdminPanelRoute>>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Couriers}>
                <MotoIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<AdminPanelRoute>>
            <AppLink<AdminPanelRoute>
                class="" selected_class=""
                route={AdminPanelRoute::Consumers}>
                <UserBadgeIcon class="w-8 h-8 stroke-neutral-400" />
            </AppLink<AdminPanelRoute>>
        </div>
    }
}
