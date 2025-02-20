use crate::{
    contexts::{CommerceDataStore, DriverDataStore, OrderHubAction, OrderHubStore},
    router::DriverRoute,
};
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{AppLink, LoadingScreen, OrderList, OrderPickup, OrderPickupModal},
    models::{
        OrderInvoiceState, OrderStatus, OrderUpdateRequest, DRIVER_HUB_PUB_KEY,
        NOSTR_KIND_COURIER_UPDATE,
    },
};
use lucide_yew::ScrollText;
use nostr_minions::{
    browser_api::{GeolocationCoordinates, HtmlForm},
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use yew::prelude::*;

use fuente::models::DriverStateUpdate;
use gloo::timers::callback::Interval;
use yew::platform::spawn_local;

fn respond_to_order(
    nostr_keys: NostrKeypair,
    send_note: Callback<NostrNote>,
    order: NostrNote,
    update_kind: u32,
) -> Callback<SubmitEvent> {
    Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Could not get form");
        let status_update_str = form
            .select_value("order_status")
            .expect("Could not parse order status");
        let status_update =
            OrderStatus::try_from(status_update_str).expect("Could not parse order status");
        let new_request = OrderUpdateRequest {
            order: order.clone(),
            status_update,
        };
        let signed_req = new_request
            .sign_update(&nostr_keys, update_kind)
            .expect("Could not sign order");
        send_note.emit(signed_req);
    })
}
#[function_component(HomePage)]
pub fn home_page() -> Html {
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("Failed to get language context");
    let translations = language_ctx.translations();
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Failed to get commerce context");
    let order_ctx = use_context::<OrderHubStore>().expect("Failed to get order context");
    let send_note = use_context::<NostrProps>().expect("Nostr context not found");
    let update_kind = NOSTR_KIND_COURIER_UPDATE;
    let key_ctx = use_context::<NostrIdStore>().expect("Nostr context not found");
    let nostr_keys = key_ctx.get_nostr_key().expect("Nostr key not found");
    if !commerce_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    let orders = order_ctx.live_orders();

    if orders.is_empty() && order_ctx.get_live_order().is_none() {
        return html! {
            <div class="min-h-screen flex items-center justify-center flex-1">
                <h2 class="text-3xl max-w-1/2 font-mplus text-fuente-dark px-4 text-center">{"No orders yet!"}</h2>
            </div>
        };
    };
    if let Some((order, order_note)) = order_ctx.get_live_order() {
        return html! {
            <LiveOrderDetails {order} {order_note} />
        };
    }
    html! {
        <main class="flex-1 overflow-hidden">
            <div class="flex flex-col h-full">
                <div class="flex flex-row justify-between items-center p-4 lg:p-10">
                    <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-0 lg:text-6xl tracking-tighter font-bold">
                        {&translations["orders_heading"]}
                    </h1>
                    <AppLink<DriverRoute>
                        route={DriverRoute::History}
                        class="block lg:hidden flex items-center bg-fuente-buttons p-2 rounded-xl"
                        selected_class="">
                            <ScrollText class="w-6 h-6 text-fuente" />
                    </AppLink<DriverRoute>>
                    <AppLink<DriverRoute>
                        route={DriverRoute::History}
                        class="lg:block hidden flex items-center bg-fuente-buttons px-6 py-3 rounded-full text-fuente-forms space-x-2 font-bold text-sm md:text-md lg:text-lg"
                        selected_class="">
                        <span>{&translations["orders_historic"]}</span>
                    </AppLink<DriverRoute>>
                </div>

                <div class="flex flex-col flex-1 overflow-hidden">
                    <div class="flex-1 overflow-hidden mt-4 px-4 flex justify-center">
                        <OrderList title={OrderStatus::ReadyForDelivery}>
                            {orders.iter().filter(|o| o.0.order_status == OrderStatus::ReadyForDelivery && o.0.courier.is_none()
                                ).map(|order| {
                                    let commerce = commerce_ctx
                                        .find_commerce(&order.0.get_commerce_pubkey())
                                        .expect("Failed to find commerce");
                                let on_click = {
                                    respond_to_order(nostr_keys.clone(), send_note.send_note.clone(), order.1.clone(), update_kind)
                                };
                                html! {
                                    <OrderPickup order={order.0.clone()} on_click={on_click} commerce_profile={commerce.profile().clone()} />
                                }
                            }).collect::<Html>()}
                        </OrderList>
                    </div>
                </div>
            </div>
        </main>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct OrderPickupProps {
    pub order: OrderInvoiceState,
    pub order_note: NostrNote,
}

#[function_component(LiveOrderDetails)]
pub fn live_order_details(props: &OrderPickupProps) -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Failed to get commerce context");
    let key_ctx = use_context::<NostrIdStore>().expect("Failed to get key context");
    let keys = key_ctx.get_nostr_key().expect("Failed to get keys");
    let relay_ctx = use_context::<NostrProps>().expect("Failed to get order context");
    let live_order_ctx = use_context::<OrderHubStore>().expect("Failed to get live order context");
    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);
    let sender = relay_ctx.send_note.clone();
    let OrderPickupProps { order, order_note } = props;
    let order_req = order.get_order_request();
    let order_coordinates = order_req.address.coordinates();
    let order_status = order.order_status.clone();
    let commerce = commerce_ctx
        .find_commerce(&order_req.commerce)
        .expect("Failed to find commerce");
    let onclick = {
        let order_clone = order.clone();
        let keys_clone = keys.clone();
        let order_ctx = live_order_ctx.clone();
        let order_note = order_note.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let new_status = match order_clone.order_status {
                OrderStatus::ReadyForDelivery => OrderStatus::InDelivery,
                OrderStatus::InDelivery => {
                    order_ctx.dispatch(OrderHubAction::OrderCompleted(order_clone.order_id()));
                    OrderStatus::Completed
                }
                _ => {
                    return;
                }
            };
            let update_req = OrderUpdateRequest {
                order: order_note.clone(),
                status_update: new_status,
            };
            let signed_order = update_req
                .sign_update(&keys_clone, NOSTR_KIND_COURIER_UPDATE)
                .expect("Failed to sign order");
            sender.emit(signed_order);
        })
    };
    if order_coordinates.latitude.is_empty() || order_coordinates.longitude.is_empty() {
        return html! {
            <div class="flex items-center justify-center h-full">
                <p class="text-red-500">{"Invalid order location coordinates"}</p>
            </div>
        };
    }
    html! {
        <>
        <OrderPickupModal
            order={order.clone()}
            commerce_profile={commerce.profile().clone()}
            on_order_click={onclick}
            />
            {if order_status == OrderStatus::InDelivery {
                html! {
                    <LocationTracker order_id={order.order_id()} location_state={location_state.clone()} />
                }
            } else {
                html! {
                }
            }}
        </>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct LocationTrackerProps {
    pub order_id: String,
    pub location_state: UseStateHandle<Option<GeolocationCoordinates>>,
}

#[function_component(LocationTracker)]
pub fn location_tracker(props: &LocationTrackerProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let driver_ctx = use_context::<DriverDataStore>().expect("DriverDataStore not found");
    let order_ctx = use_context::<OrderHubStore>().expect("OrderHub context not found");

    // Clone the data we need before the effect
    let order_id = props.order_id.clone();

    use_effect_with(order_id.clone(), move |_| {
        let keys = key_ctx.get_nostr_key().expect("No keys found");
        let sender = relay_ctx.send_note.clone();
        let driver_profile = driver_ctx
            .get_profile_note()
            .expect("No driver profile found");

        let interval = {
            Interval::new(5000, move || {
                let keys = keys.clone();
                let sender = sender.clone();
                let driver_profile = driver_profile.clone();
                let order_ctx = order_ctx.clone();

                spawn_local(async move {
                    if let Some((order_state, _order_note)) = order_ctx.get_live_order() {
                        // Get the customer's pubkey from the original order
                        let customer_pubkey = order_state.order.pubkey.clone();

                        match DriverStateUpdate::new(driver_profile.clone()).await {
                            Ok(state_update) => {
                                // Send to driver hub
                                if let Ok(final_note) = state_update
                                    .to_encrypted_note(&keys, DRIVER_HUB_PUB_KEY.to_string())
                                {
                                    sender.emit(final_note);
                                }

                                // Send to customer
                                if let Ok(final_note) =
                                    state_update.to_encrypted_note(&keys, customer_pubkey)
                                {
                                    sender.emit(final_note);
                                }
                            }
                            Err(e) => gloo::console::error!(
                                "Failed to create state update:",
                                e.to_string()
                            ),
                        }
                    }
                });
            })
        };

        move || {
            interval.cancel();
        }
    });

    html! { <></> }
}
