use crate::contexts::{CommerceDataAction, CommerceDataStore};
use fuente::mass::{
    CardComponent, CommerceProfileAddressDetails, CommerceProfileDetails,
    DrawerSection, NewAddressForm, NewAddressProps, SimpleInput,
    SimpleTextArea,
};
use fuente::models::{CommerceProfile, CommerceProfileIdb};
use nostr_minions::{
    browser_api::HtmlForm,
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
};
use yew::{prelude::*, props};
use fuente::mass::CommerceProfileProps;

#[derive(Clone, PartialEq)]
pub enum SettingsPage {
    Profile,
    KeyRecovery,
    FAQ,
    Legal,
}

#[derive(Clone, PartialEq)]
pub enum ProfilePageMenu {
    None,
    EditProfile,
    EditBusinessAddress,
}

#[function_component(SettingsPageComponent)]
pub fn settings_page() -> Html {
    let current_page = use_state(|| SettingsPage::Profile);

    html! {
        <div class="h-full w-full flex flex-col">
            <h2 class="text-4xl mb-6 p-4">{"Settings"}</h2>
            
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
                        <button 
                            onclick={let page = current_page.clone();
                                Callback::from(move |_| page.set(SettingsPage::FAQ))}
                            class="text-left p-2 hover:bg-gray-100 rounded">
                            {"FAQ"}
                        </button>
                        <button 
                            onclick={let page = current_page.clone();
                                Callback::from(move |_| page.set(SettingsPage::Legal))}
                            class="text-left p-2 hover:bg-gray-100 rounded">
                            {"Legal"}
                        </button>
                    </div>
                </div>

                // Content Area
                <div class="flex-1 p-4">
                    {match *current_page {
                        SettingsPage::Profile => html! {
                            <ProfileSettingsSection />
                        },
                        SettingsPage::KeyRecovery => html! {
                            <KeyRecoverySection />
                        },
                        SettingsPage::FAQ => html! {
                            <FAQSection />
                        },
                        SettingsPage::Legal => html! {
                            <LegalSection />
                        },
                    }}
                </div>
            </div>
        </div>
    }
}

// Key Recovery Section
#[function_component(KeyRecoverySection)]
fn key_recovery_section() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrIdStore found");
    let keys = key_ctx.get_nostr_key().expect("No keys found");
    
    // Convert secret key bytes to hex string
    let secret_key_bytes = keys.get_secret_key();
    let secret_key_hex: String = secret_key_bytes.iter()
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

// FAQ Section
#[function_component(FAQSection)]
fn faq_section() -> Html {
    html! {
        <div class="flex flex-col gap-4">
            <h2 class="text-2xl font-bold">{"Frequently Asked Questions"}</h2>
            <div class="space-y-4">
                <div class="bg-white p-4 rounded-lg shadow">
                    <h3 class="font-bold mb-2">{"How do I manage my products?"}</h3>
                    <p>{"Use the Products section to add, edit, or remove items from your menu."}</p>
                </div>
                <div class="bg-white p-4 rounded-lg shadow">
                    <h3 class="font-bold mb-2">{"How do orders work?"}</h3>
                    <p>{"You'll receive orders in real-time. Accept them to begin processing."}</p>
                </div>
            </div>
        </div>
    }
}

// Legal Section
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

