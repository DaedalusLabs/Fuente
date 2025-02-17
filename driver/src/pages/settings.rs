use crate::{
    contexts::{DriverDataAction, DriverDataStore},
    router::DriverRoute,
};
use fuente::{
    contexts::LanguageConfigsStore,
    mass::{
        templates::{KeyRecoverySection, SettingsPageTemplate}, AppLink, LanguageToggle, PopupProps, PopupSection, SimpleInput
    },
    models::{DriverProfile, DriverProfileIdb, DRIVER_HUB_PUB_KEY},
};
use lucide_yew::{Key, Phone, ScrollText, SquarePen, X};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum SettingsPage {
    Profile,
    KeyRecovery,
    Language,
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
            <div class="flex items-center gap-4">
                <div class="flex justify-center items-center">
                <AppLink<DriverRoute>
                    class="lg:block hidden flex items-center bg-white border-2 border-fuente px-6 py-3 rounded-full text-fuente space-x-2 font-bold text-sm md:text-md lg:text-lg"
                    selected_class=""
                    route={DriverRoute::History} >
                    <span class="hidden lg:block text-lg font-bold text-center">{&translations["profile_address_button_orders"]}</span>
                </AppLink<DriverRoute>>
                <AppLink<DriverRoute>
                    class="block lg:hidden flex items-center bg-white border-2 border-fuente p-2 rounded-xl"
                    selected_class=""
                    route={DriverRoute::History} >
                    <ScrollText class={classes!("h-6", "w-6", "stroke-fuente")} />
                </AppLink<DriverRoute>>
                </div>
            </div>
        }
    };
    let profile_popup_handle = use_state(|| false);
    let edit_button = {
        let profile_popup_handle = profile_popup_handle.clone();
        match *current_page {
            SettingsPage::KeyRecovery => {
                html! {
                    <button
                        type="button" class="absolute right-2 top-2 m-2 flex gap-4 tracking-wide">
                        <span class="text-red-600 font-bold">
                            {&translations["profile_personal_information_delete_account_button"]}
                        </span>
                        <X class={classes!("feather", "feather-plus", "text-red-600","w-6", "h-6")} />
                    </button>
                }
            }
            SettingsPage::Language => {
                html! {}
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
            ]}
            sidebar_options={ vec![
                (translations["profile_address_personal_information_button"].clone(), go_to_profile, if *current_page == SettingsPage::Profile { true } else { false }),
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
                                <NewProfileForm close_handle={profile_popup_handle.clone()} />
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
    let user_ctx = use_context::<DriverDataStore>().expect("No user context found");
    let profile = user_ctx.get_profile().expect("No user profile found");

    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();

    let pubkey = match user_ctx.get_profile_note() {
        Some(note) => note.pubkey[..12].to_string(),
        None => "No public key found".to_string(),
    };

    html! {
        <div class="w-full lg:mt-6 lg:mr-6 flex flex-1 lg:items-center">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-10 h-full p-6 rounded-lg">
                <div class="space-y-6">
                    <h3 class="text-gray-500 text-2xl font-semibold border-b pb-2">
                        {profile.nickname()}
                    </h3>
                    <div class="space-y-4">
                        <div class="flex items-center space-x-3">
                            <Phone class="text-gray-500 w-5 h-5" />
                            <div>
                                <p class="text-gray-500 text-lg font-bold">{&translations["checkout_client_information_heading_phone"]}</p>
                                <p class="text-gray-500">{&profile.telephone()}</p>
                            </div>
                        </div>
                        <div class="flex items-center space-x-3">
                            <Key class="text-gray-500 w-5 h-5" />
                            <div>
                                <p class="text-gray-500 text-lg font-bold">{&translations["admin_courier_details_key"]}</p>
                                <p class="text-gray-600">{&pubkey}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[function_component(NewProfileForm)]
pub fn new_profile_form(props: &PopupProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let user_ctx = use_context::<DriverDataStore>().expect("No CryptoId Context found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let sender = relay_pool.send_note.clone();
    let popup_handle = props.close_handle.clone();
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
            .giftwrapped_data(
                &keys,
                DRIVER_HUB_PUB_KEY.to_string(),
                DRIVER_HUB_PUB_KEY.to_string(),
            )
            .expect("Failed to giftwrap data");

        sender.emit(giftwrap);
        sender.emit(pool_copy);
        user_ctx.dispatch(DriverDataAction::NewProfile(db));
        popup_handle.set(false);
    });

    html! {
        <form {onsubmit}
            class="w-full h-full flex items-center flex-col gap-4 rounded-3xl p-4 bg-fuente-forms">
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
                <button
                    type="submit"
                    class="bg-fuente-light text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl">
                    {"Save"}
                </button>
        </form>
    }
}
