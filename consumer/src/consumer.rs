use consumer::{
    contexts::{
        CartProvider, CommerceDataProvider, ConsumerDataAction, ConsumerDataProvider,
        ConsumerDataStore, LiveOrderProvider,
    },
    pages::NewProfilePage,
    router::ConsumerPages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore},
    mass::{LoadingScreen, MainLayout, NewAddressForm, NewAddressProps, NewUserPage},
    models::{init_consumer_db, ConsumerAddress, ConsumerAddressIdb},
};
use html::ChildrenProps;
use minions::{
    browser_api::GeolocationCoordinates,
    init_nostr_db,
    key_manager::{NostrIdProvider, NostrIdStore},
    relay_pool::{RelayProvider, UserRelay},
};
use yew::{prelude::*, props};
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
        <BrowserRouter>
            <RelayPoolComponent>
                <AuthContext>
                    <MainLayout>
                    <LoginCheck>
                        <AppContext>
                            <ProfileCheck>
                                <ConsumerPages />
                            </ProfileCheck>
                        </AppContext>
                    </LoginCheck>
                    </MainLayout>
                </AuthContext>
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
                       {props.children.clone()}
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
            <div class="flex justify-center items-center flex-1">
                <NewUserPage />
            </div>
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
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let user_ctx = use_context::<ConsumerDataStore>().expect("ConsumerDataStore not found");
    let coordinate_state = use_state(|| None::<GeolocationCoordinates>);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
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
    let nominatim = nominatim_state.clone();
    let coordinate = coordinate_state.clone();
    if user_ctx.get_default_address().is_none() {
        let onclick = Callback::from(move |_: MouseEvent| {
            if let (Some(address), Some(coords), Some(keys)) = (
                (*nominatim).clone(),
                (*coordinate).clone(),
                key_ctx.get_nostr_key(),
            ) {
                let address = ConsumerAddress::new(address, coords.into());
                let mut db_entry = ConsumerAddressIdb::new(address.clone(), &keys);
                db_entry.set_default(true);
                user_ctx.dispatch(ConsumerDataAction::NewAddress(db_entry));
            }
        });
        let props = props!(NewAddressProps {
            map_handle: map_state,
            marker_handle: marker_state,
            coord_handle: coordinate_state.clone(),
            nominatim_handle: nominatim_state.clone(),
            onclick,
        });
        return html! {
            <div class="flex flex-col flex-1 p-8">
                <h2 class="text-2xl m-8 font-bold">{"Save Your Address"}</h2>
                <NewAddressForm ..props />
            </div>
        };
    }
    html! {
        <>
            {props.children.clone()}
        </>
    }
}
