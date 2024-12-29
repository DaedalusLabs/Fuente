use bright_lightning::LndHodlInvoice;
use fuente::{
    contexts::AdminConfigsStore,
    mass::{CancelIcon, DriverDetailsComponent, OrderRequestDetailsComponent, SpinnerIcon},
    models::{
        CommerceProfile, DriverProfileIdb, DriverStateUpdate, OrderInvoiceState,
        OrderPaymentStatus, OrderStatus, OrderUpdateRequest, NOSTR_KIND_CONSUMER_CANCEL,
        NOSTR_KIND_DRIVER_STATE,
    },
};
use html::ChildrenProps;
use nostr_minions::{
    browser_api::{clipboard_copy, GeolocationCoordinates},
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
    widgets::leaflet::{
        IconOptions, LatLng, LeafletComponent, LeafletMap, LeafletMapOptions, Marker,
    },
};
use yew::prelude::*;

use crate::contexts::{CommerceDataExt, CommerceDataStore, LiveOrderStore};

#[function_component(LiveOrderCheck)]
pub fn live_order_check(props: &ChildrenProps) -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let admin_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let exchange_rate = admin_ctx.get_exchange_rate();
    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");

    if order_ctx.order.is_none() {
        return html! {};
    }

    let inside_html = if let Some(order) = &order_ctx.order {
        let commerce_profile = commerce_ctx
            .find_commerce_by_id(&order.1.get_commerce_pubkey())
            .expect("Commerce not found");
        match order.1.payment_status {
            OrderPaymentStatus::PaymentPending => Ok(html! {
                <>
                    <h2 class="text-2xl font-bold">{"Order Received!"}</h2>
                    <OrderInvoiceComponent invoice={order.1.consumer_invoice.as_ref().cloned().unwrap()} {exchange_rate} />
                </>
            }),
            OrderPaymentStatus::PaymentReceived => Ok(html! {
                <>
                    <div class="bg-white p-8 rounded-lg">
                        <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                    </div>
                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                        <p>{"Order ID: "}{order.1.order_id()[..12].to_string()}</p>
                        <p>{"Waiting for confirmation..."}</p>
                    </div>
                </>
            }),
            OrderPaymentStatus::PaymentSuccess => {
                let order = order.1.clone();
                let status = &order.order_status;
                if status == &OrderStatus::Completed || status == &OrderStatus::Canceled {
                    Err(html! {<></>})
                } else {
                    if status == &OrderStatus::InDelivery {
                        Ok(html! {
                            <>
                                <h2 class="text-2xl font-bold">{"Order in Delivery!"}</h2>
                                <LiveOrderTracking order={order.clone()} commerce={commerce_profile} />
                            </>
                        })
                    } else {
                        if let Some(courier_note) = order.courier.as_ref().cloned() {
                            let driver_db = DriverProfileIdb::try_from(courier_note).unwrap();
                            let driver = driver_db.profile();
                            let pubkey = driver_db.pubkey();

                            Ok(html! {
                                <>
                                    <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                                        <p>{"Order ID: "}{order.order_id()[..12].to_string()}</p>
                                        <p>{"Order Status: "}{order.order_status}</p>
                                        <DriverDetailsComponent {pubkey} {driver} />
                                    </div>
                                </>
                            })
                        } else {
                            Ok(html! {
                                <>
                                    <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                                        <p>{"Order ID: "}{order.order_id()[..12].to_string()}</p>
                                        <p>{"Order Status: "}{order.order_status}</p>
                                    </div>
                                </>
                            })
                        }
                    }
                }
            }
            _ => Err(html! {<>
            </>}),
        }
    } else {
        Err(html! {<>{props.children.clone()}</>})
    };
    let onclick = {
        let order_ctx = order_ctx.clone();
        let keys = key_ctx.get_nostr_key().clone();
        let sender = relay_ctx.send_note.clone();
        Callback::from(move |_| {
            let keys = keys.clone().expect("Nostr keys not found");
            let order = order_ctx.order.clone().expect("Order not found").0;
            gloo::console::log!("Cancelling order: ", format!("{:?}", order));
            let update_req = OrderUpdateRequest {
                order,
                status_update: OrderStatus::Canceled,
            };
            let signed_req = update_req
                .sign_update(&keys, NOSTR_KIND_CONSUMER_CANCEL)
                .expect("Failed to sign order update");
            sender.emit(signed_req);
        })
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
                    <button {onclick} class="absolute top-4 right-4">
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
    pub exchange_rate: f64,
}

