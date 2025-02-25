use nostr_minions::{
    browser_api::GeolocationCoordinates,
    widgets::leaflet::{IconOptions, LeafletComponent, LeafletMap, LeafletMapOptions, Marker},
};
use yew::prelude::*;

use crate::{
    contexts::LanguageConfigsStore,
    mass::{
        CommerceProfileAddressDetails, CommerceProfileDetails, CustomerAddressDetails,
        CustomerDetails,
    },
    models::{CommerceProfile, OrderInvoiceState, OrderStatus},
};
#[derive(Clone, PartialEq, Properties)]
pub struct OrderPickupModalProps {
    pub order: OrderInvoiceState,
    pub commerce_profile: CommerceProfile,
    pub on_order_click: Callback<SubmitEvent>,
}
#[function_component(OrderPickupModal)]
pub fn order_detail_modal(props: &OrderPickupModalProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("Language context not found");
    let location_state: UseStateHandle<Option<GeolocationCoordinates>> = use_state(|| None);
    let translations = language_ctx.translations();
    let OrderPickupModalProps {
        order,
        commerce_profile,
        on_order_click,
    } = props;
    let request = order.get_order_request();
    let customer_profile = &request.profile;
    let address: GeolocationCoordinates = request.address.coordinates().into();
    let commerce_address = commerce_profile.geolocation();
    let order_state = order.order_status.clone();
    html! {
       <main class="bg-white rounded-2xl py-5 px-4 sm:px-10 mx-auto max-w-full sm:max-w-xl h-[90vh] sm:h-[640px] overflow-y-auto">
           <div class="flex items-center justify-between border-b border-b-gray-400 pb-3">
               <div>
                   <p class="text-fuente-dark font-bold text-xl sm:text-2xl">{format!("#{}", &order.order_id()[..12])}</p>
                   <p class="text-gray-500 font-light text-base sm:text-lg">{&translations["store_order_modal_title"]}</p>
               </div>
               <button
                   class="border-2 border-gray-400 text-gray-400 bg-white rounded-2xl py-2 px-3 sm:py-3 sm:px-4 text-center font-semibold text-sm sm:text-base">{order.order_status.to_string()}</button>
           </div>

           <OrderPickupMapPreview
               order_id={order.order_id()}
               commerce_location={commerce_address}
               consumer_location={address}
               own_location={location_state.clone()}
               classes={classes!["rounded-lg", "min-w-64", "min-h-64", "h-48", "sm:h-64", "w-full", "p-2"]}
           />
           {match order_state {
               OrderStatus::ReadyForDelivery => html! {
                   <div class="grid grid-cols-1 gap-4">
                       <CommerceProfileDetails commerce_data={commerce_profile.clone()} />
                       <CommerceProfileAddressDetails commerce_data={commerce_profile.clone()} />
                       <CustomerDetails customer={customer_profile.clone()} />
                   </div>
               },
               OrderStatus::InDelivery => html! {
                   <div class="grid grid-cols-1 gap-4">
                       <CustomerDetails customer={customer_profile.clone()} />
                       <CustomerAddressDetails customer={request.address.clone()} />
                   </div>
               },
               _ => html! {<></>},
           }}
           <form onsubmit={on_order_click.clone()} class="mt-5">
           <select id="order_status" name="order_status" class="hidden">
               <option value={OrderStatus::ReadyForDelivery.to_string()}></option>
           </select>
           <input type="submit" value={translations["store_order_modal_button_submit"].clone()}
               class="bg-fuente-orange text-white text-center text-base sm:text-lg font-bold rounded-full w-full py-2 sm:py-3 mt-5" />
           </form>
       </main>
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
            {map_options}
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
            style="height: 100%; width: 100%; border-radius: 1rem; border: 2px solid #f0f0f0;"
        />
    }
}
