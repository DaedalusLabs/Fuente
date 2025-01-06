use crate::{
    contexts::{CommerceDataStore, DriverDataAction, DriverDataStore, OrderHubAction, OrderHubStore},
    router::DriverRoute,
};
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{AppLink, LoadingScreen, OrderList, OrderPickup, OrderPickupModal},
    models::{
        DriverProfile, DriverProfileIdb, OrderInvoiceState, OrderStatus, OrderUpdateRequest, DRIVER_HUB_PUB_KEY, NOSTR_KIND_COURIER_UPDATE
    },
};
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
            <div class="flex flex-col flex-1 gap-8">
                <h2 class="text-3xl max-w-1/2 font-mplus text-fuente-dark px-4">{"No orders yet!"}</h2>
            </div>
        };
    };
    if let Some((order, order_note)) = order_ctx.get_live_order() {
        return html! {
            <LiveOrderDetails {order} {order_note} />
        };
    }
    html! {
        <main class="container mx-auto mt-10">
            <div class="flex justify-between items-center">
                <h1 class="text-fuente text-4xl pb-10 lg:pb-0 text-center lg:text-left lg:text-6xl font-bold tracking-tighter">
                    {&translations["orders_heading"]}
                </h1>
                <AppLink<DriverRoute> 
                    route={DriverRoute::History}
                    class="border-2 border-fuente rounded-full py-3 px-10 text-center text-xl text-fuente font-semibold"
                    selected_class="">
                    <span>{&translations["orders_historic"]}</span>
                </AppLink<DriverRoute>>
            </div>

            <div class="flex justify-center gap-10 mt-10 min-h-96">
                <div class="grid grid-cols-1 gap-2 lg:w-1/2 xl:w-[40%] 2xl:w-[30%]">
                    <section>
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
                    </section>
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
    let sender = relay_ctx.send_note.clone();
    let OrderPickupProps { order, order_note } = props;
    let order_req = order.get_order_request();
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

    html! {
        <OrderPickupModal
            order={order.clone()}
            commerce_profile={commerce.profile().clone()}
            on_order_click={onclick}
            />
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
                                let coords = state_update.get_location();
                                gloo::console::log!(
                                    "Created state update with coords:",
                                    format!("{:?}", coords)
                                );

                                // Send to driver hub
                                if let Ok(final_note) = state_update
                                    .to_encrypted_note(&keys, DRIVER_HUB_PUB_KEY.to_string())
                                {
                                    gloo::console::log!("Sending to hub, kind:", final_note.kind);
                                    gloo::console::log!(
                                        "Hub encrypted content length:",
                                        final_note.content.len()
                                    );
                                    sender.emit(final_note);
                                }

                                // Send to customer
                                if let Ok(final_note) =
                                    state_update.to_encrypted_note(&keys, customer_pubkey)
                                {
                                    gloo::console::log!(
                                        "Sending to customer, kind:",
                                        final_note.kind
                                    );
                                    gloo::console::log!(
                                        "Customer encrypted content length:",
                                        final_note.content.len()
                                    );
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

