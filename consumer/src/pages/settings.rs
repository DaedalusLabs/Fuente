use crate::contexts::{ConsumerDataAction, ConsumerDataStore};
use crate::router::ConsumerRoute;
use fuente::contexts::LanguageConfigsStore;
use fuente::mass::PopupProps;
use fuente::mass::{
    templates::SettingsPageTemplate, AddressLookupDetails, AppLink, CardComponent,
    ImageUploadInput, LanguageToggle, NewAddressForm, NewAddressProps, PopupSection, SimpleInput,
};
use fuente::models::{
    ConsumerAddress, ConsumerAddressIdb, ConsumerProfile, ConsumerProfileIdb, TEST_PUB_KEY,
};
use lucide_yew::{ScrollText, SquarePen, Truck, X};
use nostr_minions::{
    browser_api::{GeolocationCoordinates, HtmlForm},
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
};
use yew::prelude::*;
use yew::props;

#[derive(Clone, PartialEq)]
pub enum SettingsPage {
    Profile,
    Address,
    KeyRecovery,
    Language,
}

#[derive(Clone, PartialEq)]
pub enum ProfilePageMenu {
    None,
    EditProfile,
    AddAddress,
}

#[derive(Clone, PartialEq, Properties)]
pub struct MenuProps {
    pub handle: UseStateHandle<ProfilePageMenu>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct MenuHeaderProps {
    pub title: String,
    pub handle: UseStateHandle<ProfilePageMenu>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct AddressListItemProps {
    pub consumer_address: ConsumerAddressIdb,
}

#[function_component(SettingsPageComponent)]
pub fn settings_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();
    let current_page = use_state(|| SettingsPage::Profile);
    let go_to_profile = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(SettingsPage::Profile))
    };
    let go_to_address = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(SettingsPage::Address))
    };
    let go_to_key_recovery = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(SettingsPage::KeyRecovery))
    };
    let go_to_language = {
        let page = current_page.clone();
        Callback::from(move |_| page.set(SettingsPage::Language))
    };
    let my_orders_button = {
        html! {
            <div class="flex justify-center items-center">
                <AppLink<ConsumerRoute>
                    class="flex items-center bg-white border-2 border-fuente outline-fuente px-10 py-3 rounded-full text-fuente space-x-2 font-bold"
                    selected_class=""
                    route={ConsumerRoute::History}>
                    <ScrollText class={classes!("text-sm", "text-fuente", "scale-x-[-1]", "mr-2", "font-bold")} />
                    {&translations["profile_personal_information_orders_button"]}
                </AppLink<ConsumerRoute>>
            </div>
        }
    };
    let track_packages_button = {
        html! {
            <div class="flex justify-center items-center">
                <AppLink<ConsumerRoute>
                    class="flex items-center bg-white border-2 border-fuente outline-fuente px-10 py-3 rounded-full text-fuente space-x-2 font-bold"
                    selected_class=""
                    route={ConsumerRoute::TrackPackages}> // Changed from History to TrackPackages
                    <Truck class={classes!("text-sm", "text-fuente", "mr-2", "font-bold")} />
                    {&translations["profile_personal_information_packages_button"]}
                </AppLink<ConsumerRoute>>
            </div>
        }
    };
    let profile_popup_handle = use_state(|| false);
    let address_popup_handle = use_state(|| false);
    let edit_button = {
        let profile_popup_handle = profile_popup_handle.clone();
        let address_popup_handle = address_popup_handle.clone();
        match *current_page {
            SettingsPage::KeyRecovery => {
                html! {
                    <button
                        type="button" class="absolute right-2 top-2 m-2 flex gap-4 tracking-wide">
                        <span class="text-red-600 font-bold text-sm">
                            {&translations["profile_personal_information_delete_account_button"]}
                        </span>
                        <X class={classes!("feather", "feather-plus", "text-red-600","w-6", "h-6")} />
                    </button>
                }
            }
            SettingsPage::Language => {
                html! {}
            }
            SettingsPage::Address => {
                html! {
                    <button onclick={Callback::from(move |_| address_popup_handle.set(true))}
                        type="button" class="absolute right-2 top-2 m-2 flex gap-4 tracking-wide">
                        <span class="text-fuente font-bold text-xl">
                            {&translations["profile_personal_information_edit_button"]}
                        </span>
                        <SquarePen class={classes!("feather", "feather-plus", "text-fuente","w-6", "h-6")} />
                    </button>
                }
            }
            SettingsPage::Profile => {
                html! {
                    <button onclick={Callback::from(move |_| profile_popup_handle.set(true))}
                        type="button" class="absolute right-2 top-2 m-2 flex gap-4 tracking-wide">
                        <span class="text-fuente font-bold text-xl">
                            {&translations["profile_personal_information_edit_button"]}
                        </span>
                        <SquarePen class={classes!("feather", "feather-plus", "text-fuente","w-6", "h-6")} />
                    </button>
                }
            }
        }
    };
    html! {
        <SettingsPageTemplate
            heading={"My Profile".to_string()}
            options={ vec![
                my_orders_button,
                track_packages_button,
            ]}
            sidebar_options={ vec![
                (translations["profile_address_personal_information_button"].clone(), go_to_profile, if *current_page == SettingsPage::Profile { true } else { false }),
                (translations["profile_address_address_button"].clone(), go_to_address, if *current_page == SettingsPage::Address { true } else { false }),
                (translations["profile_settings_key"].clone(), go_to_key_recovery, if *current_page == SettingsPage::KeyRecovery { true } else { false }),
                (translations["profile_settings_language"].clone(), go_to_language, if *current_page == SettingsPage::Language { true } else { false }),
            ]}
            content_button={Some(edit_button)} >
            <>
            {match *current_page {
                    SettingsPage::Profile => html! {
                        <>
                        <MyContactDetails />
                        <PopupSection close_handle={profile_popup_handle.clone()}>
                            <EditProfileMenu close_handle={profile_popup_handle.clone()} />
                        </PopupSection>
                        </>
                    },
                    SettingsPage::Address => html! {
                        <>
                        <MyAddressDetails />
                        <PopupSection close_handle={address_popup_handle.clone()}>
                            <NewAddressMenu close_handle={address_popup_handle.clone()} />
                        </PopupSection>
                        </>
                    },
                    SettingsPage::KeyRecovery => html! {
                        <KeyRecoverySection />
                    },
                    SettingsPage::Language => html! {
                        <LanguageToggle />
                    },
            }}
            </>
        </SettingsPageTemplate>
    }
}

