use business::{
    contexts::{CommerceDataProvider, CommerceDataStore, ConsumerDataProvider, OrderDataProvider},
    pages::NewProfilePage,
    router::CommercePages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore, LanguageConfigsProvider},
    mass::{LoadingScreen, LoginPage},
    models::{init_commerce_db, init_consumer_db},
};
use html::ChildrenProps;
use nostr_minions::{
    init_nostr_db,
    key_manager::{NostrIdProvider, NostrIdStore},
    relay_pool::{RelayProvider, UserRelay},
};
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
        spawn_local(async move {
            let sw = nostr_minions::browser_api::AppServiceWorker::new()
                .expect("Error initializing service worker");
            sw.install("serviceWorker.js")
                .await
                .expect("Error installing service worker");
        });
        || {}
    });
    html! {
        <LanguageConfigsProvider>
        <BrowserRouter>
            <RelayProviderComponent>
                <AppContext>
                    <LoginCheck>
                        <CommercePages />
                    </LoginCheck>
                </AppContext>
            </RelayProviderComponent>
        </BrowserRouter>
        </LanguageConfigsProvider>
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
            <AdminConfigsProvider >
                <CommerceDataProvider>
                    <ConsumerDataProvider >
                        <OrderDataProvider>
                            {props.children.clone()}
                        </OrderDataProvider>
                    </ConsumerDataProvider>
                </CommerceDataProvider>
            </AdminConfigsProvider>
        </NostrIdProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let user_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    let config_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    if !key_ctx.finished_loading() || !config_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    if key_ctx.get_nostr_key().is_none() {
        return html! { <LoginPage /> };
    }
    if !user_ctx.checked_db() {
        return html! {<LoadingScreen />};
    }
    if user_ctx.profile().is_none() && user_ctx.checked_relay() {
        return html! {
            <NewProfilePage />
        };
    }
    let whitelist = config_ctx.get_commerce_whitelist();
    if !whitelist.contains(&key_ctx.get_nostr_key().unwrap().public_key()) {
        return html! {
            <div class="flex justify-center items-center flex-1">
                <h2 class="text-2xl px-8 py-4 font-bold text-center">{"You are not yet authorized to access this page"}</h2>
            </div>
        };
    }
    html! {
        {props.children.clone()}
    }
}
