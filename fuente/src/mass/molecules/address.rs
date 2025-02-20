use gloo::timers::callback::Timeout;
use nostr_minions::{
    browser_api::GeolocationCoordinates,
    widgets::leaflet::{
        nominatim::NominatimLookup, IconOptions, LatLng, LeafletComponent, LeafletMap,
        LeafletMapOptions, Marker,
    },
};
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
            <div class="flex flex-col gap-1 text-wrap shrink">
                <span class="text-sm font-bold">{&name}</span>
                <span class="text-xs overflow-hidden text-ellipsis">
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

#[function_component(NewAddressForm)]
pub fn new_address_menu(props: &NewAddressProps) -> Html {
    let NewAddressProps { onclick, .. } = props;
    html! {
        <div class="mt-4 flex flex-col gap-8 items-center text-white">
            <AddressDetails ..props.clone()/>
            <AddressSearch ..props.clone() />
            <AddressPickerMap ..props.clone()/>
            <button
                {onclick}
                class="bg-fuente text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl self-start"
            >{"Save"}
        </button>
        </div>
    }
}

#[function_component(AddressDetails)]
pub fn address_details(props: &NewAddressProps) -> Html {
    if props.nominatim_handle.is_none() || props.coord_handle.is_none() {
        return html! {<div class="w-full h-full flex justify-center items-center">{"Loading..."}</div>};
    };
    let address = (*props.nominatim_handle).clone().unwrap();
    html! {
        <AddressLookupDetails lookup={address} />
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
            <div class="flex w-full items-center space-x-2">
                <input
                    type="text"
                    class="w-full p-2 px-4 rounded-3xl shadow-xl bg-transparent placeholder:bg-transparent text-white placeholder:text-white
                        focus:outline-none focus:ring-fuente focus:ring-2 border-white"
                    placeholder="Search for address"
                    {oninput}
                    {onblur}
                />
            </div>
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
                                    let latlng: LatLng = coordinates.into();
                                    let js_value: JsValue = latlng.try_into().unwrap();
                                    marker.as_ref().unwrap().set_lat_lng(&js_value);
                                    map.as_ref().unwrap().set_view(&js_value, 13);
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
                                        <span class="text-xs text-neutral-900 font-bold overflow-hidden text-ellipsis whitespace-nowrap">
                                            {&name}
                                        </span>
                                        <span class="text-xs text-neutral-700 overflow-hidden text-ellipsis whitespace-nowrap truncate">
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

#[function_component(AddressPickerMap)]
pub fn address_picker_v2(props: &NewAddressProps) -> Html {
    let map = props.map_handle.clone();
    let location_state = props.coord_handle.clone();
    let lookup_handle = props.nominatim_handle.clone();
    let map_options = LeafletMapOptions {
        double_click_zoom: false,
        zoom_control: true,
        scroll_wheel_zoom: true,
        zoom: 13,
        min_zoom: Some(3),
        max_zoom: Some(18),
        ..Default::default()
    };
    let location_icon_options = IconOptions {
        icon_url: "/public/assets/img/red-pin.svg".to_string(),
        icon_size: Some(vec![32, 32]),
        icon_anchor: Some(vec![16, 32]),
    };
    let geo_handler_clone = props.coord_handle.clone();
    let address_handler_clone = props.nominatim_handle.clone();
    let marker = props.marker_handle.clone();
    use_effect_with(map.clone(), move |map_handle| {
        if let Some(map) = map_handle.as_ref() {
            let map_closure = move |e: MouseEvent| {
                let leaflet_event = LatLng::try_from(e).expect("Could not parse event");
                let coordinates: GeolocationCoordinates = leaflet_event.clone().into();
                geo_handler_clone.set(Some(coordinates.clone()));
                if let Some(marker) = marker.as_ref() {
                    marker.set_lat_lng(
                        &leaflet_event
                            .try_into()
                            .expect("Could not conver to Js value"),
                    );
                }
                let handle = address_handler_clone.clone();
                spawn_local(async move {
                    if let Ok(address) = NominatimLookup::reverse(coordinates.clone()).await {
                        handle.set(Some(address));
                    }
                });
            };
            map.add_closure("dblclick", map_closure);
        }
        || {}
    });
    let marker_handle = props.marker_handle.clone();
    let markers = use_state(|| Vec::<(f64, f64)>::new());
    html! {
        <LeafletComponent
            map_id="map"
            {map_options}
            {location_icon_options}
            markers={(*markers).clone()}
            on_location_changed={Callback::from({
                let location_state = location_state.clone();
                move |coords: GeolocationCoordinates| {
                    location_state.set(Some(coords));
                }
            })}
            on_map_created={Callback::from({
                let map = map.clone();
                move |map_instance: LeafletMap| map.set(Some(map_instance))
            })}
            on_location_name_changed={Callback::from({
                // let location_name = location_name.clone();
                move |lookup: NominatimLookup| {
                    lookup_handle.set(Some(lookup));
                }
            })}
            on_marker_created={Callback::from({
                move |marker: Marker| {
                    marker_handle.set(Some(marker));
                }
            })}
            style="height: 100%; width: 100%; border-radius: 1rem; border: 2px solid #f0f0f0;"
            class={classes!["z-50","w-64", "h-64", "border-2", "border-fuente", "rounded-3xl", "shadow-xl"]}
        />
    }
}
