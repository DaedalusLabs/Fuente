use consumer::{
    contexts::{
        cart::CartProvider,
        commerce_data::CommerceDataProvider,
        consumer_data::{ConsumerDataProvider, ConsumerDataStore},
        live_order::{LiveOrderProvider, LiveOrderStore},
    },
    pages::new_user::NewProfilePage,
    router::ConsumerPages,
};
use fuente::{
    contexts::{
        key_manager::{NostrIdProvider, NostrIdStore},
        relay_pool::RelayProvider,
    },
    mass::{
        atoms::layouts::{LoadingScreen, MainLayout},
        molecules::login::NewUserPage,
    },
    models::{
        orders::{OrderPaymentStatus, OrderStatus},
        relays::UserRelay,
    },
};
use html::ChildrenProps;
use yew::prelude::*;
use yew_router::BrowserRouter;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
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
    if user_ctx.is_none() || key_ctx.is_none() {
        return html! {<LoadingScreen />};
    }
    let key_ctx = key_ctx.unwrap();
    let user_ctx = user_ctx.unwrap();
    if !key_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    if key_ctx.get_key().is_none() {
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
