use crate::{
    browser_api::{GeolocationCoordinates, GeolocationPosition},
    mass::atoms::CardComponent,
    widgets::leaflet::{
        IconOptions, LatLng, LeafletMap, LeafletMapOptions, Marker, NominatimLookup, L,
    },
};
use gloo::timers::callback::Timeout;
use wasm_bindgen::{JsCast, JsValue};
use yew::{platform::spawn_local, prelude::*};

#[derive(Properties, Clone, PartialEq)]
pub struct LookupProps {
    pub lookup: NominatimLookup,
}

#[function_component(AddressLookupDetails)]
pub fn consumer_address_card(props: &LookupProps) -> Html {
    let LookupProps { lookup } = props;
    let mut split = lookup.display_name().split(",");
    let name = split.next().unwrap().to_string();
    let display_name = split.collect::<Vec<&str>>().join(",");
    html! {
        <div class="flex flex-row gap-4">
            <div class="min-w-16 min-h-16 h-16 w-16 bg-neutral-200 rounded-2xl"></div>
            <div class="flex flex-col gap-1 text-wrap shrink">
                <span class="text-sm font-bold">{&name}</span>
                <span class="text-xs text-neutral-400 overflow-hidden text-ellipsis">
                    {&display_name}
                </span>
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct NewAddressProps {
    pub map_handle: UseStateHandle<Option<LeafletMap>>,
    pub marker_handle: UseStateHandle<Option<Marker>>,
    pub coord_handle: UseStateHandle<Option<GeolocationCoordinates>>,
    pub nominatim_handle: UseStateHandle<Option<NominatimLookup>>,
    pub onclick: Callback<MouseEvent>,
}

#[function_component(NewAddressMenu)]
pub fn new_address_menu(props: &NewAddressProps) -> Html {
    let NewAddressProps { onclick, .. } = props;
    html! {
        <>
            <div class="w-full flex flex-col gap-2">
                <div class="flex flex-row justify-between items-center pr-4">
                    <h3 class="font-bold">{"Address Details"}</h3>
                    <button
                        {onclick}
                        class="bg-fuente text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl"
                        >{"Save"}
                    </button>
                </div>
                <AddressDetails ..props.clone()/>
            </div>
            <AddressSearch ..props.clone() />
            <AddressPickerMap ..props.clone()/>
        </>
    }
}

#[function_component(AddressDetails)]
pub fn address_details(props: &NewAddressProps) -> Html {
    if props.nominatim_handle.is_none() || props.coord_handle.is_none() {
        return html! {<div class="w-full h-full flex justify-center items-center">{"Loading..."}</div>};
    };
    let address = (*props.nominatim_handle).clone().unwrap();
    html! {
        <CardComponent>
            <AddressLookupDetails lookup={address} />
        </CardComponent>
    }
}

#[function_component(AddressSearch)]
pub fn address_search(props: &NewAddressProps) -> Html {
    let NewAddressProps {
        map_handle,
        marker_handle,
        coord_handle,
        nominatim_handle: handle,
        ..
    } = props;
    let search_results = use_state(|| vec![]);

    let result_handle = search_results.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        let query = e
            .target()
            .expect("No target found")
            .dyn_ref::<web_sys::HtmlInputElement>()
            .expect("No input found")
            .value();
        let result_handle = result_handle.clone();
        spawn_local(async move {
            match NominatimLookup::address(&query).await {
                Ok(addresses) => {
                    if !addresses.is_empty() {
                        result_handle.set(addresses);
                    }
                }
                Err(_) => {
                    result_handle.set(vec![]);
                }
            }
        });
    });
    let result_handle = search_results.clone();
    let onblur = Callback::from(move |_| {
        let result_handle = result_handle.clone();
        Timeout::new(210, move || {
            result_handle.set(vec![]);
        })
        .forget();
    });
    html! {
        <div class="w-full relative">
            <form class="flex w-full items-center space-x-2">
                <input
                    type="text"
                    class="w-full p-2 px-4 rounded-3xl shadow-xl bg-transparent placeholder:bg-transparent
                        focus:outline-none focus:ring-fuente focus:ring-2"
                    placeholder="Search for address"
                    {oninput}
                    {onblur}
                />
            </form>
            {if !search_results.is_empty() {
                html! {
                    <div class="absolute top-full left-0 right-0 mt-1 z-[9998]
                             flex flex-col gap-2 h-64 overflow-y-scroll p-2 overflow-x-hidden">
                        {for search_results.iter().map(|address| {
                            let address_clone = address.clone();
                            let address_handle = handle.clone();
                            let coordinate_handle = coord_handle.clone();
                            let map = (*map_handle).clone();
                            let marker = (*marker_handle).clone();
                            let onclick = Callback::from(move |_| {
                                let coordinates = GeolocationCoordinates {
                                    latitude: address_clone.lat_as_f64(),
                                    longitude: address_clone.long_as_f64(),
                                    accuracy: 0.0,
                                    altitude: None,
                                    altitude_accuracy: None,
                                    speed: None,
                                };
                                coordinate_handle.set(Some(coordinates.clone()));
                                address_handle.set(Some(address_clone.clone()));
                                if map.as_ref().is_some()  || marker.as_ref().is_some() {
                                    marker.as_ref().unwrap().set_lat_lng(&coordinates.clone().into());
                                    map.as_ref().unwrap().set_view(&coordinates.into(), 13);
                                }
                            });
                            let split = address.display_name().split(",");
                            let name = split.clone().next().unwrap().to_string();
                            let display_name = split.clone().skip(1).collect::<Vec<&str>>().join(",");

                            html! {
                                <button
                                    type="button"
                                    {onclick}
                                    class="flex flex-row gap-2 bg-neutral-50 shadow-xl p-4 rounded-xl hover:bg-neutral-100 z-[9999]">
                                    <div class="flex flex-col gap-1 text-wrap h-fit w-full items-start text-start truncate">
                                        <span class="text-xs font-bold overflow-hidden text-ellipsis whitespace-nowrap">
                                            {&name}
                                        </span>
                                        <span class="text-xs text-neutral-400 overflow-hidden text-ellipsis whitespace-nowrap truncate">
                                            {&display_name}
                                        </span>
                                    </div>
                                </button>
                            }
                        })}
                       </div>
                    }
            } else {
                html! {<></>}
            }}
        </div>
    }
}

