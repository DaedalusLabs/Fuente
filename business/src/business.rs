use business::{
    contexts::{
        commerce_data::{CommerceDataProvider, CommerceDataStore},
        consumer_data::ConsumerDataProvider,
        order_data::OrderDataProvider,
    },
    pages::new_user::NewProfilePage,
    router::CommercePages,
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
    models::{init_commerce_db, init_consumer_db, init_shared_db, relays::UserRelay},
};
use yew::prelude::*;
use yew_router::BrowserRouter;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    use_effect_with((), move |_| {
        init_shared_db().unwrap();
        init_commerce_db().unwrap();
        init_consumer_db().unwrap();
        || {}
    });
    html! {
        <BrowserRouter>
            <RelayProviderComponent>
                <AppContext>
                    <MainLayout>
                        <LoginCheck>
                            <CommercePages />
                        </LoginCheck>
                    </MainLayout>
                </AppContext>
            </RelayProviderComponent>
        </BrowserRouter>
    }
}

#[function_component(RelayProviderComponent)]
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
            <CommerceDataProvider>
                <ConsumerDataProvider >
                    <OrderDataProvider>
                        {props.children.clone()}
                    </OrderDataProvider>
                </ConsumerDataProvider>
            </CommerceDataProvider>
        </NostrIdProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>();
    let user_ctx = use_context::<CommerceDataStore>();
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
    if !user_ctx.checked_db() {
        return html! {<LoadingScreen />};
    }
    if user_ctx.profile().is_none() && user_ctx.checked_relay() {
        return html! {
            <div class="flex flex-col w-full h-full overflow-y-scroll">
                <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Save Your Contact Details"}</h2>
                <NewProfilePage />
            </div>
        };
    }
    html! {
        {props.children.clone()}
    }
}
