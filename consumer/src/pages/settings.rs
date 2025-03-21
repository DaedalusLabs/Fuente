use crate::contexts::{ConsumerDataAction, ConsumerDataStore};
use crate::router::ConsumerRoute;
use fuente::contexts::LanguageConfigsStore;
use fuente::mass::PopupProps;
use fuente::mass::{
    templates::{KeyRecoverySection, SettingsPageTemplate},
    AddressLookupDetails, AppLink, CardComponent, ImageUploadInput, LanguageToggle, NewAddressForm,
    NewAddressProps, PopupSection, SimpleInput,
};
use fuente::models::{
    ConsumerAddress, ConsumerAddressIdb, ConsumerProfile, ConsumerProfileIdb, TEST_PUB_KEY,
};
use lucide_yew::{Compass, Mail, MapPin, Phone, ScrollText, SquarePen, Trash2, Truck, Upload};
use nostr_minions::key_manager::NostrIdAction;
use nostr_minions::{
    browser_api::{GeolocationCoordinates, HtmlForm},
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
};
use yew::prelude::*;
use yew::props;
use yew_router::hooks::use_navigator;

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
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let translations = language_ctx.translations();
    let current_page = use_state(|| SettingsPage::Profile);
    let navigator = use_navigator().expect("Navigator not found");
    if user_ctx.get_profile().is_none() {
        navigator.push(&ConsumerRoute::Register);
        return html! { <div></div> };
    }
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
                class="lg:block hidden flex items-center bg-white border-2 border-fuente px-6 py-3 rounded-full text-fuente space-x-2 font-bold text-sm md:text-md lg:text-lg"
                selected_class=""
                route={ConsumerRoute::History}>
                <span class="hidden lg:flex text-lg font-bold text-center items-center gap-2">
                    <ScrollText class={classes!("h-6", "w-6", "stroke-fuente")} />
                    {&translations["profile_personal_information_orders_button"]}
                </span>
            </AppLink<ConsumerRoute>>
            <AppLink<ConsumerRoute>
                class="block lg:hidden flex items-center bg-white border-2 border-fuente p-2 rounded-xl"
                selected_class=""
                route={ConsumerRoute::History}>
                <ScrollText class={classes!("h-6", "w-6", "stroke-fuente")} />
            </AppLink<ConsumerRoute>>
            </div>
        }
    };
    let track_packages_button = {
        html! {
            <div class="flex justify-center items-center">
                <AppLink<ConsumerRoute>
                    class="lg:block hidden flex items-center bg-white border-2 border-fuente px-6 py-3 rounded-full text-fuente space-x-2 font-bold text-sm md:text-md lg:text-lg"
                    selected_class=""
                    route={ConsumerRoute::TrackPackages}> // Changed from History to TrackPackages
                    <span class="hidden lg:flex text-lg font-bold text-center items-center gap-2">
                        <Truck class={classes!("text-sm", "mr-2", "font-bold", "stroke-fuente")} />
                        {&translations["profile_personal_information_packages_button"]}
                    </span>
                </AppLink<ConsumerRoute>>
                <AppLink<ConsumerRoute>
                    class="block lg:hidden flex items-center bg-white border-2 border-fuente p-2 rounded-xl"
                    selected_class=""
                    route={ConsumerRoute::TrackPackages}>
                    <Truck class={classes!("h-6", "w-6", "stroke-fuente")} />
                </AppLink<ConsumerRoute>>
            </div>
        }
    };
    let profile_popup_handle = use_state(|| false);
    let address_popup_handle = use_state(|| false);
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let key_handle = key_ctx.clone();
    let edit_button = {
        let profile_popup_handle = profile_popup_handle.clone();
        let address_popup_handle = address_popup_handle.clone();
        match *current_page {
            SettingsPage::KeyRecovery => {
                let delete_key = {
                    let key_handle = key_handle.clone();
                    let message =
                        translations["profile_personal_information_delete_account_confirm"].clone();
                    Callback::from(move |_| {
                        match web_sys::window()
                            .expect("No window found")
                            .confirm_with_message(message.as_str())
                        {
                            Ok(true) => key_handle.dispatch(NostrIdAction::DeleteIdentity),
                            _ => (),
                        }
                    })
                };
                html! {
                    <button onclick={delete_key}
                        type="button" class="absolute right-2 top-2 m-2 flex gap-4 tracking-wide">
                        <span class="text-red-600 font-bold text-xl text-fuente">
                            {&translations["profile_personal_information_delete_account_button"]}
                        </span>
                        <Trash2 class={classes!("feather", "feather-plus", "text-red-600","w-6", "h-6")} />
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
            heading={translations["profile_address_heading"].clone()}
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
                        <div class="w-full">
                            <MyContactDetails />
                            <PopupSection close_handle={profile_popup_handle.clone()}>
                                <EditProfileMenu close_handle={profile_popup_handle.clone()} />
                            </PopupSection>
                        </div>
                    },
                    SettingsPage::Address => html! {
                        <div class="w-full">
                            <MyAddressDetails />
                            <PopupSection close_handle={address_popup_handle.clone()}>
                                <NewAddressMenu close_handle={address_popup_handle.clone()} />
                            </PopupSection>
                        </div>
                    },
                    SettingsPage::KeyRecovery => html! {
                        <div class="w-full">
                            <KeyRecoverySection />
                        </div>
                    },
                    SettingsPage::Language => html! {
                        <div class="w-full">
                            <LanguageToggle />
                        </div>
                    },
            }}
            </>
        </SettingsPageTemplate>
    }
}

