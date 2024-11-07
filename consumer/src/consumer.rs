use consumer::{
    contexts::{
        CartProvider, CommerceDataProvider, ConsumerDataAction, ConsumerDataProvider,
        ConsumerDataStore, LiveOrderProvider, LiveOrderStore,
    },
    pages::NewProfilePage,
    router::ConsumerPages,
};
use fuente::{
    browser_api::GeolocationCoordinates,
    contexts::{init_nostr_db, NostrIdProvider, NostrIdStore, RelayProvider, UserRelay},
    mass::{
        DriverDetailsComponent, NewUserPage, OrderRequestDetailsComponent,
        {CancelIcon, SpinnerIcon}, {LoadingScreen, MainLayout}, {NewAddressMenu, NewAddressProps},
    },
    models::{
        init_consumer_db, DriverProfileIdb, {ConsumerAddress, ConsumerAddressIdb},
        {OrderPaymentStatus, OrderStatus},
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
        <LiveOrderCheck />
        </>
    }
}

#[function_component(LiveOrderCheck)]
fn live_order_check() -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let inside_html = if let Some(order) = &order_ctx.order {
        match order.1.get_payment_status() {
            OrderPaymentStatus::PaymentPending => Ok(html! {
                <>
                    <h2 class="text-2xl font-bold">{"Order Received!"}</h2>
                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                        <p>{"Order ID: "}{order.1.id()}</p>
                        <p class="max-w-md text-wrap">{"Invoice: "}{order.1.get_consumer_invoice()}</p>
                        <BitcoinQrCode
                            id={"qr".to_string()} width={"200".to_string()} height={"200".to_string()}
                            lightning={order.1.get_consumer_invoice().expect("").payment_request()} type_="svg" />
                    </div>
                </>
            }),
            OrderPaymentStatus::PaymentReceived => Ok(html! {
                <>
                    <div class="bg-white p-8 rounded-lg">
                        <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                    </div>
                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                        <p>{"Order ID: "}{order.1.id()[..12].to_string()}</p>
                        <p>{"Waiting for confirmation..."}</p>
                    </div>
                </>
            }),
            OrderPaymentStatus::PaymentSuccess => {
                let order = order.1.clone();
                let status = order.get_order_status();
                if status == OrderStatus::Completed || status == OrderStatus::Canceled {
                    Err(html! {<></>})
                } else {
                    if let Some(courier_note) = order.get_courier() {
                        let driver_db = DriverProfileIdb::try_from(courier_note).unwrap();
                        let driver = driver_db.profile();
                        let pubkey = driver_db.pubkey();

                        Ok(html! {
                            <>
                                <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                                <div class="flex flex-col gap-4 text-wrap max-w-md">
                                    <p>{"Order ID: "}{order.id()[..12].to_string()}</p>
                                    <p>{"Order Status: "}{order.get_order_status()}</p>
                                    <DriverDetailsComponent {pubkey} {driver} />
                                </div>
                            </>
                        })
                    } else {
                        Ok(html! {
                            <>
                                <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                                <div class="flex flex-col gap-4 text-wrap max-w-md">
                                    <p>{"Order ID: "}{order.id()[..12].to_string()}</p>
                                    <p>{"Order Status: "}{order.get_order_status()}</p>
                                </div>
                            </>
                        })
                    }
                }
            }
            _ => Err(html! {<></>}),
        }
    } else {
        Err(html! {<></>})
    };
    match inside_html {
        Err(e) => e,
        Ok(inside_html) => {
            let order = order_ctx
                .order
                .clone()
                .unwrap()
                .1
                .get_order_request()
                .products;
            html! {
                <div class="fixed w-dvw h-dvh bg-black flex items-center justify-center z-20">
                    <div class="relative bg-white p-4 flex flex-col gap-4">
                        <OrderRequestDetailsComponent {order} />
                        {inside_html}
                        <SpinnerIcon class="absolute top-4 right-4 w-4 h-4 text-fuente" />
                    </div>
                    <button class="absolute top-4 right-4">
                        <CancelIcon class="w-8 h-8 text-red-500" />
                    </button>

                </div>
            }
        }
    }
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
