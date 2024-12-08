use crate::contexts::{ConsumerDataAction, ConsumerDataStore};

use super::PageHeader;
use fuente::{
    mass::{
        AddressLookupDetails, BackArrowIcon, CardComponent, ConsumerProfileDetails, LookupIcon,
        NewAddressForm, NewAddressProps, PopupSection, SimpleInput,
    },
    models::{ConsumerAddress, ConsumerAddressIdb, ConsumerProfile, ConsumerProfileIdb},
};
use nostr_minions::{
    browser_api::{GeolocationCoordinates, HtmlForm},
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
};
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

#[function_component(NewAddressMenu)]
pub fn new_address_menu(props: &MenuProps) -> Html {
    let data_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let close_handle = props.handle.clone();
    let coordinate_state = use_state(|| None);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
    let address = (*nominatim_state).clone();
    let coords: Option<GeolocationCoordinates> = (*coordinate_state).clone();
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
    let props = props!(NewAddressProps {
        map_handle: map_state,
        marker_handle: marker_state,
        coord_handle: coordinate_state.clone(),
        nominatim_handle: nominatim_state.clone(),
        onclick
    });
    html! {
        <>
            <NewAddressForm ..props />
        </>
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
                .giftwrapped_data(&user_keys, user_keys.public_key())
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