#[function_component(MyContactDetails)]
pub fn my_contact_details() -> Html {
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let profile = user_ctx.get_profile().expect("No user profile found");

    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();

    let profile_popup_handle = use_state(|| false);
    let profile_popup_handle_clone = profile_popup_handle.clone();
    html! {
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-10 h-full">
        <div class="space-y-3">
            <h3 class="text-gray-500 text-2xl font-semibold">{profile.nickname.clone()}</h3>

            <div class="flex flex-col xl:flex-row xl:items-center justify-between">
                <p class="text-gray-500 text-lg font-bold">{&translations["checkout_client_information_heading_email"]}</p>
                <p class="text-gray-500 font-light">{&profile.email}</p>
            </div>
            <div class="flex flex-col xl:flex-row xl:items-center justify-between">
                <p class="text-gray-500 text-lg font-bold">{&translations["checkout_client_information_heading_phone"]}</p>
                <p class="text-gray-500 font-light">{&profile.telephone}</p>
            </div>
        </div>
        <div class="flex flex-col gap-4 flex-1 items-center justify-center">
            <img class="w-36 h-36 rounded-lg" src={profile.avatar_url.clone()} />
            <div class="flex justify-center">
                <button onclick={Callback::from(move |_| profile_popup_handle_clone.set(true))} 
                    class="bg-fuente-buttons text-fuente-forms py-3 rounded-full px-10 font-semibold text-sm">
                    {&translations["profile_settings_upload"]}
                </button>
            </div>
        </div>
        <PopupSection close_handle={profile_popup_handle.clone()}>
            <EditProfileAvatarPopup close_handle={profile_popup_handle.clone()} />
        </PopupSection>
    </div>
    }
}

