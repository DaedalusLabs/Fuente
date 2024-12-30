use crate::contexts::CommerceDataStore;
use fuente::contexts::LanguageConfigsStore;
use fuente::mass::{
    templates::SettingsPageTemplate, CommerceProfileAddressDetails, CommerceProfileDetails,
    SimpleInput, SimpleTextArea,
};
use fuente::mass::{CommerceProfileProps, LanguageToggle};
use fuente::models::CommerceProfile;
use lucide_yew::{ScrollText, ShoppingBag, SquarePen, X};
use nostr_minions::key_manager::NostrIdStore;
use yew::prelude::*;

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
    EditBusinessAddress,
}

#[function_component(SettingsPageComponent)]
pub fn settings_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let profile = use_context::<CommerceDataStore>()
        .expect("No CommerceDataStore found")
        .profile();

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
    let my_orders_onclick = {
        let page = current_page.clone();
        Callback::noop()
    };
    let my_orders_button = {
        html! {
            <div class="flex justify-center items-center">
                <button onclick={my_orders_onclick}
                    type="button" class="flex items-center bg-white border-2 border-fuente px-10 py-3 rounded-full text-fuente space-x-2">

                    <ScrollText class={classes!("text-sm", "text-fuente", "scale-x-[-1]", "mr-2")} />

                    <span class="text-lg font-bold text-center">{&translations["profile_address_button_orders"]}</span>
                </button>
            </div>
        }
    };
    let my_store_onclick = {
        let page = current_page.clone();
        Callback::noop()
    };
    let my_store_button = {
        html! {
            <div class="flex justify-center items-center">
                <button onclick={my_store_onclick}
                    type="button" class="flex items-center bg-white border-2 border-fuente px-10 py-3 rounded-full text-fuente space-x-2">

                    <ShoppingBag class={classes!("text-sm", "text-fuente", "scale-x-[-1]", "mr-2")} />

                    <span class="text-lg font-bold text-center">{&translations["admin_store_new_products_heading"]}</span>
                </button>
            </div>
        }
    };
    let edit_button = {
        match *current_page {
            SettingsPage::KeyRecovery => {
                html! {
                    <button type="button" class="flex gap-4 tracking-wide">
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
            _ => {
                html! {
                    <button type="button" class="flex gap-4 tracking-wide">
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
            heading={translations["admin_store_profile_heading"].clone()}
            options={ vec![
                (my_orders_button),
                (my_store_button),
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
                        <MyContactDetails profile={profile.clone()}/>
                    },
                    SettingsPage::Address => html! {
                        <MyBusinessAddress profile={profile.clone()}/>
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
// Key Recovery Section
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

#[derive(Properties, Clone, PartialEq)]
pub struct MenuProps {
    #[prop_or_default]
    pub profile: Option<CommerceProfile>,
}

#[function_component(MyContactDetails)]
fn my_contact_details(props: &MenuProps) -> Html {
    let MenuProps { profile } = props;
    if let Some(profile) = profile {
        html! {
            <CommerceProfileDetails commerce_data={profile.clone()} />
        }
    } else {
        html! {}
    }
}

#[function_component(MyBusinessAddress)]
fn my_business_address(props: &MenuProps) -> Html {
    let MenuProps { profile } = props;
    if let Some(profile) = profile {
        html! {
            <CommerceProfileAddressDetails commerce_data={profile.clone()} />
        }
    } else {
        html! {}
    }
}

// #[function_component(EditProfileMenu)]
// pub fn edit_profile_menu(props: &MenuProps) -> Html {
//     let MenuProps { handle, profile: _ } = props;
//     let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
//     let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
//     let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
//
//     let logo_url = use_state(|| None);
//     let logo_handle = logo_url.clone();
//
//     let banner_url = use_state(|| None);
//     let banner_handle = banner_url.clone();
//
//     let profile = user_ctx.profile().expect("No user profile found");
//     let keys = key_ctx.get_nostr_key().expect("No user keys found");
//     let sender = relay_pool.send_note.clone();
//     let handle = handle.clone();
//
//     let coordinate_state = use_state(|| None);
//     let nominatim_state = use_state(|| None);
//     let map_state = use_state(|| None);
//     let marker_state = use_state(|| None);
//     let props = props!(NewAddressProps {
//         coord_handle: coordinate_state.clone(),
//         nominatim_handle: nominatim_state.clone(),
//         map_handle: map_state,
//         marker_handle: marker_state,
//         onclick: Callback::from(move |_: MouseEvent| {}),
//     });
//
//     let coords = (*coordinate_state).clone();
//     let address = (*nominatim_state).clone();
//     let onsubmit = Callback::from(move |e: SubmitEvent| {
//         e.prevent_default();
//         let form = HtmlForm::new(e).expect("Failed to get form");
//         let user_keys = keys.clone();
//         let handle = handle.clone();
//         let sender = sender.clone();
//         let user_ctx = user_ctx.clone();
//         let new_profile = CommerceProfile::new(
//             form.input_value("name").expect("Failed to get name"),
//             form.textarea_value("description")
//                 .expect("Failed to get description"),
//             form.input_value("telephone")
//                 .expect("Failed to get telephone"),
//             form.input_value("web").expect("Failed to get web"),
//             address.clone().expect("No address found"),
//             coords.clone().expect("No coordinates found"),
//             form.input_value("ln_address")
//                 .expect("Failed to get lightning address"),
//             logo_url.as_ref().cloned().expect("No profile pic found"),
//             banner_url.as_ref().cloned().expect("No banner found"),
//         );
//         let db = CommerceProfileIdb::new(new_profile.clone(), &user_keys)
//             .expect("Failed to create profile");
//         let note = db.signed_note();
//         sender.emit(note.clone());
//         user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
//         handle.set(ProfilePageMenu::None);
//     });
//     let details_card_state = use_state(|| false);
//     let address_card_state = use_state(|| false);
//     let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
//     html! {
//         <form {onsubmit}
//             class="w-full h-full flex flex-col gap-4 overflow-y-scroll p-8">
//             <div class="flex flex-row w-full justify-between items-center pr-4">
//                 <h3 class="font-bold">{"Edit Profile"}</h3>
//                 <button
//                     type="submit"
//                     class="text-sm bg-fuente text-white font-bold p-2 px-4 rounded-3xl"
//                     >{"Save"}</button>
//             </div>
//             <DrawerSection title={"Edit Details"} open={details_card_state.clone()}>
//                 <NewAddressInputs commerce_data={profile.clone()} />
//                 <ImageUploadInput
//                     url_handle={logo_handle} nostr_keys={nostr_keys.clone()}
//                     classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")}
//                     input_id="logo-image-upload"/>
//                 <ImageUploadInput
//                     url_handle={banner_handle} nostr_keys={nostr_keys}
//                     classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")}
//                     input_id="banner-image-upload"/>
//             </DrawerSection>
//             <DrawerSection title={"Edit Address"} open={address_card_state.clone()}>
//                 <NewAddressForm ..props />
//             </DrawerSection>
//         </form>
//     }
// }

#[function_component(NewAddressInputs)]
pub fn new_address_inputs(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps { commerce_data } = props;
    html! {
        <div class="flex flex-col px-4 gap-2">
            <SimpleInput
                id="name"
                name="name"
                label="Name"
                value={commerce_data.name.to_string()}
                input_type="text"
                required={true}
            />
            <SimpleInput
                id="telephone"
                name="telephone"
                label="Telephone"
                value={commerce_data.telephone.to_string()}
                input_type="tel"
                required={true}
            />
            <SimpleInput
                id="web"
                name="web"
                label="Web"
                value={commerce_data.web.to_string()}
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
                value={commerce_data.description.to_string()}
                input_type="text"
                required={true}
            />
        </div>
    }
}

// #[function_component(EditAddressMenu)]
// pub fn edit_address_menu(props: &MenuProps) -> Html {
//     let MenuProps { handle, .. } = props;
//     let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
//
//     let coordinate_state = use_state(|| None);
//     let nominatim_state = use_state(|| None);
//     let map_state = use_state(|| None);
//     let marker_state = use_state(|| None);
//
//     let coords = (*coordinate_state).clone();
//     let address = (*nominatim_state).clone();
//     let onclick = {
//         let handle = handle.clone();
//         Callback::from(move |_| {
//             if let (Some(_address), Some(_coords), Some(_keys)) =
//                 (address.clone(), coords.clone(), key_ctx.get_nostr_key())
//             {
//                 handle.set(ProfilePageMenu::None);
//             }
//         })
//     };
//
//     let props = props!(NewAddressProps {
//         coord_handle: coordinate_state.clone(),
//         nominatim_handle: nominatim_state.clone(),
//         map_handle: map_state,
//         marker_handle: marker_state,
//         onclick,
//     });
//
//     html! {
//         <>
//             <NewAddressForm ..props />
//         </>
//     }
// }
