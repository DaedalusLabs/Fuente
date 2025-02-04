use consumer::{
    contexts::{
        CartProvider, CommerceDataProvider, CommerceDataStore, ConsumerDataProvider,
        ConsumerDataStore, FavoritesProvider, LiveOrderProvider, LiveOrderStore, LoginModal,
        LoginProvider, RatingsProvider,
    },
    pages::{NewAddressPage, NewProfilePage},
    router::ConsumerPages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore, LanguageConfigsProvider},
    mass::{LoadingScreen, LoginPage, PwaInstall, ToastProvider},
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
            <BrowserRouter>
                <RelayPoolComponent>
                    <NostrIdProvider>
                        <AdminConfigsProvider>
                            <LoginProvider>
                                <CommerceDataProvider>
                                    <RatingsProvider>
                                    <AppContext>
                                        <ConsumerPages />
                                        <LoginModal />
                                        <PwaInstall />
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

// New component to handle authentication checks
#[function_component(AuthenticationCheck)]
fn authentication_check() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");

    // Show loading state while context is initializing
    if !key_ctx.finished_loading() {
        return html! {
            <LoadingScreen />
        };
    }

    // Show login page or main app based on authentication
    if key_ctx.get_nostr_key().is_none() {
        return html! {
            <LoginPage />
        };
    }

    // Main app content when authenticated
    html! {
        <AppContext>
            <ConsumerPages />
            <PwaInstall />
        </AppContext>
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
                              {props.children.clone()}
                          </ToastProvider>
                  </FavoritesProvider>
              </LiveOrderProvider>
          </CartProvider>
       </ConsumerDataProvider>
    }
}
#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let admin_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    if !key_ctx.finished_loading() || !admin_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    if key_ctx.get_nostr_key().is_none() {
        return html! {
            <LoginPage />
        };
    }
    html! {
        <>
            {props.children.clone()}
        </>
    }
}
#[function_component(ProfileCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let user_ctx = use_context::<ConsumerDataStore>().expect("ConsumerDataStore not found");
    let order_ctx = use_context::<LiveOrderStore>().expect("No order context found");
    let commerce_ctx = use_context::<CommerceDataStore>().expect("No commerce context found");
    if !order_ctx.has_loaded || !commerce_ctx.finished_loading() || !user_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    if user_ctx.get_profile().is_none() {
        return html! {
            <NewProfilePage />
        };
    }
    if user_ctx.get_default_address().is_none() {
        return html! {
            <NewAddressPage />
        };
    }
    html! {
        <>
            {props.children.clone()}
        </>
    }
}
