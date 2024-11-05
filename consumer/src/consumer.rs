use consumer::{
    contexts::{
        cart::CartProvider,
        commerce_data::CommerceDataProvider,
        consumer_data::{ConsumerDataAction, ConsumerDataProvider, ConsumerDataStore},
        live_order::{LiveOrderProvider, LiveOrderStore},
    },
    pages::new_user::NewProfilePage,
    router::ConsumerPages,
};
use fuente::{
    browser_api::GeolocationCoordinates,
    contexts::{
        init_nostr_db,
        key_manager::{NostrIdProvider, NostrIdStore},
        relay_pool::{RelayProvider, UserRelay},
    },
    mass::{
        atoms::layouts::{LoadingScreen, MainLayout},
        molecules::{
            address::{NewAddressMenu, NewAddressProps},
            login::NewUserPage,
        },
    },
    models::{
        address::{ConsumerAddress, ConsumerAddressIdb},
        init_consumer_db,
        orders::{OrderPaymentStatus, OrderStatus},
    },
    widgets::leaflet::{
        IconOptions, LatLng, LeafletMap, LeafletMapOptions, Marker, NominatimLookup, L,
    },
};
use html::ChildrenProps;
use wasm_bindgen::JsValue;
use yew::{platform::spawn_local, prelude::*, props};
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
        <LiveOrderCheck />
        </>
    }
}

pub fn start_new_address_picker_map(
    location: GeolocationCoordinates,
    map_handler: UseStateHandle<Option<LeafletMap>>,
    marker_handler: UseStateHandle<Option<Marker>>,
    geo_handler: UseStateHandle<Option<GeolocationCoordinates>>,
    address_handler: UseStateHandle<Option<NominatimLookup>>,
) -> Result<(), JsValue> {
    let map_options = LeafletMapOptions {
        double_click_zoom: false,
        center: Some(location.clone().into()),
        ..Default::default()
    };
    let map = L::render_map_with_options("new-user-map", map_options)?;
    map_handler.set(Some(map.clone()));
    let icon_options = IconOptions {
        icon_url: "public/assets/marker.png".to_string(),
        icon_size: Some(vec![32, 32]),
        icon_anchor: None,
    };
    let marker = map.add_marker_with_icon(&location, icon_options)?;
    marker_handler.set(Some(marker.clone()));
    geo_handler.set(Some(location));

    let geo_handler_clone = geo_handler.clone();
    let address_handler_clone = address_handler.clone();
    let map_closure = move |e: MouseEvent| {
        let leaflet_event = LatLng::try_from(e).expect("Could not parse event");
        let coordinates: GeolocationCoordinates = leaflet_event.clone().into();
        geo_handler_clone.set(Some(coordinates.clone()));
        marker.set_lat_lng(
            &leaflet_event
                .try_into()
                .expect("Could not conver to Js value"),
        );
        let handle = address_handler_clone.clone();
        spawn_local(async move {
            if let Ok(address) = NominatimLookup::reverse(coordinates.clone()).await {
                handle.set(Some(address));
            }
        });
    };
    map.add_closure("dblclick", map_closure);

    Ok(())
}

#[function_component(LiveOrderCheck)]
fn live_order_check() -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    if let Some(order) = &order_ctx.order {
        match order.1.get_payment_status() {
            OrderPaymentStatus::PaymentPending => {
                return html! {
                    <div class="fixed inset-0 bg-black flex justify-center items-center z-20">
                        <div class="bg-white p-8 rounded-lg w-fit h-fit text-wrap max-w-lg">
                        <div class="bg-white p-8 rounded-lg">
                            <h2 class="text-2xl font-bold">{"Order Received!"}</h2>
                        </div>
                        <div class="flex flex-col gap-4 text-wrap max-w-md">
                            <p>{"Order ID: "}{order.1.id()}</p>
                            <p class="max-w-md text-wrap">{"Invoice: "}{order.1.get_consumer_invoice()}</p>
                            <BitcoinQrCode
                                id={"qr".to_string()} width={"200".to_string()} height={"200".to_string()}
                                lightning={order.1.get_consumer_invoice().expect("").payment_request()} type_="svg" />
                        </div>
                        <button class="absolute top-4 right-4">
                            {"Close"}
                        </button>
                        </div>
                    </div>
                };
            }
            OrderPaymentStatus::PaymentReceived => {
                return html! {
                    <div class="fixed inset-0 bg-black flex justify-center items-center z-20">
                        <div class="bg-white p-8 rounded-lg w-fit h-fit text-wrap max-w-lg">
                        <div class="bg-white p-8 rounded-lg">
                            <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                        </div>
                        <div class="flex flex-col gap-4 text-wrap max-w-md">
                            <p>{"Order ID: "}{order.1.id()}</p>
                            <p>{"Waiting for confirmation..."}</p>
                        </div>
                        <button class="absolute top-4 right-4">
                            {"Close"}
                        </button>
                        </div>
                    </div>
                };
            }
            OrderPaymentStatus::PaymentSuccess => match order.1.get_order_status() {
                OrderStatus::Completed => {}
                OrderStatus::Canceled => {}
                _ => {
                    return html! {
                        <div class="fixed inset-0 bg-black flex justify-center items-center z-20">
                            <div class="bg-white p-8 rounded-lg w-fit h-fit text-wrap max-w-lg">
                            <div class="bg-white p-8 rounded-lg">
                                <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                            </div>
                            <div class="flex flex-col gap-4 text-wrap max-w-md">
                                <p>{"Order ID: "}{order.1.id()}</p>
                                <p>{"Order Status: "}{order.1.get_order_status()}</p>
                                <p>{"Courier: "}{order.1.get_courier()}</p>
                            </div>
                            <button class="absolute top-4 right-4">
                                {"Close"}
                            </button>
                            </div>
                        </div>
                    };
                }
            },
            _ => {}
        }
    };
    html! {<></>}
}
#[derive(Properties, Clone, PartialEq)]
pub struct BitcoinQrCodeProps {
    pub id: String,
    pub width: String,
    pub height: String,
    pub lightning: String,
    pub type_: String,
}

#[function_component(BitcoinQrCode)]
pub fn bitcoin_qr(props: &BitcoinQrCodeProps) -> Html {
    let BitcoinQrCodeProps {
        id,
        width,
        height,
        type_,
        lightning,
    } = props.clone();
    html! {
    <bitcoin-qr
        {id}
        {width}
        {height}
        {lightning}
        type={type_}
        corners-square-color="#b23c05"
        corners-dot-color={"#e24a04"}
        corners-square-type={"extra-rounded"}
        dots-type={"classy-rounded"}
        dots-color={"#ff5000"}
    />
    }
}
