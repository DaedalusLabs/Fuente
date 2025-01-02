use bright_lightning::LndHodlInvoice;
use fuente::{
    contexts::AdminConfigsStore,
    mass::{DriverDetailsComponent, OrderRequestDetailsComponent, SpinnerIcon},
    models::{
        CommerceProfile, DriverProfileIdb, DriverStateUpdate, OrderInvoiceState, OrderPaymentStatus, OrderStatus, OrderUpdateRequest, SatisfactionRecord, NOSTR_KIND_CONSUMER_CANCEL, NOSTR_KIND_DRIVER_STATE, TEST_PUB_KEY
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

use crate::contexts::{CommerceDataExt, CommerceDataStore, LiveOrderAction, LiveOrderStore};

#[function_component(LiveOrderCheck)]
pub fn live_order_check(props: &ChildrenProps) -> Html {
    let order_ctx = use_context::<LiveOrderStore>().expect("LiveOrderStore not found");
    let admin_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("Nostr context not found");
    let exchange_rate = admin_ctx.get_exchange_rate();
    let commerce_ctx = use_context::<CommerceDataStore>().expect("CommerceDataStore not found");

    let show_rating = use_state(|| false);
    let rating_shown = use_state(|| false);

    // Effect to handle order completion and show rating
    {
        let show_rating = show_rating.clone();
        use_effect_with(order_ctx.clone(), move |order_ctx| {
            gloo::console::log!("Order context changed");
            
            if let Some((_, order_state)) = &order_ctx.order {
                gloo::console::log!("Current order status:", format!("{:?}", order_state.order_status));
                gloo::console::log!("Current payment status:", format!("{:?}", order_state.payment_status));
                
                // Show rating only when order is Completed/Canceled AND payment is Success
                let should_show_rating = 
                    (order_state.order_status == OrderStatus::Completed || 
                    order_state.order_status == OrderStatus::Canceled) && 
                    order_state.payment_status == OrderPaymentStatus::PaymentSuccess;
                
                if should_show_rating && !*show_rating {
                    gloo::console::log!("Setting show_rating to true - Order completed/canceled with payment success");
                    show_rating.set(true);
                } else if !should_show_rating && *show_rating {
                    gloo::console::log!("Resetting show_rating - Conditions not met");
                    show_rating.set(false);
                }
            }
            || {}
        });
    }

    // Debug effect for show_rating changes
    {
        let show_rating_clone = show_rating.clone();
        use_effect_with(show_rating_clone, move |show_rating| {
            gloo::console::log!("show_rating state changed:", **show_rating);
            || {}
        });
    }

    if order_ctx.order.is_none() {
        gloo::console::log!("No order found");
        return html! {};
    }

    // Prepare rating prompt component
    let rating_prompt = if *show_rating {
        gloo::console::log!("Attempting to render rating prompt");
        if let Some((_, order_state)) = &order_ctx.order {
            gloo::console::log!("Have order data for rating prompt - creating component");
            let order_ctx_clone = order_ctx.clone();
            let order_id = order_state.order_id();
            
            html! {
                <div class="fixed inset-0 flex items-center justify-center" style="z-index: 9999;">
                    <div class="fixed inset-0 bg-black opacity-50"></div>
                    <div class="relative z-50">
                        <RatingPrompt 
                            order_id={order_state.order_id()}
                            commerce_id={order_state.get_commerce_pubkey()}
                            onclose={
                                let show_rating = show_rating.clone();
                                Callback::from(move |_| {
                                    gloo::console::log!("Rating prompt closing");
                                    show_rating.set(false);
                                    order_ctx_clone.dispatch(LiveOrderAction::CompleteOrder(order_id.clone()));
                                })
                            }
                        />
                    </div>
                </div>
            }
        } else {
            gloo::console::log!("No order data for rating prompt");
            html! {}
        }
    } else {
        html! {}
    };


    let inside_html = if let Some((order_note, order_state)) = &order_ctx.order {
        match order_state.payment_status {
            OrderPaymentStatus::PaymentPending => Ok(html! {
                <>
                    <h2 class="text-2xl font-bold">{"Order Received!"}</h2>
                    <OrderInvoiceComponent 
                        invoice={order_state.consumer_invoice.as_ref().cloned().unwrap()} 
                        {exchange_rate} 
                    />
                </>
            }),
            OrderPaymentStatus::PaymentReceived => Ok(html! {
                <>
                    <div class="bg-white p-8 rounded-lg">
                        <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                    </div>
                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                        <p>{"Order ID: "}{order_state.order_id()[..12].to_string()}</p>
                        <p>{"Waiting for confirmation..."}</p>
                    </div>
                </>
            }),
            OrderPaymentStatus::PaymentSuccess => {
                let status = &order_state.order_status;
                if status == &OrderStatus::Completed || status == &OrderStatus::Canceled {
                    Err(html! {<></>})
                } else {
                    if status == &OrderStatus::InDelivery {
                        let commerce = commerce_ctx
                            .find_commerce_by_id(&order_state.get_commerce_pubkey())
                            .expect("Failed to find commerce");
                        Ok(html! {
                            <>
                                <h2 class="text-2xl font-bold">{"Order in Delivery!"}</h2>
                                <LiveOrderTracking 
                                    order={order_state.clone()} 
                                    commerce={commerce} 
                                />
                            </>
                        })
                    } else {
                        if let Some(courier_note) = order_state.courier.as_ref().cloned() {
                            let driver_db = DriverProfileIdb::try_from(courier_note).unwrap();
                            let driver = driver_db.profile();
                            let pubkey = driver_db.pubkey();

                            Ok(html! {
                                <>
                                    <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                                        <p>{"Order ID: "}{order_state.order_id()[..12].to_string()}</p>
                                        <p>{"Order Status: "}{order_state.order_status.clone()}</p>
                                        <DriverDetailsComponent {pubkey} {driver} />
                                    </div>
                                </>
                            })
                        } else {
                            Ok(html! {
                                <>
                                    <h2 class="text-2xl font-bold">{"Order Paid!"}</h2>
                                    <div class="flex flex-col gap-4 text-wrap max-w-md">
                                        <p>{"Order ID: "}{order_state.order_id()[..12].to_string()}</p>
                                        <p>{"Order Status: "}{order_state.order_status.clone()}</p>
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

    let cancel_onclick = {
        let order_ctx = order_ctx.clone();
        Callback::from(move |_| {
            if let Some((order_note, order_state)) = &order_ctx.order {
                let keys = key_ctx.get_nostr_key().expect("No user keys found");
                let update_req = OrderUpdateRequest {
                    order: order_note.clone(),
                    status_update: OrderStatus::Canceled,
                };
                if let Ok(signed_req) = update_req.sign_update(&keys, NOSTR_KIND_CONSUMER_CANCEL) {
                    relay_ctx.send_note.emit(signed_req);
                }
            }
        })
    };

    match inside_html {
        Err(e) => {
            if *show_rating {
                html! {
                    <>
                        {rating_prompt}
                        {e}
                    </>
                }
            } else {
                e
            }
        },
        Ok(inside_html) => {
            let order = order_ctx
                .order
                .clone()
                .unwrap()
                .1
                .get_order_request()
                .products;
            
            gloo::console::log!("Final render - show_rating:", *show_rating);
            
            html! {
                <>
                    {rating_prompt}
                    <div class="relative h-full w-full flex items-center justify-center">
                        <div class="relative bg-white p-4 flex flex-col gap-4">
                            <OrderRequestDetailsComponent {order} />
                            {inside_html}
                            <SpinnerIcon class="absolute top-4 right-4 w-4 h-4 text-fuente" />
                        </div>
                        <button onclick={cancel_onclick} class="absolute top-4 right-4">
                            <Cross class="w-8 h-8 text-red-500" />
                        </button>
                    </div>
                </>
            }
        }
    }
}

use lucide_yew::{Copy, Cross};
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
        <div class="bg-zinc-100 p-4 rounded-2xl flex flex-col gap-3">
            <div class="flex justify-between flex-1">
                <h3 class="text-fuente text-xl font-bold">{"Total Amount"}</h3>
                <p class="text-gray-400 text-lg">{format!("{:.2} SRD", srd_amount)}</p>
            </div>
            <div class="flex justify-between">
                <h3 class="text-fuente text-xl font-bold">{"Bitcoin"}</h3>
                <p class="text-gray-400 text-lg">{format!("{:.8} BTC", sat_amount as f64 / 100_000_000.0)}</p>
            </div>
            <div class="flex justify-between">
                <p class="text-xs font-bold text-gray-500">
                    {"Tap the invoice to open in your wallet."}
                </p>
                <button
                    onclick={onclick_copy} >
                    <Copy class="text-xs font-bold text-gray-500"  />
                </button>
            </div>
            <div class="align-self-center justify-center w-full items-center flex">
                <BitcoinQrCode
                    id={"qr".to_string()} width={"200".to_string()} height={"200".to_string()}
                    lightning={invoice_pr.clone()} type_="svg" />
            </div>
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
pub struct RatingPromptProps {
    pub order_id: String,
    pub commerce_id: String, 
    pub onclose: Callback<()>,
}

#[function_component(RatingPrompt)]
pub fn rating_prompt(props: &RatingPromptProps) -> Html {
    let RatingPromptProps { order_id, commerce_id, onclose } = props.clone();
    let rating = use_state(|| 5); // Default 5 stars
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrIdStore found");
    let relay_ctx = use_context::<NostrProps>().expect("No RelayPool context found");

    let onsubmit = {
        let onclose = onclose.clone();
        let rating = rating.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(keys) = key_ctx.get_nostr_key() {
                let satisfaction = SatisfactionRecord {
                    order_id: order_id.clone(),
                    participant: fuente::models::OrderParticipant::Commerce,
                    satisfaction: rating.to_string(),
                    rater_pubkey: keys.public_key(),
                };
                
                gloo::console::log!("Sending satisfaction record:", format!("{:?}", satisfaction));
                
                let sender = relay_ctx.send_note.clone();
                let mut giftwrap = nostro2::notes::NostrNote {
                    kind: fuente::models::NOSTR_KIND_SATISFACTION_EVENT,
                    content: serde_json::to_string(&satisfaction).unwrap(),
                    pubkey: keys.public_key(),
                    ..Default::default()
                };
                
                // Sign the note first
                keys.sign_nostr_event(&mut giftwrap);
                
                // Now encrypt it
                match keys.sign_nip_04_encrypted(&mut giftwrap, TEST_PUB_KEY.to_string()) {
                    Ok(()) => {
                        gloo::console::log!("Successfully encrypted and sending satisfaction record");
                        sender.emit(giftwrap);
                        onclose.emit(());
                    },
                    Err(e) => {
                        gloo::console::error!("Failed to encrypt satisfaction record:", e.to_string());
                    }
                }
            }
        })
    };

    html! {
        <div class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50">
            <form class="bg-white p-8 rounded-xl shadow-2xl max-w-md w-full mx-4" {onsubmit}>
                <h3 class="text-2xl font-bold mb-6">{"How was your experience?"}</h3>
                
                <div class="flex justify-center gap-4 mb-8">
                    {(1..=5).map(|i| {
                        let rating_state = rating.clone();
                        html! {
                            <button 
                                type="button"
                                onclick={Callback::from(move |_| rating_state.set(i))}
                                class={if *rating >= i { "text-yellow-400" } else { "text-gray-300" }}
                            >
                                <svg viewBox="0 0 20 20" fill="currentColor" class="w-12 h-12">
                                    <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                                </svg>
                            </button>
                        }
                    }).collect::<Html>()}
                </div>

                <div class="flex justify-end gap-4">
                    <button 
                        type="button"
                        onclick={Callback::from(move |_| onclose.emit(()))}
                        class="px-6 py-2 border rounded-lg hover:bg-gray-50"
                    >
                        {"Skip"}
                    </button>
                    <button 
                        type="submit"
                        class="px-6 py-2 bg-fuente text-white rounded-lg hover:bg-fuente-dark"
                    >
                        {"Submit"}
                    </button>
                </div>
            </form>
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
        corners-square-color="#B40A2D"
        corners-dot-color={"#ECC81D"}
        corners-square-type={"extra-rounded"}
        dots-type={"classy-rounded"}
        dots-color={"#377E3F"}
    />
    }
}
