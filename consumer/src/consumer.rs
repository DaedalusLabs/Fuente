use consumer::{
    contexts::{
        CartProvider, CommerceDataProvider, ConsumerDataProvider, ConsumerDataStore,
        FavoritesProvider, LiveOrderProvider,
    },
    pages::{NewAddressPage, NewProfilePage},
    router::ConsumerPages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore, LanguageConfigsProvider},
    mass::{LoadingScreen, LoginPage},
    models::init_consumer_db,
};
use html::ChildrenProps;
use nostr_minions::{
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
        init_consumer_db().expect("Error initializing consumer database");
        || {}
    });
    html! {
    <LanguageConfigsProvider>
        <BrowserRouter>
            <RelayPoolComponent>
                <AuthContext>
                    <LoginCheck>
                        <AppContext>
                            <ProfileCheck>
                                <ConsumerPages />
                            </ProfileCheck>
                        </AppContext>
                    </LoginCheck>
                </AuthContext>
            </RelayPoolComponent>
        </BrowserRouter>
    </LanguageConfigsProvider>
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
           <CommerceDataProvider>
               <CartProvider>
                   <LiveOrderProvider>
                       <FavoritesProvider>
                           {props.children.clone()}
                       </FavoritesProvider>
                   </LiveOrderProvider>
               </CartProvider>
           </CommerceDataProvider>
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
    if !user_ctx.finished_loading() {
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
