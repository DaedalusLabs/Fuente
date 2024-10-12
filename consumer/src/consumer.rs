use consumer::{
    contexts::{
        cart::CartProvider,
        commerce_data::CommerceDataProvider,
        consumer_data::{ConsumerDataProvider, ConsumerDataStore},
        live_order::LiveOrderProvider,
    },
    pages::new_user::NewProfilePage,
    router::ConsumerPages,
};
use html::ChildrenProps;
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
                            <ConsumerPages />
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
            <ConsumerDataProvider>
                <CommerceDataProvider>
                    <CartProvider>
                        <LiveOrderProvider>
                            {props.children.clone()}
                        </LiveOrderProvider>
                    </CartProvider>
                </CommerceDataProvider>
            </ConsumerDataProvider>
        </NostrIdProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>();
    let user_ctx = use_context::<ConsumerDataStore>();
    if user_ctx.is_none() || key_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    let key_ctx = key_ctx.unwrap();
    let user_ctx = user_ctx.unwrap();
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
    if !user_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    if user_ctx.get_profile().is_none() {
        return html! {
            <div class="flex flex-col flex-1">
                <h2 class="text-2xl m-8 font-bold">{"Save Your Contact Details"}</h2>
                <NewProfilePage />
            </div>
        };
    }
    html! {
        {props.children.clone()}
    }
}
