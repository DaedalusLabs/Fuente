use super::PageHeader;
use crate::contexts::{ConsumerDataAction, ConsumerDataStore};
use fuente::mass::LookupIcon;
use fuente::mass::{
    AddressLookupDetails, CardComponent, ConsumerProfileDetails, ImageUploadInput, NewAddressForm,
    NewAddressProps, PopupSection, SimpleInput, templates::SettingsPageTemplate,
};
use fuente::models::{
    ConsumerAddress, ConsumerAddressIdb, ConsumerProfile, ConsumerProfileIdb, TEST_PUB_KEY,
};
use lucide_yew::ScrollText;
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
    let my_orders_button = {
        html! {
            <>
                <ScrollText class={classes!("text-sm", "text-fuente", "scale-x-[-1]", "mr-2")} />
                {"My Orders"}
            </>
        }
    };
    let my_orders_onclick = {
        let page = current_page.clone();
        Callback::noop()
    };
    html! {
        <SettingsPageTemplate
            heading={"My Profile".to_string()}
            options={ vec![
                (my_orders_button, my_orders_onclick),
            ]}
            sidebar_options={ vec![
                ("Profile Settings".to_string(), go_to_profile, if *current_page == SettingsPage::Profile { true } else { false }),
                ("Address Settings".to_string(), go_to_address, if *current_page == SettingsPage::Address { true } else { false }),
                ("Key Recovery".to_string(), go_to_key_recovery, if *current_page == SettingsPage::KeyRecovery { true } else { false }),
            ]}
            content_button={None}>
            <>
            {match *current_page {
                    SettingsPage::Profile => html! {
                        <div class="flex flex-col w-full h-full gap-8 px-4">
                        </div>
                    },
                    SettingsPage::KeyRecovery => html! {
                        <KeyRecoverySection />
                    },
                    SettingsPage::Address => html! {
                        <>
                        </>

                    },
            }}
            </>
        </SettingsPageTemplate>
    }
}
#[function_component(SettingsPageComponent2)]
pub fn settings_page() -> Html {
    let current_page = use_state(|| SettingsPage::Profile);
    let profile_menu_state = use_state(|| ProfilePageMenu::None);

    html! {
        <div class="h-full w-full flex flex-col">
            <PageHeader title={"Settings".to_string()} />

            <div class="flex flex-row h-full">
                // Settings Menu
                <div class="w-64 border-r p-4">
                    <div class="flex flex-col gap-4">
                        <button
                            onclick={let page = current_page.clone();
                                Callback::from(move |_| page.set(SettingsPage::Profile))}
                            class="text-left p-2 hover:bg-gray-100 rounded">
                            {"Profile Settings"}
                        </button>
                        <button
                            onclick={let page = current_page.clone();
                                Callback::from(move |_| page.set(SettingsPage::KeyRecovery))}
                            class="text-left p-2 hover:bg-gray-100 rounded">
                            {"Key Recovery"}
                        </button>
                    </div>
                </div>

                // Content Area
                <div class="flex-1 p-4">
                    {match *current_page {
                        SettingsPage::Profile => match *profile_menu_state {
                            ProfilePageMenu::None => html! {
                                <div class="flex flex-col w-full h-full gap-8 px-4">
                                    <MyAvatar handle={profile_menu_state.clone()} />
                                    <MyContactDetails handle={profile_menu_state.clone()} />
                                    <MyAddressDetails handle={profile_menu_state.clone()} />
                                </div>
                            },
                            ProfilePageMenu::EditProfile => html! {
                                <div class="flex flex-col w-full h-full gap-8">
                                    <EditProfileMenu handle={profile_menu_state.clone()} />
                                </div>
                            },
                            ProfilePageMenu::AddAddress => html! {
                                <div class="flex flex-col w-full flex-1 gap-8">
                                    <NewAddressMenu handle={profile_menu_state.clone()} />
                                </div>
                            },
                        },
                        SettingsPage::KeyRecovery => html! {
                            <KeyRecoverySection />
                        },
                        SettingsPage::Address => html! {
                            <></>
                        },
                    }}
                </div>
            </div>
        </div>
    }
}

#[function_component(MyAvatar)]
pub fn my_avatar(props: &MenuProps) -> Html {
    let user_ctx = use_context::<ConsumerDataStore>().expect("No user context found");
    let profile = user_ctx.get_profile().expect("No user profile found");
    let handle = props.handle.clone();
    html! {
        <div class="w-full flex flex-col gap-2">
            <div class="flex flex-row justify-between items-center pr-4">
                <h3 class="font-bold">{"Avatar"}</h3>
                <button
                    onclick={Callback::from(move |_| handle.set(ProfilePageMenu::EditProfile))}
                    class="text-sm text-fuente">{"Edit"}</button>
            </div>
            <CardComponent>
                <img class="w-24 h-24 rounded-lg" src={profile.avatar_url.clone()} />
            </CardComponent>
        </div>
    }
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
    let avatar_url = use_state(|| profile.avatar_url.clone());
    let url_handle = avatar_url.clone();
    let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
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
        let user_profile = ConsumerProfile::new(nickname, email, telephone, (*avatar_url).clone());
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
        handle.set(ProfilePageMenu::None);
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
            <ImageUploadInput {url_handle} {nostr_keys} classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")} input_id="user-profile-upload" />
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
                label={"Nickname"} value={profile.nickname.clone()}
                id={"nickname"} name={"nickname"}
                input_type={"text"} required={true} />
            <SimpleInput
                label={"Email"} value={profile.email.clone()}
                id={"email"} name={"email"}
                input_type={"email"} required={true} />
            <SimpleInput
                label={"Telephone"} value={profile.telephone.clone()}
                id={"telephone"} name={"telephone"}
                input_type={"tel"} required={true} />
        </div>
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

