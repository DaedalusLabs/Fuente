use crate::{
    contexts::{
        commerce_data::CommerceDataStore,
        driver_data::DriverDataStore,
        live_order::{OrderHubAction, OrderHubStore},
    },
    router::DriverRoute,
};
use fuente::{
    browser_api::{GeolocationCoordinates, GeolocationPosition},
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    mass::atoms::{
        forms::AppLink,
        layouts::LoadingScreen,
        svgs::{HistoryIcon, HomeIcon, MenuBarsIcon, SpinnerIcon, UserBadgeIcon},
    },
    models::{
        consumer_profile::ConsumerProfile,
        orders::{OrderInvoiceState, OrderStatus},
    },
    widgets::leaflet::{IconOptions, L},
};
use gloo::utils::format::JsValueSerdeExt;
use wasm_bindgen::JsValue;
use yew::{platform::spawn_local, prelude::*};

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Failed to get commerce context");
    let order_ctx = use_context::<OrderHubStore>().expect("Failed to get order context");
    if !commerce_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    let orders = order_ctx.get_orders();

    if orders.is_empty() {
        return html! {
            <div class="flex flex-col flex-1 gap-8">
                <h2 class="text-3xl max-w-1/2 font-mplus text-fuente-dark px-4">{"No orders yet!"}</h2>
            </div>
        };
    };
    if let Some(live_order) = order_ctx.get_live_order() {
        return html! {
            <div class="flex flex-col flex-1 gap-8 overflow-y-auto">
                <h2>{"Live Order!"}</h2>
                <LiveOrderDetails order={live_order} />
            </div>
        };
    }
    html! {
            <div class="flex flex-col flex-1 gap-8 overflow-y-auto">
            {{orders.iter().map(|order| {
                html! {
                    <OrderPickupDetails order={order.clone()} />
                }
            }).collect::<Html>()}}
            </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct OrderPickupProps {
    pub order: OrderInvoiceState,
}

#[function_component(LiveOrderDetails)]
pub fn live_order_details(props: &OrderPickupProps) -> Html {
    let commerce_ctx = use_context::<CommerceDataStore>().expect("Failed to get commerce context");
    let key_ctx = use_context::<NostrIdStore>().expect("Failed to get key context");
    let keys = key_ctx.get_nostr_key().expect("Failed to get keys");
    let relay_ctx = use_context::<NostrProps>().expect("Failed to get order context");
    let live_order_ctx = use_context::<OrderHubStore>().expect("Failed to get live order context");
    let sender = relay_ctx.send_note.clone();
    let OrderPickupProps { order } = props;
    let order_req = order.get_order_request();
    let commerce = commerce_ctx
        .find_commerce(&order_req.commerce)
        .expect("Failed to find commerce");
    let commerce_address = commerce.profile().geolocation();
    let profile = order_req.profile;
    let address: GeolocationCoordinates = order_req.address.coordinates().into();

    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);
    let location_state_clone = location_state.clone();
    use_effect_with((), move |_| {
        let state = location_state_clone.clone();
        spawn_local(async move {
            if let Ok(position) = GeolocationPosition::locate().await {
                state.set(Some(position.coords));
            }
        });
        move || {}
    });
    if location_state.is_none() {
        return html! {<SpinnerIcon class="w-8 h-8" />};
    }
    let location_state = location_state.as_ref().clone().unwrap();
    let onclick = {
        let order_clone = order.clone();
        let keys_clone = keys.clone();
        let order_ctx = live_order_ctx.clone();
        Callback::from(move |_| {
            let mut order_clone = order_clone.clone();
            match order_clone.get_order_status() {
                OrderStatus::ReadyForDelivery => {
                    order_clone.update_order_status(OrderStatus::InDelivery);
                }
                OrderStatus::InDelivery => {
                    order_clone.update_order_status(OrderStatus::Completed);
                    order_ctx.dispatch(OrderHubAction::OrderCompleted(order_clone.id()));
                }
                _ => {}
            }
            let signed_order = order_clone
                .sign_server_request(&keys_clone)
                .expect("Failed to sign order");
            sender.emit(signed_order);
        })
    };

    html! {
        <div class="flex flex-col flex-1 gap-2 shadow-xl p-2 w-fit h-fit">
            <div class="flex flex-row">
                <p class="text-2xl font-mplus text-fuente-dark">
                    {format!("Order #{} - for {}", order.id()[..12].to_string(), profile.nickname())}
                </p>
            </div>
            <div class="flex flex-row">
                <OrderPickupMapPreview
                    order_id={order.id()}
                    commerce_location={commerce_address}
                    consumer_location={address}
                    own_location={location_state.clone()}
                />
                <div class="flex flex-1 flex-col gap-4 items-center justify-center">
                    <button {onclick} class="bg-fuente text-white rounded-3xl p-2 w-1/2">
                        {{match order.get_order_status() {
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
    let driver_ctx = use_context::<DriverDataStore>().expect("Failed to get driver context");
    let driver_profile = driver_ctx
        .get_profile_note()
        .expect("Failed to get driver profile");
    let sender = relay_ctx.send_note.clone();
    let OrderPickupProps { order } = props;
    let order_req = order.get_order_request();
    let commerce = commerce_ctx
        .find_commerce(&order_req.commerce)
        .expect("Failed to find commerce");
    let commerce_address = commerce.profile().geolocation();
    let profile = order_req.profile;
    let address: GeolocationCoordinates = order_req.address.coordinates().into();

    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);
    let location_state_clone = location_state.clone();
    use_effect_with((), move |_| {
        let state = location_state_clone.clone();
        spawn_local(async move {
            if let Ok(position) = GeolocationPosition::locate().await {
                state.set(Some(position.coords));
            }
        });
        move || {}
    });
    if location_state.is_none() {
        return html! {<SpinnerIcon class="w-8 h-8" />};
    }
    let location_state = location_state.as_ref().clone().unwrap();
    let onclick = {
        let order_clone = order.clone();
        let keys_clone = keys.clone();
        let driver_note = driver_profile.clone();
        Callback::from(move |_| {
            let mut order_clone = order_clone.clone();
            order_clone.update_courier(driver_note.clone());
            let signed_order = order_clone
                .sign_server_request(&keys_clone)
                .expect("Failed to sign order");
            sender.emit(signed_order);
        })
    };

    html! {
        <div class="flex flex-col flex-1 gap-2 shadow-xl p-2 w-fit h-fit">
            <div class="flex flex-row">
                <p class="text-2xl font-mplus text-fuente-dark">
                    {format!("Order #{} - for {}", order.id()[..12].to_string(), profile.nickname())}
                </p>
            </div>
            <div class="flex flex-row">
                <OrderPickupMapPreview
                    order_id={order.id()}
                    commerce_location={commerce_address}
                    consumer_location={address}
                    own_location={location_state.clone()}
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
    pub own_location: GeolocationCoordinates,
}
#[function_component(OrderPickupMapPreview)]
pub fn order_pickup_map_preview(props: &OrderPickupMapPreviewProps) -> Html {
    let OrderPickupMapPreviewProps {
        order_id,
        commerce_location,
        consumer_location,
        own_location,
    } = props;
    let map_state: UseStateHandle<Option<fuente::widgets::leaflet::LeafletMap>> =
        use_state(|| None);
    let map_id = format!("order-map-{}", order_id);
    let map_id_clone = map_id.clone();
    let state_clone = map_state.clone();
    let commerce_loc = commerce_location.clone();
    let consumer_loc = consumer_location.clone();
    let position = own_location.clone();
    use_effect_with((), move |_| {
        gloo::console::log!("Rendering map");
        let id = map_id_clone.clone();
        let state = state_clone.clone();
        let commerce_location = commerce_loc.clone();
        let consumer_location = consumer_loc.clone();
        gloo::console::log!("Rendering map on position:", format!("{:?}", position));
        gloo::console::log!("Rendering map on element: ", id.clone());
        let map = L::render_map(&id, &position).expect("Failed to render map");
        let user_marker_options = IconOptions {
            icon_url: "public/assets/img/marker.png".to_string(),
            icon_size: None,
            icon_anchor: None,
        };
        map.add_marker_with_icon(&commerce_location, user_marker_options)
            .expect("Failed to add marker");
        let commerce_marker_options = IconOptions {
            icon_url: "public/assets/img/my_marker.png".to_string(),
            icon_size: None,
            icon_anchor: None,
        };
        map.add_marker_with_icon(&commerce_location, commerce_marker_options)
            .expect("Failed to add marker");
        let rider_marker_options = IconOptions {
            icon_url: "public/assets/img/rider2.png".to_string(),
            icon_size: None,
            icon_anchor: None,
        };
        map.add_marker_with_icon(&commerce_location, rider_marker_options)
            .expect("Failed to add marker");
        let js_array = vec![
            vec![commerce_location.latitude, commerce_location.longitude],
            vec![consumer_location.latitude, consumer_location.longitude],
            vec![position.latitude, position.longitude],
        ];
        let js_value = JsValue::from_serde(&js_array).expect("Failed to convert to JsValue");
        map.fit_bounds(js_value);
        state.set(Some(map));
        move || {}
    });
    html! {
        <div id={map_id}
            class="w-64 h-64 max-w-64 max-h-64 min-w-64 min-h-64
            border-2 border-fuente rounded-3xl shadow-xl"
            >
        </div>
    }
}