#[function_component(EditProfileAvatarPopup)]
pub fn edit_avatar(props: &PopupProps) -> Html {
    let close_handle = props.close_handle.clone();
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_ctx = use_context::<NostrProps>().expect("No RelayPool Context found");
    let profile = user_ctx.get_profile().expect("No user profile found");
    let avatar_url = use_state(|| None);
    let url_handle = avatar_url.clone();
    let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
    let user_keys = nostr_keys.clone();
    let sender = relay_ctx.send_note.clone();
    let url_clone = url_handle.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let mut user_profile = profile.clone();
        user_profile.avatar_url = (*url_clone).clone();
        let db = ConsumerProfileIdb::new(user_profile.clone(), &user_keys);
        let giftwrapped_note = user_profile
            .giftwrapped_data(&user_keys, user_keys.public_key())
            .expect("Failed to giftwrap data");
        let server_registry = user_profile
            .registry_data(&user_keys, TEST_PUB_KEY.to_string())
            .expect("Failed to giftwrap data");
        sender.emit(server_registry);
        sender.emit(giftwrapped_note);
        user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
        close_handle.set(false);
    });
    html! {
        <form {onsubmit}
            class="w-full flex flex-col gap-2 bg-white rounded-3xl p-4 items-center">
            <ImageUploadInput {url_handle} {nostr_keys} classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")} input_id="user-profile-upload" />
            <button
                type="submit"
                class="bg-fuente text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl"
                >{"Save"}
            </button>
        </form>
    }
}

#[function_component(EditProfileMenu)]
pub fn edit_profile_menu(props: &PopupProps) -> Html {
    let close_handle = props.close_handle.clone();
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let profile = user_ctx.get_profile().expect("No user profile found");
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();
    let profile_clone = profile.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let nickname = form
            .input_value("nickname")
            .expect("Failed to get nickname");
        let email = form.input_value("email").expect("Failed to get email");
        let telephone = form
            .input_value("telephone")
            .expect("Failed to get telephone");
        let user_keys = keys.clone();
        let sender = sender.clone();
        let user_ctx = user_ctx.clone();
        let mut user_profile = profile_clone.clone();
        user_profile.nickname = nickname;
        user_profile.email = email;
        user_profile.telephone = telephone;
        let db = ConsumerProfileIdb::new(user_profile.clone(), &user_keys);
        let giftwrapped_note = user_profile
            .giftwrapped_data(&user_keys, user_keys.public_key())
            .expect("Failed to giftwrap data");
        let server_registry = user_profile
            .registry_data(&user_keys, TEST_PUB_KEY.to_string())
            .expect("Failed to giftwrap data");
        sender.emit(server_registry);
        sender.emit(giftwrapped_note);
        user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
        close_handle.set(false);
    });
    html! {
        <form {onsubmit}
            class="w-full flex flex-col gap-2 bg-white rounded-3xl p-4 items-center">
            <EditProfileForm {profile} />
            <button
                type="submit"
                class="bg-fuente text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl"
                >{"Save"}
            </button>
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
        <div class="space-y-2">
            <label for="nickname" class="text-gray-500 text-lg font-bold">{"Nickname"}</label>
            <SimpleInput
                label={"Nickname"} value={profile.nickname.clone()}
                id={"nickname"} name={"nickname"}
                input_type={"text"} required={true} />
            <label for="email" class="text-gray-500 text-lg font-bold">{"Email"}</label>
            <SimpleInput
                label={"Email"} value={profile.email.clone()}
                id={"email"} name={"email"}
                input_type={"email"} required={true} />
            <label for="telephone" class="text-gray-500 text-lg font-bold">{"Telephone"}</label>
            <SimpleInput
                label={"Telephone"} value={profile.telephone.clone()}
                id={"telephone"} name={"telephone"}
                input_type={"tel"} required={true} />
        </div>
    }
}

