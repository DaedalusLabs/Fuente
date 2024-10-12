use driver::{
    contexts::{
        commerce_data::CommerceDataProvider,
        driver_data::{DriverDataProvider, DriverDataStore},
        live_order::OrderHubProvider,
    },
    router::DriverPages,
};
use fuente::{
    contexts::{
        key_manager::{NostrIdProvider, NostrIdStore},
        relay_pool::RelayProvider,
    },
    mass::{
        atoms::layouts::{LoadingScreen, MainLayout},
        molecules::login::NewUserPage,
    },
    models::relays::UserRelay,
};
use html::ChildrenProps;
use yew::prelude::*;
use yew_router::BrowserRouter;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <RelayPoolComponent>
                <AppContext>
                    <MainLayout>
                        <LoginCheck>
                            <DriverPages />
                        </LoginCheck>
                    </MainLayout>
                </AppContext>
            </RelayPoolComponent>
        </BrowserRouter>
    }
}

#[function_component(RelayPoolComponent)]
fn relay_pool_component(props: &ChildrenProps) -> Html {
    let relays = vec![
        UserRelay {
            url: "wss://relay.arrakis.lat".to_string(),
            read: true,
            write: true,
        },
        UserRelay {
            url: "wss://relay.illuminodes.com".to_string(),
            read: true,
            write: true,
        },
    ];
    html! {
        <RelayProvider {relays}>
            {props.children.clone()}
        </RelayProvider>
    }
}

#[function_component(AppContext)]
fn app_context(props: &ChildrenProps) -> Html {
    html! {
        <NostrIdProvider>
            <DriverDataProvider>
                <CommerceDataProvider>
                    <OrderHubProvider>
                        {props.children.clone()}
                    </OrderHubProvider>
                </CommerceDataProvider>
            </ DriverDataProvider>
        </NostrIdProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>();
    let user_ctx = use_context::<DriverDataStore>();
    if user_ctx.is_none() || key_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    let key_ctx = key_ctx.unwrap();
    if !key_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    if key_ctx.get_key().is_none() {
        return html! {
            <div class="flex justify-center items-center flex-1">
                <NewUserPage />
            </div>
        };
    }
    let user_ctx = user_ctx.unwrap();
    if !user_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    html! {
        {props.children.clone()}
    }
}
