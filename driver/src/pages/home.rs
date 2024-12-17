use crate::contexts::{
    CommerceDataStore, {OrderHubAction, OrderHubStore},
};
use fuente::{
    mass::LoadingScreen,
    models::{OrderInvoiceState, OrderStatus, OrderUpdateRequest, NOSTR_KIND_COURIER_UPDATE},
};
use nostr_minions::{
    browser_api::GeolocationCoordinates,
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
    widgets::leaflet::{IconOptions, LeafletComponent, LeafletMap, Marker},
};
use nostro2::notes::NostrNote;
use yew::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Failed to get commerce context");
    let order_ctx = use_context::<OrderHubStore>().expect("Failed to get order context");
    if !commerce_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    let orders = order_ctx.get_orders();

    if orders.is_empty() && order_ctx.get_live_order().is_none() {
        return html! {
            <div class="flex flex-col flex-1 gap-8">
                <h2 class="text-3xl max-w-1/2 font-mplus text-fuente-dark px-4">{"No orders yet!"}</h2>
            </div>
        };
    };
    if let Some((order, order_note)) = order_ctx.get_live_order() {
        return html! {
            <div class="flex flex-col flex-1 gap-4 p-4 overflow-y-auto">
                <h2>{"Live Order!"}</h2>
                <LiveOrderDetails {order} {order_note} />
            </div>
        };
    }
    html! {
            <div class="flex flex-col flex-1 gap-4 p-4 overflow-y-auto">
            {{orders.iter().map(|(order, order_note)| {
                html! {
                    <OrderPickupDetails order={order.clone()} order_note={order_note.clone()}/>
                }
            }).collect::<Html>()}}
            </div>
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
    let commerce_address = commerce.profile().geolocation();
    let profile = order_req.profile;
    let address: GeolocationCoordinates = order_req.address.coordinates().into();

    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);
    let onclick = {
        let order_clone = order.clone();
        let keys_clone = keys.clone();
        let order_ctx = live_order_ctx.clone();
        let order_note = order_note.clone();
        Callback::from(move |_| {
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
        <div class="flex flex-col gap-2 shadow-xl p-4 w-full h-full rounded-lg">
            <div class="flex flex-row">
                <p class="text-2xl font-mplus text-fuente-dark">
                    {format!("Order #{} - for {}", order.order_id()[..12].to_string(), profile.nickname)}
                </p>
            </div>
            <div class="flex flex-row flex-1 ">
                <OrderPickupMapPreview
                    order_id={order.order_id()}
                    commerce_location={commerce_address}
                    consumer_location={address}
                    own_location={location_state.clone()}
                    classes={classes!["w-full", "h-full", "rounded-lg", "min-w-96", "min-h-96"]}
                />
                <div class="flex flex-col gap-4 p-4">
                    <button {onclick} class="w-fit bg-fuente text-white rounded-3xl p-2 w-1/2">
                        {{match order.order_status {
                            OrderStatus::ReadyForDelivery => "Picked up Package",
                            OrderStatus::InDelivery => "Delivered Package",
                            _ => "No Action"
                        }}}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[function_component(OrderPickupDetails)]
pub fn order_pickup_details(props: &OrderPickupProps) -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Failed to get commerce context");
    let key_ctx = use_context::<NostrIdStore>().expect("Failed to get key context");
    let keys = key_ctx.get_nostr_key().expect("Failed to get keys");
    let relay_ctx = use_context::<NostrProps>().expect("Failed to get order context");
    let sender = relay_ctx.send_note.clone();
    let OrderPickupProps { order, order_note } = props;
    let order_req = order.get_order_request();
    let commerce = commerce_ctx
        .find_commerce(&order_req.commerce)
        .expect("Failed to find commerce");
    let commerce_address = commerce.profile().geolocation();
    let profile = order_req.profile;
    let address: GeolocationCoordinates = order_req.address.coordinates().into();

    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);
    let onclick = {
        let order_clone = order.clone();
        let keys_clone = keys.clone();
        let order_note = order_note.clone();
        Callback::from(move |_| {
            if order_clone.courier.is_none() {
                let update_req = OrderUpdateRequest {
                    order: order_note.clone(),
                    status_update: OrderStatus::ReadyForDelivery,
                };
                let signed_order = update_req
                    .sign_update(&keys_clone, NOSTR_KIND_COURIER_UPDATE)
                    .expect("Failed to sign order");
                sender.emit(signed_order);
                return;
            }
            let new_status = match order_clone.order_status {
                OrderStatus::ReadyForDelivery => OrderStatus::InDelivery,
                OrderStatus::InDelivery => OrderStatus::Completed,
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
        <div class="flex flex-col gap-2 shadow-xl p-2 w-fit h-fit">
            <div class="flex flex-row">
                <p class="text-2xl font-mplus text-fuente-dark">
                    {format!("Order #{} - for {}", order.order_id()[..12].to_string(), profile.nickname)}
                </p>
            </div>
            <div class="flex flex-row">
                <OrderPickupMapPreview
                    order_id={order.order_id()}
                    commerce_location={commerce_address}
                    consumer_location={address}
                    own_location={location_state.clone()}
                    classes={classes!["rounded-lg", "min-w-64", "min-h-64"]}
                />
                <div class="flex flex-1 flex-col gap-4 items-center justify-center">
                    <button {onclick} class="bg-fuente text-white rounded-3xl p-2 w-1/2">
                        {"Accept Order"}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct OrderPickupMapPreviewProps {
    pub order_id: String,
    pub commerce_location: GeolocationCoordinates,
    pub consumer_location: GeolocationCoordinates,
    pub own_location: UseStateHandle<Option<GeolocationCoordinates>>,
    pub classes: Classes,
}
#[function_component(OrderPickupMapPreview)]
pub fn order_pickup_map_preview(props: &OrderPickupMapPreviewProps) -> Html {
    let OrderPickupMapPreviewProps {
        order_id,
        commerce_location,
        consumer_location,
        own_location,
        classes,
    } = props.clone();
    let map_state: UseStateHandle<Option<LeafletMap>> = use_state(|| None);
    let markers: UseStateHandle<Vec<(f64, f64)>> = use_state(|| vec![]);
    let map_id = format!("order-map-{}", order_id);
    let own_marker_state = use_state(|| None::<Marker>);
    use_effect_with(map_state.clone(), move |map_state| {
        if let Some(map) = map_state.as_ref() {
            let commerce_icon = IconOptions {
                icon_url: "/public/assets/img/pay_pickup.png".to_string(),
                icon_size: Some(vec![32, 32]),
                icon_anchor: Some(vec![16, 16]),
            };
            let _ = map.add_marker_with_icon(&commerce_location, commerce_icon);
            let consumer_icon = IconOptions {
                icon_url: "/public/assets/img/my_marker.png".to_string(),
                icon_size: Some(vec![32, 32]),
                icon_anchor: Some(vec![16, 16]),
            };
            let _ = map.add_marker_with_icon(&consumer_location, consumer_icon);
            let bounds = vec![
                vec![commerce_location.latitude, commerce_location.longitude],
                vec![consumer_location.latitude, consumer_location.longitude],
            ];
            let js_value_bounds = serde_wasm_bindgen::to_value(&bounds).unwrap();
            let _ = map.fitBounds(&js_value_bounds);
        }
        || {}
    });
    let location_icon_options = Some(IconOptions {
        icon_url: "/public/assets/img/rider2.png".to_string(),
        icon_size: Some(vec![32, 32]),
        icon_anchor: Some(vec![16, 16]),
    });
    html! {
        <LeafletComponent
            {map_id}
            {location_icon_options}
            markers={(*markers).clone()}
            on_location_changed={Callback::from({
                let location_state = own_location.clone();
                move |coords: GeolocationCoordinates| {
                    location_state.set(Some(coords));
                }
            })}
            on_map_created={Callback::from({
                let map = map_state.clone();
                move |map_instance: LeafletMap| map.set(Some(map_instance))
            })}
            on_marker_created={Callback::from({
                move |marker: Marker| {
                    own_marker_state.set(Some(marker));
                }
            })}
            class={classes}
        />
    }
}
