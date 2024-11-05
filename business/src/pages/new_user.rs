use fuente::{
    browser_api::{GeolocationCoordinates, GeolocationPosition, HtmlForm},
    contexts::{key_manager::NostrIdStore, relay_pool::NostrProps},
    mass::{
        atoms::{
            forms::{SimpleInput, SimpleTextArea},
            layouts::CardComponent,
        },
        molecules::address::AddressLookupDetails,
    },
    models::commerce::{CommerceProfile, CommerceProfileIdb},
    widgets::leaflet::{IconOptions, LatLng, LeafletMap, Marker, NominatimLookup, L},
};
use gloo::timers::callback::Timeout;
use wasm_bindgen::{JsCast, JsValue};
use yew::{platform::spawn_local, prelude::*, props};

use crate::contexts::commerce_data::{CommerceDataAction, CommerceDataStore};

#[function_component(NewProfilePage)]
pub fn edit_profile_menu() -> Html {
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();

    let coordinate_state = use_state(|| None);
    let nominatim_state = use_state(|| None);
    let props = props!(CommerceAddressProps {
        coord_handle: coordinate_state.clone(),
        nominatim_handle: nominatim_state.clone(),
    });

    let coords = (*coordinate_state).clone();
    let address = (*nominatim_state).clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let user_keys = keys.clone();
        let sender = sender.clone();
        let user_ctx = user_ctx.clone();
        let new_profile = CommerceProfile::new(
            form.input_value("name").expect("Failed to get name"),
            form.textarea_value("description")
                .expect("Failed to get description"),
            form.input_value("telephone")
                .expect("Failed to get telephone"),
            form.input_value("web").expect("Failed to get web"),
            address.clone().expect("No address found"),
            coords.clone().expect("No coordinates found"),
            form.input_value("ln_address")
                .expect("Failed to get lightning address"),
        );
        let db = CommerceProfileIdb::new(new_profile.clone(), &user_keys)
            .expect("Failed to create profile");
        let note = db.signed_note();
        sender.emit(note.clone());
        user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
    });
    html! {
        <form {onsubmit}
            class="w-full h-full flex flex-col gap-4 px-8 py-4">
                <div class="flex flex-row w-full justify-between items-center pr-4">
                    <h3 class="font-bold">{"Business Details"}</h3>
                    <button
                        type="submit"
                        class="text-sm bg-purple-900 text-white font-bold p-2 px-4 rounded-3xl"
                        >{"Save"}</button>
                </div>
                <ProfileInputs />
                <NewAddressMenu ..props/>
        </form>
    }
}

#[function_component(ProfileInputs)]
pub fn profile_inputs() -> Html {
    html! {
        <div class="flex flex-col gap-2">
            <SimpleInput
                id="name"
                name="name"
                label="Name"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleInput
                id="telephone"
                name="telephone"
                label="Telephone"
                value=""
                input_type="tel"
                required={true}
                />
            <SimpleInput
                id="web"
                name="web"
                label="Website"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleInput
                id="ln_address"
                name="ln_address"
                label="Lightning Address"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleTextArea
                id="description"
                name="description"
                label="Description"
                value=""
                input_type="text"
                required={true}
                />
        </div>
    }
}

pub fn start_new_address_picker_map(
    location: GeolocationCoordinates,
    map_handler: UseStateHandle<Option<LeafletMap>>,
    marker_handler: UseStateHandle<Option<Marker>>,
    geo_handler: UseStateHandle<Option<GeolocationCoordinates>>,
    address_handler: UseStateHandle<Option<NominatimLookup>>,
) -> Result<(), JsValue> {
    let map = L::render_default_map("map", &location)?;
    map_handler.set(Some(map.clone()));
    let icon_options = IconOptions {
        icon_url: "public/assets/img/marker.png".to_string(),
        icon_size: None,
        icon_anchor: None,
    };
    let marker = map.add_marker_with_icon(&location, icon_options)?;
    marker_handler.set(Some(marker.clone()));
    geo_handler.set(Some(location));

    let geo_handler_clone = geo_handler.clone();
    let address_handler_clone = address_handler.clone();
    let map_closure = move |e: JsValue| {
        let leaflet_event = LatLng::try_from(e).expect("Failed to get LatLng");
        let coordinates: GeolocationCoordinates = leaflet_event.clone().into();
        geo_handler_clone.set(Some(coordinates.clone()));
        marker.set_lat_lng(&leaflet_event.try_into().expect("Failed to convert LatLng"));
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
#[derive(Clone, PartialEq, Properties)]
pub struct CoordinateLocationProps {
    pub map_handle: UseStateHandle<Option<LeafletMap>>,
    pub marker_handle: UseStateHandle<Option<Marker>>,
    pub coord_handle: UseStateHandle<Option<GeolocationCoordinates>>,
    pub nominatim_handle: UseStateHandle<Option<NominatimLookup>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct CommerceAddressProps {
    pub coord_handle: UseStateHandle<Option<GeolocationCoordinates>>,
    pub nominatim_handle: UseStateHandle<Option<NominatimLookup>>,
}

#[function_component(NewAddressMenu)]
pub fn new_address_menu(props: &CommerceAddressProps) -> Html {
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
    let props = props!(CoordinateLocationProps {
        map_handle: map_state,
        marker_handle: marker_state,
        coord_handle: props.coord_handle.clone(),
        nominatim_handle: props.nominatim_handle.clone(),
    });
    html! {
        <>
            <div class="w-full flex flex-col gap-2">
                <AddressDetails ..props.clone()/>
            </div>
            <AddressSearch ..props.clone() />
            <AddressPickerMap ..props/>
        </>
    }
}

#[function_component(AddressDetails)]
pub fn address_details(props: &CoordinateLocationProps) -> Html {
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
pub fn address_search(props: &CoordinateLocationProps) -> Html {
    let handle = props.nominatim_handle.clone();
    let coordinate_handle = props.coord_handle.clone();
    let map_handle = props.map_handle.clone();
    let marker_handle = props.marker_handle.clone();
    let search_results = use_state(|| vec![]);

    let result_handle = search_results.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        let query = e
            .target()
            .unwrap()
            .dyn_ref::<web_sys::HtmlInputElement>()
            .unwrap()
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
                    id="address"
                    name="address"
                    class="w-full p-2 px-4 rounded-3xl shadow-xl bg-transparent placeholder:bg-transparent
                        focus:outline-none focus:ring-purple-900 focus:ring-2"
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
                            let coordinate_handle = coordinate_handle.clone();
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

#[function_component(AddressPickerMap)]
pub fn address_picker(props: &CoordinateLocationProps) -> Html {
    let coordinate_handle = props.coord_handle.clone();
    let nominatim_handle = props.nominatim_handle.clone();
    let map_handle = props.map_handle.clone();
    let marker_handle = props.marker_handle.clone();
    use_effect_with((), move |_| {
        let address_handle = nominatim_handle.clone();
        spawn_local(async move {
            if let Ok(position) = GeolocationPosition::locate().await {
                if let Err(e) = start_new_address_picker_map(
                    position.coords.clone(),
                    map_handle,
                    marker_handle,
                    coordinate_handle,
                    nominatim_handle,
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
        <div id="map" class="w-full h-full min-h-64 border-2 border-purple-900 rounded-3xl shadow-xl"></div>
    }
}