#[function_component(MyContactDetails)]
pub fn my_contact_details() -> Html {
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();
    let profile_popup_handle = use_state(|| false);

    // Get profile safely
    match user_ctx.get_profile() {
        Some(profile) => {
            html! {
                <div class="w-full lg:mt-6 lg:mr-6 flex flex-1 lg:items-center">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 md:gap-10 lg:gap-32 h-full p-6 rounded-lg">
                        <div class="space-y-6">
                            <h3 class="text-gray-500 text-2xl font-semibold border-b pb-2">
                                {&profile.nickname}
                            </h3>

                            <div class="space-y-4">
                                <div class="flex items-center space-x-3">
                                    <Mail class="text-gray-500 w-5 h-5" />
                                    <div>
                                        <p class="text-gray-500 text-lg font-bold">{&translations["checkout_client_information_heading_email"]}</p>
                                        <p class="text-gray-600 font-light">{&profile.email}</p>
                                    </div>
                                </div>

                                <div class="flex items-center space-x-3">
                                    <Phone class="text-gray-500 w-5 h-5" />
                                    <div>
                                        <p class="text-gray-500 text-lg font-bold">{&translations["checkout_client_information_heading_phone"]}</p>
                                        <p class="text-gray-600">{&profile.telephone}</p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="flex flex-col items-center justify-center space-y-6">
                            <div class="relative w-full max-w-xs aspect-square">
                                <img
                                    src={profile.avatar_url.clone().unwrap_or_else(|| "/public/assets/img/logo.png".to_string())}
                                    alt="Profile Logo"
                                    class="border-2 border-dashed border-gray-300 bg-gray-100 rounded-lg object-cover w-full h-full max-w-56 max-h-56"
                                />
                            </div>

                            <button
                                onclick={
                                    let profile_popup_handle = profile_popup_handle.clone();
                                    Callback::from(move |_| profile_popup_handle.set(true))
                                }
                                class="bg-fuente-light text-white py-3 rounded-full px-10 font-semibold flex items-center space-x-2 hover:bg-opacity-90 transition duration-300"
                            >
                                <Upload class="w-5 h-5 stroke-white" />
                                <span>{&translations["profile_settings_upload"]}</span>
                            </button>
                        </div>
                    </div>
                    <PopupSection close_handle={profile_popup_handle.clone()}>
                        <EditProfileAvatarPopup close_handle={profile_popup_handle.clone()} />
                    </PopupSection>
                </div>
            }
        }
        None => {
            html! {
                <div class="flex flex-col items-center justify-center p-8">
                    <h2 class="text-2xl font-semibold mb-4">{&translations["profile_not_setup"]}</h2>
                    <p class="text-gray-600 mb-4">{&translations["profile_setup_required"]}</p>
                    <AppLink<ConsumerRoute>
                        class="bg-fuente text-white px-6 py-2 rounded-lg"
                        selected_class=""
                        route={ConsumerRoute::Home}>
                        {&*translations["back_to_home"]}
                    </AppLink<ConsumerRoute>>
                </div>
            }
        }
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
    let nostr_keys = key_ctx.clone();
    let user_keys = nostr_keys.get_identity().cloned();
    let sender = relay_ctx.send_note.clone();
    let url_clone = url_handle.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let profile = profile.clone();
        let url_clone = url_clone.clone();
        let sender = sender.clone();
        let user_ctx = user_ctx.clone();
        let close_handle = close_handle.clone();
        let user_keys = user_keys.clone();
        yew::platform::spawn_local(async move {
            let mut user_profile = profile.clone();
            let user_keys = user_keys.clone().expect("No user keys found");
            let pubkey = user_keys.get_pubkey().await.expect("No pubkey found");

            user_profile.avatar_url = (*url_clone).clone();
            let db = ConsumerProfileIdb::new(user_profile.clone(), &user_keys).await;
            let giftwrapped_note = user_profile
                .giftwrapped_data(&user_keys, pubkey)
                .await
                .expect("Failed to giftwrap data");
            let server_registry = user_profile
                .registry_data(&user_keys, TEST_PUB_KEY.to_string())
                .await
                .expect("Failed to giftwrap data");
            sender.emit(server_registry);
            sender.emit(giftwrapped_note);
            user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
            close_handle.set(false);
        });
    });
    html! {
        <form {onsubmit}
            class="w-full flex flex-col gap-2 bg-white rounded-3xl p-4 items-center">
            <ImageUploadInput {url_handle} nostr_keys={nostr_keys.get_identity().cloned().expect("no id")} classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")} input_id="user-profile-upload" />
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
    let keys = key_ctx.get_identity().cloned().expect("No user keys found");
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
        let close_handle = close_handle.clone();
        yew::platform::spawn_local(async move {
            let db = ConsumerProfileIdb::new(user_profile.clone(), &user_keys).await;
            let giftwrapped_note = user_profile
                .giftwrapped_data(&user_keys, user_keys.get_pubkey().await.expect("No pubkey"))
                .await
                .expect("Failed to giftwrap data");
            let server_registry = user_profile
                .registry_data(&user_keys, TEST_PUB_KEY.to_string())
                .await
                .expect("Failed to giftwrap data");
            sender.emit(server_registry);
            sender.emit(giftwrapped_note);
            user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
            close_handle.set(false);
        });
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
            let lookup = address.address().lookup();
            html! {
                <div class="max-w-full flex flex-col p-6 rounded-lg space-y-6 overflow-hidden">
                    <h3 class="text-gray-800 text-2xl font-semibold border-b pb-2">
                      {&translations["profile_address_registered"]}
                    </h3>

                    <div class="space-y-4">
                      <div class="flex items-start space-x-3">
                        <MapPin class="text-gray-500 w-5 h-5 mt-1 flex-shrink-0" />
                        <div class="flex-grow">
                          <p class="text-gray-700 text-lg font-bold">
                            {&translations["profile_address_registered"]}
                          </p>
                          <p class="text-gray-600 text-xl font-light break-words max-w-xs sm:max-w-sm">
                            {&lookup.display_name()}
                          </p>
                        </div>
                      </div>

                      <div class="flex items-start space-x-3">
                        <Compass class="text-gray-500 w-5 h-5 mt-1 flex-shrink-0" />
                        <div class="flex-grow">
                          <p class="text-gray-700 text-lg font-bold">
                            {&translations["profile_address_coordinates"]}
                          </p>
                          <p class="text-gray-600 text-xl font-light">
                            {format!("Latitude: {:.2} Longitude: {:.2}", &lookup.lat_as_f64(), &lookup.long_as_f64())}
                          </p>
                        </div>
                      </div>
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="p-10 ">
                    <p class="text-xl font-bold text-gray-500">{&translations["profile_address_registered"]}</p>
                    <span class="text-xl font-thin text-gray-500">{&translations["profile_no_address_registered"]}</span>
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
        if let (Some(address), Some(coords), Some(keys)) = (
            address.clone(),
            coords.clone(),
            key_ctx.get_identity().cloned(),
        ) {
            let address = ConsumerAddress::new(address, coords.into());
            let close_handle = close_handle.clone();
            let data_ctx = data_ctx.clone();
            yew::platform::spawn_local(async move {
                let db_entry = ConsumerAddressIdb::new(address.clone(), &keys).await;
                let handle = close_handle.clone();
                data_ctx.dispatch(ConsumerDataAction::NewAddress(db_entry.clone()));
                data_ctx.dispatch(ConsumerDataAction::SetDefaultAddress(db_entry));
                handle.set(false);
            });
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
        <div class="bg-fuente-dark rounded-3xl p-8 max-w-sm sm:max-w-md md:max-w-lg lg:max-w-xl xl:max-w-2xl">
            <NewAddressForm ..props />
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
