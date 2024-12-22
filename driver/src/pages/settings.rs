use crate::contexts::{DriverDataAction, DriverDataStore};
use fuente::{
    mass::{SimpleFormButton, SimpleInput},
    models::{DriverProfile, DriverProfileIdb, DRIVER_HUB_PUB_KEY},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum SettingsPage {
    Profile,
    KeyRecovery,
    FAQ,
    Legal,
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
                    <h3 class="font-bold mb-2">{"How do I pick up orders?"}</h3>
                    <p>{"Accept available orders from the dashboard and follow the delivery instructions."}</p>
                </div>
                <div class="bg-white p-4 rounded-lg shadow">
                    <h3 class="font-bold mb-2">{"How do earnings work?"}</h3>
                    <p>{"Earnings are calculated per delivery and paid via Lightning Network."}</p>
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
                
                <h3 class="font-bold mb-2">{"Delivery Guidelines"}</h3>
                <p>{"Our delivery policy and guidelines..."}</p>
            </div>
        </div>
    }
}

#[function_component(ProfileSettingsSection)]
fn profile_settings() -> Html {
    let user_ctx = use_context::<DriverDataStore>().expect("DriverDataStore not found");
    let profile = user_ctx.get_profile();

    match profile {
        Some(profile) => html! {
            <div class="flex flex-col gap-4">
                <h2 class="text-2xl font-bold">{"Profile Settings"}</h2>
                <div class="bg-white p-4 rounded-lg shadow">
                    <div class="flex flex-col gap-4">
                        <div>
                            <label class="font-bold">{"Name"}</label>
                            <p>{profile.nickname()}</p>
                        </div>
                        <div>
                            <label class="font-bold">{"Phone"}</label>
                            <p>{profile.telephone()}</p>
                        </div>
                    </div>
                </div>
            </div>
        },
        None => html! {
            <NewProfileForm />
        }
    }
}

#[function_component(NewProfileForm)]
pub fn new_profile_form() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let user_ctx = use_context::<DriverDataStore>().expect("No CryptoId Context found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let sender = relay_pool.send_note.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let user_ctx = user_ctx.clone();
        let keys = key_ctx.get_nostr_key().expect("No user keys found");
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let nickname = form_element
            .input_value("name")
            .expect("Failed to get name");
        let telephone = form_element
            .input_value("telephone")
            .expect("Failed to get telephone");
        let sender = sender.clone();
        let user_profile = DriverProfile::new(nickname, telephone);
        let db = DriverProfileIdb::new(user_profile.clone(), &keys);
        
        // Fix the giftwrapped_data calls by providing the proper parameters
        let giftwrap = user_profile
            .giftwrapped_data(&keys, keys.public_key(), keys.public_key())
            .expect("Failed to giftwrap data");
        let pool_copy = user_profile
            .giftwrapped_data(&keys, DRIVER_HUB_PUB_KEY.to_string(), DRIVER_HUB_PUB_KEY.to_string())
            .expect("Failed to giftwrap data");
            
        sender.emit(giftwrap);
        sender.emit(pool_copy);
        user_ctx.dispatch(DriverDataAction::NewProfile(db));
    });

    html! {
        <form {onsubmit} class="flex flex-col p-8 gap-8 flex-1 items-center">
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
                <SimpleFormButton>
                    {"Save"}
                </SimpleFormButton>
        </form>
    }
}
