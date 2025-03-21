use consumer::{
    contexts::{
        CartProvider, CommerceDataExt, CommerceDataProvider, CommerceDataStore,
        ConsumerDataProvider, FavoritesProvider, LiveOrderProvider, LoginProvider, RatingsProvider,
    },
    router::ConsumerPages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore, LanguageConfigsProvider},
    mass::{LoadingScreen, PwaInstall, ToastProvider},
    models::init_consumer_db,
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
        init_consumer_db().expect("Error initializing consumer database");
        spawn_local(async move {
            let sw = nostr_minions::browser_api::AppServiceWorker::new()
                .expect("Error initializing service worker");
            if let Err(e) = sw.install("serviceWorker.js").await {
                gloo::console::error!("Error installing service worker: {:?}", e);
            }
        });
        || {}
    });

    html! {
        <LanguageConfigsProvider>
            <PwaInstall />
            <BrowserRouter>
                <RelayPoolComponent>
                    <NostrIdProvider>
                        <AdminConfigsProvider>
                            <LoginProvider>
                                <CommerceDataProvider>
                                    <RatingsProvider>
                                    <AppContext>
                                        <ConsumerPages />
                                    </AppContext>
                                    </RatingsProvider>
                                </CommerceDataProvider>
                            </LoginProvider>
                        </AdminConfigsProvider>
                    </NostrIdProvider>
                </RelayPoolComponent>
            </BrowserRouter>
        </LanguageConfigsProvider>
    }
}

#[function_component(RelayPoolComponent)]
fn relay_pool_component(props: &ChildrenProps) -> Html {
    let relays = include_str!("../../relays.txt")
        .trim()
        .lines()
        .map(|url| UserRelay {
            url: url.trim().to_string(),
            read: true,
            write: true,
        })
        .collect::<Vec<UserRelay>>();
    html! {
        <RelayProvider {relays}>
            {props.children.clone()}
        </RelayProvider>
    }
}

#[function_component(AuthContext)]
fn app_context(props: &ChildrenProps) -> Html {
    html! {
        <NostrIdProvider>
            <AdminConfigsProvider>
                {props.children.clone()}
            </AdminConfigsProvider>
        </NostrIdProvider>
    }
}

#[function_component(AppContext)]
fn app_context(props: &ChildrenProps) -> Html {
    html! {
       <ConsumerDataProvider>
          <CartProvider>
              <LiveOrderProvider>
                  <FavoritesProvider>
                        <ToastProvider>
                          <LoadingCheck>
                              {props.children.clone()}
                            </LoadingCheck>
                          </ToastProvider>
                  </FavoritesProvider>
              </LiveOrderProvider>
          </CartProvider>
       </ConsumerDataProvider>
    }
}

#[function_component(LoadingCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let admin_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");
    if !key_ctx.loaded() || !admin_ctx.is_loaded() || !commerce_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    html! {
        <>
            {props.children.clone()}
        </>
    }
}