#[function_component(MyAddressDetails)]
pub fn my_address_details() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();
    let mut addresses = use_context::<ConsumerDataStore>()
        .expect("No user context found")
        .get_address_entrys();
    addresses.sort_by(|a, b| a.is_default().cmp(&b.is_default()));
    addresses.reverse();
    let default_address = addresses.iter().find(|a| a.is_default());
    html! {
        {if let Some(address) = default_address {
            html! {
                <div>
                    <p class="text-xl font-bold text-gray-500">{&translations["profile_address_address_registered"]}</p>
                    <span class="text-xl font-thin text-gray-500">{address.address().lookup().display_name()}</span>
                </div>
            }
        } else {
            html! {
                <div>
                    <p class="text-xl font-bold text-gray-500">{&translations["profile_address_address_registered"]}</p>
                    <span class="text-xl font-thin text-gray-500">{&translations["profile_address_no_address_registered"]}</span>
                </div>
            }
        }}
    }
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
pub fn new_address_menu(props: &PopupProps) -> Html {
    let data_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let close_handle = props.close_handle.clone();
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
            data_ctx.dispatch(ConsumerDataAction::NewAddress(db_entry.clone()));
            data_ctx.dispatch(ConsumerDataAction::SetDefaultAddress(db_entry));
            handle.set(false);
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

#[function_component(KeyRecoverySection)]
fn key_recovery_section() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrIdStore found");
    let keys = key_ctx.get_nostr_key().expect("No keys found");

    // Convert secret key bytes to hex string
    let secret_key_bytes = keys.get_secret_key();
    let secret_key_hex: String = secret_key_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    html! {
        <div class="flex flex-col gap-4">
            <h2 class="text-2xl font-bold">{"Key Recovery"}</h2>
            <div class="bg-white p-4 rounded-lg shadow">
                <p class="mb-4">{"Your private key is very important. Store it safely:"}</p>
                <div class="bg-gray-100 p-4 rounded-lg break-all select-all">
                    {secret_key_hex}
                </div>
                <p class="mt-4 text-sm text-gray-600">
                    {"Save this key somewhere safe. You'll need it to recover your account."}
                </p>
            </div>
        </div>
    }
}

#[function_component(FAQSection)]
fn faq_section() -> Html {
    html! {
        <div class="flex flex-col gap-4">
            <h2 class="text-2xl font-bold">{"Frequently Asked Questions"}</h2>
            <div class="space-y-4">
                <div class="bg-white p-4 rounded-lg shadow">
                    <h3 class="font-bold mb-2">{"How do I place an order?"}</h3>
                    <p>{"Browse businesses, select items, add to cart, and checkout with Lightning payment."}</p>
                </div>
                <div class="bg-white p-4 rounded-lg shadow">
                    <h3 class="font-bold mb-2">{"How do payments work?"}</h3>
                    <p>{"We use Lightning Network for instant, low-fee payments."}</p>
                </div>
                // Add more FAQs as needed
            </div>
        </div>
    }
}

#[function_component(LegalSection)]
fn legal_section() -> Html {
    html! {
        <div class="flex flex-col gap-4">
            <h2 class="text-2xl font-bold">{"Legal Information"}</h2>
            <div class="bg-white p-4 rounded-lg shadow">
                <h3 class="font-bold mb-2">{"Terms of Service"}</h3>
                <p class="mb-4">{"By using our service, you agree to these terms..."}</p>

                <h3 class="font-bold mb-2">{"Privacy Policy"}</h3>
                <p class="mb-4">{"We respect your privacy and protect your data..."}</p>

                <h3 class="font-bold mb-2">{"Refund Policy"}</h3>
                <p>{"Our refund policy details..."}</p>
            </div>
        </div>
    }
}
