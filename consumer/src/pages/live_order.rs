use fuente::{
    browser_api::clipboard_copy,
    mass::{CancelIcon, DriverDetailsComponent, OrderRequestDetailsComponent, SpinnerIcon},
    models::{DriverProfileIdb, OrderInvoiceState, OrderPaymentStatus, OrderStatus},
};
use lightning::LndHodlInvoice;
use yew::prelude::*;

use crate::contexts::LiveOrderStore;

#[function_component(LiveOrderCheck)]
pub fn live_order_check() -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let inside_html = if let Some(order) = &order_ctx.order {
        match order.1.get_payment_status() {
            OrderPaymentStatus::PaymentPending => Ok(html! {
                <>
                    <h2 class="text-2xl font-bold">{"Order Received!"}</h2>
                    <OrderInvoiceComponent invoice={order.1.get_consumer_invoice().unwrap()} />
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
                <div class="h-full w-full flex items-center justify-center">
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
pub struct OrderInvoiceComponentProps {
    pub invoice: LndHodlInvoice,
}

#[function_component(OrderInvoiceComponent)]
pub fn order_invoice_details(props: &OrderInvoiceComponentProps) -> Html {
    let OrderInvoiceComponentProps { invoice } = props.clone();
    let invoice_pr = invoice.payment_request();
    let onclick_copy = {
        let pr = invoice_pr.clone();
        Callback::from(move |_| {
            let _ = clipboard_copy(&pr);
        })
    };
    html! {
        <div class="flex flex-col gap-4 items-center text-center">
            <p class="text-sm font-bold text-gray-500">
                {"Tap the invoice to open in your wallet."}
            </p>
            <BitcoinQrCode
                id={"qr".to_string()} width={"200".to_string()} height={"200".to_string()}
                lightning={invoice_pr.clone()} type_="svg" />
            <p class="text-sm font-bold text-gray-500">
                {format!("{}...", invoice_pr[..12].to_string())}
            </p>
            <button
                class="bg-fuente-light text-white p-2 rounded-md font-mplus"
                onclick={onclick_copy}>
                {"Copy Invoice"}
            </button>
        </div>
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
