use admin::{AdminPanelPages, ServerConfigsProvider, ServerConfigsStore};
use fuente::{
    mass::{AdminLoginPage, LoadingScreen, MainLayout},
    models::{init_commerce_db, init_consumer_db, ADMIN_WHITELIST},
};
use html::ChildrenProps;
use minions::{
    init_nostr_db,
    key_manager::{NostrIdProvider, NostrIdStore},
    relay_pool::{RelayProvider, UserRelay},
};
use yew::prelude::*;
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
        || {}
    });

    html! {
        <BrowserRouter>
           <AppContext>
               <MainLayout>
                   <LoginCheck>
                    <AdminPanelPages />
                   </LoginCheck>
               </MainLayout>
           </AppContext>
        </BrowserRouter>
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
            <div class="flex justify-center items-center flex-1">
                <AdminLoginPage />
            </div>
        };
    }
    if !server_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    if !ADMIN_WHITELIST.contains(&keys.unwrap().get_public_key().as_str()) {
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