#[function_component(ProfileSettingsSection)]
fn profile_settings() -> Html {
    let menu_state = use_state(|| ProfilePageMenu::None);
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let profile = user_ctx.profile();

    match *menu_state {
        ProfilePageMenu::None => {
            if let Some(profile) = profile {
                html! {
                    <div class="flex flex-col gap-8">
                        <MyContactDetails handle={menu_state.clone()} profile={Some(profile.clone())}/>
                        <MyBusinessAddress handle={menu_state.clone()} profile={Some(profile.clone())}/>
                    </div>
                }
            } else {
                html! {
                    <div>{"No profile found"}</div>
                }
            }
        },
        ProfilePageMenu::EditProfile => html! {
            <EditProfileMenu 
                handle={menu_state.clone()} 
                profile={profile}
            />
        },
        ProfilePageMenu::EditBusinessAddress => html! {
            <EditAddressMenu handle={menu_state.clone()} />
        },
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct MenuProps {
    pub handle: UseStateHandle<ProfilePageMenu>,
    #[prop_or_default]
    pub profile: Option<CommerceProfile>,
}

#[function_component(MyContactDetails)]
fn my_contact_details(props: &MenuProps) -> Html {
    let MenuProps { handle, profile } = props;
    
    // First ensure we have a profile
    if let Some(profile) = profile {
        html! {
            <div class="w-full flex flex-col gap-2">
                <div class="flex flex-row justify-between items-center">
                    <h3 class="font-bold">{"Contact Details"}</h3>
                    <button
                        onclick={let handle = handle.clone(); 
                            Callback::from(move |_| handle.set(ProfilePageMenu::EditProfile))}
                        class="text-sm text-fuente">{"Edit"}
                    </button>
                </div>
                <CardComponent>
                    <CommerceProfileDetails commerce_data={profile.clone()} />
                </CardComponent>
            </div>
        }
    } else {
        html! {}
    }
}

#[function_component(MyBusinessAddress)]
fn my_business_address(props: &MenuProps) -> Html {
    let MenuProps { handle, profile } = props;
    
    // First ensure we have a profile
    if let Some(profile) = profile {
        html! {
            <div class="w-full flex flex-col gap-2">
                <div class="flex flex-row justify-between items-center">
                    <h3 class="font-bold">{"Business Address"}</h3>
                    <button
                        onclick={let handle = handle.clone(); 
                            Callback::from(move |_| handle.set(ProfilePageMenu::EditBusinessAddress))}
                        class="text-sm text-fuente">{"Edit"}
                    </button>
                </div>
                <CardComponent>
                    <CommerceProfileAddressDetails commerce_data={profile.clone()} />
                </CardComponent>
            </div>
        }
    } else {
        html! {}
    }
}

#[function_component(EditProfileMenu)]
pub fn edit_profile_menu(props: &MenuProps) -> Html {
    let MenuProps { handle, profile: _ } = props;
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let profile = user_ctx.profile().expect("No user profile found");
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();
    let handle = handle.clone();

    let coordinate_state = use_state(|| None);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
    let props = props!(NewAddressProps {
        coord_handle: coordinate_state.clone(),
        nominatim_handle: nominatim_state.clone(),
        map_handle: map_state,
        marker_handle: marker_state,
        onclick: Callback::from(move |_: MouseEvent| {}),
    });

    let coords = (*coordinate_state).clone();
    let address = (*nominatim_state).clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let user_keys = keys.clone();
        let handle = handle.clone();
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
        handle.set(ProfilePageMenu::None);
    });
    let details_card_state = use_state(|| false);
    let address_card_state = use_state(|| false);
    html! {
        <form {onsubmit}
            class="w-full h-full flex flex-col gap-4 overflow-y-scroll p-8">
            <div class="flex flex-row w-full justify-between items-center pr-4">
                <h3 class="font-bold">{"Edit Profile"}</h3>
                <button
                    type="submit"
                    class="text-sm bg-fuente text-white font-bold p-2 px-4 rounded-3xl"
                    >{"Save"}</button>
            </div>
            <DrawerSection title={"Edit Details"} open={details_card_state.clone()}>
                <NewAddressInputs commerce_data={profile.clone()} />
            </DrawerSection>
            <DrawerSection title={"Edit Address"} open={address_card_state.clone()}>
                <NewAddressForm ..props />
            </DrawerSection>
        </form>
    }
}

#[function_component(NewAddressInputs)]
pub fn new_address_inputs(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
        <div class="flex flex-col px-4 gap-2">
            <SimpleInput
                id="name"
                name="name"
                label="Name"
                value={commerce_data.name().to_string()}
                input_type="text"
                required={true}
            />
            <SimpleInput
                id="telephone"
                name="telephone"
                label="Telephone"
                value={commerce_data.telephone().to_string()}
                input_type="tel"
                required={true}
            />
            <SimpleInput
                id="web"
                name="web"
                label="Web"
                value={commerce_data.web().to_string()}
                input_type="text"
                required={true}
            />
            <SimpleInput
                id="ln_address"
                name="ln_address"
                label="Lightning Address"
                value={commerce_data.ln_address().0.to_string()}
                input_type="text"
                required={true}
            />
            <SimpleTextArea
                id="description"
                name="description"
                label="Description"
                value={commerce_data.description().to_string()}
                input_type="text"
                required={true}
            />
        </div>
    }
}

#[function_component(EditAddressMenu)]
pub fn edit_address_menu(props: &MenuProps) -> Html {
    let MenuProps { handle, .. } = props;
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");

    let coordinate_state = use_state(|| None);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);

    let coords = (*coordinate_state).clone();
    let address = (*nominatim_state).clone();
    let onclick = {
        let handle = handle.clone();
        Callback::from(move |_| {
            if let (Some(_address), Some(_coords), Some(_keys)) = (
                address.clone(),
                coords.clone(),
                key_ctx.get_nostr_key(),
            ) {
                handle.set(ProfilePageMenu::None);
            }
        })
    };

    let props = props!(NewAddressProps {
        coord_handle: coordinate_state.clone(),
        nominatim_handle: nominatim_state.clone(),
        map_handle: map_state,
        marker_handle: marker_state,
        onclick,
    });

    html! {
        <>
            <NewAddressForm ..props />
        </>
    }
}