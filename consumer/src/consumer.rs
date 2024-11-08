use consumer::{
    contexts::{
        CartProvider, CommerceDataProvider, ConsumerDataAction, ConsumerDataProvider,
        ConsumerDataStore, LiveOrderProvider,
    },
    pages::NewProfilePage,
    router::ConsumerPages,
};
use fuente::{
    browser_api::GeolocationCoordinates,
    contexts::{init_nostr_db, NostrIdProvider, NostrIdStore, RelayProvider, UserRelay},
    mass::{
        NewUserPage, {LoadingScreen, MainLayout}, {NewAddressMenu, NewAddressProps},
    },
    models::{
        init_consumer_db, {ConsumerAddress, ConsumerAddressIdb},
    },
};
use html::ChildrenProps;
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
    let coordinate_state = use_state(|| None::<GeolocationCoordinates>);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
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
            <div class="flex flex-col flex-1">
                <h2 class="text-2xl m-8 font-bold">{"Save Your Address"}</h2>
                <NewAddressMenu ..props />
            </div>
        };
    }
    html! {
        <>
        {props.children.clone()}
        </>
    }
}