#[function_component(OrderInvoiceComponent)]
pub fn order_invoice_details(props: &OrderInvoiceComponentProps) -> Html {
    let OrderInvoiceComponentProps {
        invoice,
        exchange_rate,
    } = props.clone();
    let invoice_pr = invoice.payment_request();
    let sat_amount = invoice.sat_amount();
    let srd_amount = sat_amount as f64 / 100_000_000.0 * exchange_rate;
    let onclick_copy = {
        let pr = invoice_pr.clone();
        Callback::from(move |_| {
            let _ = clipboard_copy(&pr);
        })
    };
    html! {
        <div class="flex flex-col gap-4 items-center text-center">
            <h2 class="text-2xl font-bold">{"Invoice Details"}</h2>
            <p>{"Exchange Rate: "}{format!("1 BTC = SRD {:.2}", exchange_rate)}</p>
            <p>{"Amount: "}{format!("{} sats ~ SRD {:.2}", sat_amount, srd_amount)}</p>
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

#[derive(Clone, PartialEq, Properties)]
pub struct LiveOrderTrackingProps {
    pub order: OrderInvoiceState,
    pub commerce: CommerceProfile,
}

#[function_component(LiveOrderTracking)]
pub fn live_order_tracking(props: &LiveOrderTrackingProps) -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let LiveOrderTrackingProps { order, commerce } = props;
    let order_req = order.get_order_request();
    let delivery_location: GeolocationCoordinates = order_req.address.coordinates().into();

    let pickup_location = commerce.geolocation();
    let driver_location = use_state(|| None::<GeolocationCoordinates>);
    let map_handle = use_state(|| None::<LeafletMap>);
    let marker_handle = use_state(|| None::<Marker>);
    let location_icon_options = IconOptions {
        icon_url: "/public/assets/img/rider2.png".to_string(),
        icon_size: Some(vec![32, 32]),
        icon_anchor: Some(vec![16, 32]),
    };

    let driver_marker = marker_handle.clone();
    let map = map_handle.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            gloo::console::log!("Received note kind:", note.kind);

            // Only process driver state notes
            if note.kind == NOSTR_KIND_DRIVER_STATE {
                if let Some(keys) = key_ctx.get_nostr_key() {
                    match keys.decrypt_nip_04_content(note) {
                        Ok(decrypted) => {
                            gloo::console::log!("Decrypted content:", &decrypted);

                            // Parse directly as DriverStateUpdate
                            match serde_json::from_str::<DriverStateUpdate>(&decrypted) {
                                Ok(state_update) => {
                                    let coords = state_update.get_location();
                                    gloo::console::log!(
                                        "Got driver coordinates:",
                                        format!("{:?}", coords)
                                    );

                                    driver_location.set(Some(coords.clone()));

                                    if let (Some(marker), Some(_map)) =
                                        (driver_marker.as_ref(), map.as_ref())
                                    {
                                        let latlng: LatLng = coords.clone().into();
                                        if let Ok(js_value) = serde_wasm_bindgen::to_value(&latlng)
                                        {
                                            marker.set_lat_lng(&js_value);
                                            gloo::console::log!(
                                                "Updated marker position:",
                                                format!("{:?}", coords)
                                            );
                                        }
                                    } else {
                                        gloo::console::warn!("Marker or map not initialized");
                                    }
                                }
                                Err(e) => {
                                    gloo::console::error!(
                                        "Failed to parse driver state:",
                                        e.to_string()
                                    );
                                    gloo::console::log!("Raw decrypted content:", &decrypted);
                                }
                            }
                        }
                        Err(e) => gloo::console::error!("Failed to decrypt note:", e.to_string()),
                    }
                }
            }
        }
        || {}
    });

    let map_options = LeafletMapOptions {
        zoom: 13,
        zoom_control: true,
        scroll_wheel_zoom: true,
        double_click_zoom: true,
        dragging: true,
        min_zoom: Some(3),
        max_zoom: Some(18),
        ..Default::default()
    };
    html! {
        <div class="w-full h-96">
            <LeafletComponent
                map_id="tracking-map"
                {map_options}
                location_icon_options={Some(location_icon_options)}
                markers={vec![]}
                on_map_created={Callback::from({
                    let map = map_handle.clone();
                    move |map_instance: LeafletMap| {
                        gloo::console::log!("Map created");
                        let pickup_icon = IconOptions {
                            icon_url: "/public/assets/img/pay_pickup.png".to_string(),
                            icon_size: Some(vec![32, 32]),
                            icon_anchor: Some(vec![16, 16]),
                        };
                        let _ = map_instance.add_marker_with_icon(&pickup_location, pickup_icon);

                        let delivery_icon = IconOptions {
                            icon_url: "/public/assets/img/my_marker.png".to_string(),
                            icon_size: Some(vec![32, 32]),
                            icon_anchor: Some(vec![16, 16]),
                        };
                        let _ = map_instance.add_marker_with_icon(&delivery_location, delivery_icon);

                        let bounds = vec![
                            vec![pickup_location.latitude, pickup_location.longitude],
                            vec![delivery_location.latitude, delivery_location.longitude],
                        ];
                        let js_value_bounds = serde_wasm_bindgen::to_value(&bounds).unwrap();
                        let _ = map_instance.fitBounds(&js_value_bounds);

                        map.set(Some(map_instance));
                    }
                })}
                on_marker_created={Callback::from({
                    let marker = marker_handle.clone();
                    move |m: Marker| {
                        gloo::console::log!("Driver marker created");
                        marker.set(Some(m))
                    }
                })}
                class="w-full h-full rounded-lg shadow-lg"
            />
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