pub fn start_new_address_picker_map(
    location: GeolocationCoordinates,
    map_handler: &UseStateHandle<Option<LeafletMap>>,
    marker_handler: &UseStateHandle<Option<Marker>>,
    geo_handler: &UseStateHandle<Option<GeolocationCoordinates>>,
    address_handler: &UseStateHandle<Option<NominatimLookup>>,
) -> Result<(), JsValue> {
    let map_options = LeafletMapOptions {
        double_click_zoom: false,
        center: Some(location.clone().into()),
        ..Default::default()
    };
    let map = L::render_map_with_options("map", map_options)?;
    map_handler.set(Some(map.clone()));
    let icon_options = IconOptions {
        icon_url: "./public/assets/img/my_marker.png".to_string(),
        icon_size: Some(vec![32, 32]),
        icon_anchor: Some(vec![16, 32]),
    };
    let marker = map.add_marker_with_icon(&location, icon_options)?;
    marker_handler.set(Some(marker.clone()));
    geo_handler.set(Some(location));

    let geo_handler_clone = geo_handler.clone();
    let address_handler_clone = address_handler.clone();
    let map_closure = move |e: MouseEvent| {
        let leaflet_event = LatLng::try_from(e).expect("Could not parse event");
        let coordinates: GeolocationCoordinates = leaflet_event.clone().into();
        geo_handler_clone.set(Some(coordinates.clone()));
        marker.set_lat_lng(
            &leaflet_event
                .try_into()
                .expect("Could not conver to Js value"),
        );
        let handle = address_handler_clone.clone();
        spawn_local(async move {
            if let Ok(address) = NominatimLookup::reverse(coordinates.clone()).await {
                handle.set(Some(address));
            }
        });
    };
    map.add_closure("dblclick", map_closure);

    Ok(())
}
#[function_component(AddressPickerMap)]
pub fn address_picker(props: &NewAddressProps) -> Html {
    let NewAddressProps {
        map_handle,
        marker_handle,
        coord_handle,
        nominatim_handle,
        ..
    } = props.clone();
    use_effect_with((), move |_| {
        let address_handle = nominatim_handle.clone();
        spawn_local(async move {
            if let Ok(position) = GeolocationPosition::locate().await {
                if let Err(e) = start_new_address_picker_map(
                    position.coords.clone(),
                    &map_handle,
                    &marker_handle,
                    &coord_handle,
                    &nominatim_handle,
                ) {
                    gloo::console::error!("Error starting map: ", e);
                }
                if let Ok(address) = NominatimLookup::reverse(position.coords).await {
                    address_handle.set(Some(address));
                }
            }
        });
        || {}
    });
    html! {
        <div id="map" class="w-full h-full border-2 border-fuente rounded-3xl shadow-xl"></div>
    }
}
