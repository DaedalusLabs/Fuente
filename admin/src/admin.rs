use admin::{AdminPanelPages, ServerConfigsProvider, ServerConfigsStore};
use fuente::{
    contexts::LanguageConfigsProvider,
    mass::{AdminLoginPage, LoadingScreen},
    models::{init_commerce_db, init_consumer_db, ADMIN_WHITELIST},
};
use html::ChildrenProps;
use nostr_minions::{
    init_nostr_db,
    key_manager::{NostrIdProvider, NostrIdStore},
    relay_pool::{RelayProvider, UserRelay},
};
use yew::{platform::spawn_local, prelude::*};
use yew_router::BrowserRouter;

fn main() {
    yew::Renderer::<App>::new().render();
}
#[function_component(App)]
fn app() -> Html {
    use_effect_with((), move |_| {
        init_nostr_db().expect("Error initializing Nostr database");
        init_consumer_db().expect("Error initializing Fuente database");
        init_commerce_db().expect("Error initializing Commerce database");
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
        <LanguageConfigsProvider >
        <BrowserRouter>
           <AppContext>
                <LoginCheck>
                    <AdminPanelPages />
                </LoginCheck>
           </AppContext>
        </BrowserRouter>
        </LanguageConfigsProvider>
    }
}

#[function_component(RelayProviderComponent)]
fn relay_pool_component(props: &ChildrenProps) -> Html {
    html! {
            {props.children.clone()}
    }
}

#[function_component(AppContext)]
fn app_context(props: &ChildrenProps) -> Html {
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
                <NostrIdProvider>
                    <ServerConfigsProvider>
                        {props.children.clone()}
                    </ServerConfigsProvider>
                </NostrIdProvider>
            </RelayProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let server_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    if !key_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    let keys = key_ctx.get_nostr_key();
    if keys.is_none() {
        return html! {
            <AdminLoginPage />
        };
    }
    if !server_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    if !ADMIN_WHITELIST.contains(&keys.unwrap().public_key().as_str()) {
        return html! {
            <div class="flex justify-center items-center flex-1">
                <h2 class="text-2xl px-8 py-4 font-bold text-center">{"You are not authorized to access this page"}</h2>
            </div>
        };
    }
    html! {
        {props.children.clone()}
    }
}
