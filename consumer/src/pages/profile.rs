use crate::contexts::{ConsumerDataAction, ConsumerDataStore};

use super::PageHeader;
use fuente::{
    browser_api::{GeolocationCoordinates, GeolocationPosition, HtmlForm},
    contexts::{NostrIdStore, NostrProps},
    mass::{
        AddressLookupDetails, BackArrowIcon, CardComponent, ConsumerProfileDetails, LookupIcon,
        PopupSection, SimpleInput,
    },
    models::{
        {ConsumerAddress, ConsumerAddressIdb}, {ConsumerProfile, ConsumerProfileIdb},
    },
    widgets::leaflet::{
        IconOptions, LatLng, LeafletMap, LeafletMapOptions, Marker, NominatimLookup, L,
    },
};
use gloo::timers::callback::Timeout;
use wasm_bindgen::{JsCast, JsValue};
use yew::{platform::spawn_local, prelude::*, props};

#[function_component(ProfilePage)]
pub fn profile_page() -> Html {
    let menu_state = use_state(|| ProfilePageMenu::None);
    html! {
        <div class="h-full w-full flex flex-col justify-between items-center">
            {match &(*menu_state) {
                ProfilePageMenu::None => html! {<>
                    <PageHeader title={"My Profile".to_string()} />
                    <div class="flex flex-col w-full h-full gap-8 px-4">
                        <MyContactDetails handle={menu_state.clone()} />
                        <MyAddressDetails handle={menu_state.clone()} />
                    </div>
                </>},
                ProfilePageMenu::EditProfile => html! {<>
                    <MenuHeader title={"Edit Profile".to_string()} handle={menu_state.clone()} />
                    <div class="flex flex-col w-full h-full gap-8">
                        <EditProfileMenu handle={menu_state.clone()} />
                    </div>
                </>},
                ProfilePageMenu::AddAddress => html! {<>
                    <MenuHeader title={"Add Address".to_string()} handle={menu_state.clone()} />
                    <div class="flex flex-col w-full flex-1 gap-8">
                        <NewAddressMenu handle={menu_state.clone()} />
                    </div>
                </>},
            }}
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct MenuHeaderProps {
    pub title: String,
    pub handle: UseStateHandle<ProfilePageMenu>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct MenuProps {
    pub handle: UseStateHandle<ProfilePageMenu>,
}

#[function_component(MenuHeader)]
pub fn page_header(props: &MenuHeaderProps) -> Html {
    let handle = props.handle.clone();
    html! {
        <div class="w-full flex flex-row items-center justify-between p-4">
            <button class="" onclick={Callback::from(move |_|{ handle.set(ProfilePageMenu::None)})}>
                <BackArrowIcon class="w-8 h-8 stroke-black" />
            </button>
            <h3 class="flex-1 text-center text-lg font-semibold">{&props.title}</h3>
            <div class="h-8 w-8"></div>
        </div>
    }
}

#[derive(Clone, PartialEq)]
pub enum ProfilePageMenu {
    None,
    EditProfile,
    AddAddress,
}

#[function_component(MyContactDetails)]
pub fn my_contact_details(props: &MenuProps) -> Html {
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let profile = user_ctx.get_profile().expect("No user profile found");

    let handle = props.handle.clone();
    html! {
        <div class="w-full flex flex-col gap-2">
            <div class="flex flex-row justify-between items-center pr-4">
                <h3 class="font-bold">{"Contact Details"}</h3>
                <button
                    onclick={Callback::from(move |_| handle.set(ProfilePageMenu::EditProfile))}
                    class="text-sm text-fuente">{"Edit"}</button>
            </div>
            <CardComponent>
                <ConsumerProfileDetails consumer_profile={profile} />
            </CardComponent>
        </div>
    }
}

#[function_component(MyAddressDetails)]
pub fn my_address_details(props: &MenuProps) -> Html {
    let close_handle = props.handle.clone();
    let mut addresses = use_context::<ConsumerDataStore>()
        .expect("No user context found")
        .get_address_entrys();
    addresses.sort_by(|a, b| a.is_default().cmp(&b.is_default()));
    addresses.reverse();
    html! {
        <div class="w-full flex-1 flex flex-col gap-4 overflow-hidden">
            <div class="flex flex-row justify-between items-center pr-4">
            <h3 class="font-bold">{"My Addresses"}</h3>
                <button
                    onclick={Callback::from(move |_| close_handle.set(ProfilePageMenu::AddAddress))}
                    class="text-sm text-fuente">{"Add Address"}</button>
            </div>
            <div class="w-full flex-1 flex flex-col gap-4 overflow-y-scroll">
                {if !addresses.is_empty() {
                    gloo::console::log!(format!("{}", addresses.len()));
                    addresses.iter().map(|address| {
                        html! {
                            <AddressListItem consumer_address={address.clone()} />
                        }
                    }).collect::<Html>()
                } else {
                    html! {
                        <div class="w-full h-full flex-1 flex flex-col gap-2 justify-center items-center">
                            <LookupIcon class="w-16 h-16 stroke-neutral-300" />
                            <p class="text-neutral-400 text-sm font-semibold">
                                {"No Address Found"}
                            </p>
                        </div>
                    }
                }}
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct AddressListItemProps {
    pub consumer_address: ConsumerAddressIdb,
}

#[function_component(AddressListItem)]
pub fn address_list_item(props: &AddressListItemProps) -> Html {
    let AddressListItemProps { consumer_address } = props;
    let popup_handle = use_state(|| false);
    let lookup = consumer_address.address().lookup();
    let consumer_ctx = use_context::<ConsumerDataStore>().expect("No user context found");

    let set_as_default = {
        let handle = consumer_ctx.clone();
        let address = consumer_address.clone();
        let popup = popup_handle.clone();
        Callback::from(move |_| {
            handle.dispatch(ConsumerDataAction::SetDefaultAddress(address.clone()));
            popup.set(false);
        })
    };

    let delete_address = {
        let handle = consumer_ctx.clone();
        let address = consumer_address.clone();
        let popup = popup_handle.clone();
        Callback::from(move |_| {
            handle.dispatch(ConsumerDataAction::DeleteAddress(address.clone()));
            popup.set(false);
        })
    };
    let is_default_class = match consumer_address.is_default() {
        true => "border-2 border-fuente rounded-3xl",
        false => "",
    };
    html! {
        <>
            <div onclick={
                let handle = popup_handle.clone();
                Callback::from(move |_| handle.set(true))}>
                    <div class={is_default_class}>
                        <CardComponent>
                            <AddressLookupDetails lookup={lookup.clone()} />
                        </CardComponent>
                    </div>
            </div>
            <PopupSection close_handle={popup_handle.clone()}>
                <CardComponent>
                    <div class="flex flex-col gap-4">
                        <h3 class="font-bold">{"Edit Address"}</h3>
                        <AddressLookupDetails lookup={lookup.clone()} />
                        <div class="mt-4 w-full flex flex-row justify-end gap-4">
                            <button
                                onclick={set_as_default}
                                class="bg-fuente text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl"
                                >{"Set as Default"}
                            </button>
                            <button
                                onclick={delete_address}
                                class="bg-red-500 text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl"
                                >{"Delete"}
                            </button>
                        </div>
                    </div>
                </CardComponent>
            </PopupSection>
        </>
    }
}

pub fn start_new_address_picker_map(
    location: GeolocationCoordinates,
    map_handler: UseStateHandle<Option<LeafletMap>>,
    marker_handler: UseStateHandle<Option<Marker>>,
    geo_handler: UseStateHandle<Option<GeolocationCoordinates>>,
    address_handler: UseStateHandle<Option<NominatimLookup>>,
) -> Result<(), JsValue> {
    let map_options = LeafletMapOptions {
        double_click_zoom: false,
        center: Some(location.clone().into()),
        ..Default::default()
    };
    let map = L::render_map_with_options("map", map_options)?;
    map_handler.set(Some(map.clone()));
    let icon_options = IconOptions {
        icon_url: "public/assets/img/marker.png".to_string(),
        icon_size: Some(vec![32, 32]),
        icon_anchor: None,
    };
    let marker = map.add_marker_with_icon(&location, icon_options)?;
    marker_handler.set(Some(marker.clone()));
    geo_handler.set(Some(location));

    let geo_handler_clone = geo_handler.clone();
    let address_handler_clone = address_handler.clone();
    let map_closure = move |e: MouseEvent| {
        gloo::console::log!("Event: ", &e);
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
#[derive(Clone, PartialEq, Properties)]
pub struct CoordinateLocationProps {
    pub map_handle: UseStateHandle<Option<LeafletMap>>,
    pub marker_handle: UseStateHandle<Option<Marker>>,
    pub coord_handle: UseStateHandle<Option<GeolocationCoordinates>>,
    pub nominatim_handle: UseStateHandle<Option<NominatimLookup>>,
}

#[function_component(NewAddressMenu)]
pub fn new_address_menu(props: &MenuProps) -> Html {
    let data_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let close_handle = props.handle.clone();
    let coordinate_state = use_state(|| None);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
    let props = props!(CoordinateLocationProps {
        map_handle: map_state,
        marker_handle: marker_state,
        coord_handle: coordinate_state.clone(),
        nominatim_handle: nominatim_state.clone(),
    });
    let address = (*nominatim_state).clone();
    let coords = (*coordinate_state).clone();
    let onclick = Callback::from(move |_| {
        if let (Some(address), Some(coords), Some(keys)) =
            (address.clone(), coords.clone(), key_ctx.get_nostr_key())
        {
            let address = ConsumerAddress::new(address, coords.into());
            let db_entry = ConsumerAddressIdb::new(address.clone(), &keys);
            let handle = close_handle.clone();
            handle.set(ProfilePageMenu::None);
            data_ctx.dispatch(ConsumerDataAction::NewAddress(db_entry));
        }
    });
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
        <div id="map" class="w-full h-full border-2 border-fuente rounded-3xl shadow-xl"></div>
    }
}

#[function_component(EditProfileMenu)]
pub fn edit_profile_menu(props: &MenuProps) -> Html {
    let MenuProps { handle } = props;
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let profile = user_ctx.get_profile().expect("No user profile found");
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();
    let handle = handle.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        let form = HtmlForm::new(e).expect("Failed to get form");
        let nickname = form
            .input_value("nickname")
            .expect("Failed to get nickname");
        let email = form.input_value("email").expect("Failed to get email");
        let telephone = form
            .input_value("telephone")
            .expect("Failed to get telephone");
        let user_keys = keys.clone();
        let handle = handle.clone();
        let sender = sender.clone();
        let user_ctx = user_ctx.clone();
        spawn_local(async move {
            let user_profile = ConsumerProfile::new(nickname, email, telephone);
            let db = ConsumerProfileIdb::new(user_profile.clone(), &user_keys);
            let giftwrapped_note = user_profile
                .giftwrapped_data(&user_keys, user_keys.get_public_key())
                .expect("Failed to giftwrap data");
            sender.emit(giftwrapped_note);
            user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
            handle.set(ProfilePageMenu::None);
        });
    });
    html! {
        <form {onsubmit}
            class="w-full flex flex-col gap-2">
            <div class="flex flex-row justify-between items-center pr-4">
                <h3 class="font-bold">{"Edit Profile"}</h3>
                <button
                    type="submit"
                    class="bg-fuente text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl"
                    >{"Save"}
                </button>
            </div>
            <EditProfileForm {profile} />
        </form>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct EditProfileFormProps {
    pub profile: ConsumerProfile,
}

#[function_component(EditProfileForm)]
pub fn edit_profile_form(props: &EditProfileFormProps) -> Html {
    let EditProfileFormProps { profile } = props;
    html! {
        <div class="w-full flex flex-col gap-2">
            <SimpleInput
                label={"Nickname"} value={profile.nickname()}
                id={"nickname"} name={"nickname"}
                input_type={"text"} required={true} />
            <SimpleInput
                label={"Email"} value={profile.email()}
                id={"email"} name={"email"}
                input_type={"email"} required={true} />
            <SimpleInput
                label={"Telephone"} value={profile.telephone()}
                id={"telephone"} name={"telephone"}
                input_type={"tel"} required={true} />
        </div>
    }
}
