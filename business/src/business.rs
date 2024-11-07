use business::{
    contexts::{CommerceDataProvider, CommerceDataStore, ConsumerDataProvider, OrderDataProvider},
    pages::NewProfilePage,
    router::CommercePages,
};
use fuente::{
    contexts::{
        init_nostr_db, {NostrIdProvider, NostrIdStore}, {RelayProvider, UserRelay},
    },
    mass::{LoadingScreen, MainLayout, NewUserPage},
    models::{init_commerce_db, init_consumer_db},
};
use html::ChildrenProps;
use yew::prelude::*;
use yew_router::BrowserRouter;

// 80748881f453306f3129e3a040de263f3dd62726ba03273c248ac33cac59e0c5
// 566688e5ae72ee7875376a9f2d6c6032ef0bbac1df9ed3d972eb8135a0f022a0

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    use_effect_with((), move |_| {
        init_commerce_db().unwrap();
        init_consumer_db().unwrap();
        init_nostr_db().unwrap();
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
    if key_ctx.get_nostr_key().is_none() {
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
