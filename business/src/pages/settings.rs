use crate::contexts::{CommerceDataAction, CommerceDataStore};
use crate::router::CommerceRoute;
use fuente::contexts::LanguageConfigsStore;
use fuente::mass::templates::KeyRecoverySection;
use fuente::mass::{
    templates::SettingsPageTemplate, AppLink, PopupProps, PopupSection, SimpleInput, SimpleTextArea,
};
use fuente::mass::{
    CommerceProfileProps, ImageUploadInput, LanguageToggle, NewAddressForm, NewAddressProps,
};
use fuente::models::CommerceProfileIdb;
use lucide_yew::{
    Compass, Mail, MapPin, Phone, ScrollText, ShoppingBag, SquarePen, Upload, Zap, X,
};
use nostr_minions::browser_api::HtmlForm;
use nostr_minions::key_manager::NostrIdStore;
use nostr_minions::relay_pool::NostrProps;
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
    let _profile = use_context::<CommerceDataStore>()
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
    let my_orders_button = {
        html! {
            <div class="flex items-center gap-4">
                <div class="flex justify-center items-center">
                <AppLink<CommerceRoute>
                    class="lg:block hidden flex items-center bg-fuente-buttons px-6 py-3 rounded-full text-fuente-forms space-x-2 font-bold text-sm md:text-md lg:text-lg"
                    selected_class=""
                    route={CommerceRoute::History} >
                    <span class="hidden lg:block text-lg font-bold text-center">{&translations["profile_address_button_orders"]}</span>
                </AppLink<CommerceRoute>>
                <AppLink<CommerceRoute>
                    class="block lg:hidden flex items-center bg-fuente-buttons p-2 rounded-xl"
                    selected_class=""
                    route={CommerceRoute::History} >
                    <ScrollText class={classes!("h-6", "w-6", "stroke-fuente")} />
                </AppLink<CommerceRoute>>
                </div>
            </div>
        }
    };
    let my_store_button = {
        html! {
            <div class="flex items-center gap-4">
                <div class="flex justify-center items-center">
                <AppLink<CommerceRoute>
                    class="lg:block hidden flex items-center bg-fuente-buttons px-6 py-3 rounded-full text-fuente-forms space-x-2 font-bold text-sm md:text-md lg:text-lg"
                    selected_class=""
                    route={CommerceRoute::Products} >
                    <span class="hidden lg:block text-lg font-bold text-center">{&translations["admin_store_new_products_heading"]}</span>
                </AppLink<CommerceRoute>>
                <AppLink<CommerceRoute>
                    class="block lg:hidden flex items-center bg-fuente-buttons p-2 rounded-xl"
                    selected_class=""
                    route={CommerceRoute::Products} >
                    <ShoppingBag class={classes!("h-6", "w-6", "stroke-fuente")} />
                </AppLink<CommerceRoute>>
                </div>
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
                    <button type="button"
                        class="absolute top-1 right-1 sm:top-4 sm:right-4 p-2 rounded-full transition duration-300">
                        <span class="text-red-600 font-bold text-xl flex gap-1 items-top">
                            {&translations["profile_personal_information_delete_account_button"]}
                            <X class={classes!("feather", "feather-plus", "text-red-600","w-6", "h-6")} />
                        </span>
                    </button>
                }
            }
            SettingsPage::Language => {
                html! {}
            }
            SettingsPage::Address => {
                html! {
                    <button onclick={Callback::from(move |_| address_popup_handle.set(true))}
                        type="button"
                        class="absolute top-1 right-1 sm:top-4 sm:right-4 p-2 rounded-full transition duration-300">
                        <span class="text-fuente font-bold text-xl flex gap-1 items-top">
                            {&translations["profile_personal_information_edit_button"]}
                            <SquarePen class={classes!("feather", "feather-plus", "text-fuente","w-6", "h-6")} />
                        </span>
                    </button>
                }
            }
            SettingsPage::Profile => {
                html! {
                    <button onclick={Callback::from(move |_| profile_popup_handle.set(true))}
                        type="button"
                        class="absolute top-1 right-1 sm:top-4 sm:right-4 p-2 rounded-full transition duration-300">
                        <span class="text-fuente font-bold text-xl flex gap-1 items-top">
                            {&translations["profile_personal_information_edit_button"]}
                            <SquarePen class={classes!("feather", "feather-plus", "text-fuente","w-6", "h-6")} />
                        </span>
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
                (translations["stores_settings_option_information"].clone(), go_to_profile, if *current_page == SettingsPage::Profile { true } else { false }),
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
                            <EditCommerceModal close_handle={profile_popup_handle.clone()} />
                        </PopupSection>
                        </div>
                    },
                    SettingsPage::Address => html! {
                        <div class="w-full">
                        <MyBusinessAddress />
                        <PopupSection close_handle={address_popup_handle.clone()}>
                            <EditAddressModal close_handle={address_popup_handle.clone()} />
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
fn my_contact_details() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();

    let user_ctx = use_context::<CommerceDataStore>().expect("No CommerceDataStore found");
    let profile = user_ctx.profile().expect("No user profile found");

    let logo_popup_handle = use_state(|| false);
    let logo_popup_handle_clone = logo_popup_handle.clone();
    html! {
        <div class="w-full lg:mt-6 lg:mr-6 flex flex-1 lg:items-center">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-10 h-full p-6 rounded-lg">
                <div class="space-y-6">
                    <h3 class="text-gray-800 text-2xl font-semibold border-b pb-2">
                        {&profile.name}
                    </h3>

                  <div class="space-y-4">
                    <div class="flex items-center space-x-3">
                      <Mail class="text-gray-500 w-5 h-5" />
                      <div>
                        <p class="text-gray-700 text-lg font-bold">{&translations["checkout_client_information_heading_email"]}</p>
                        <p class="text-gray-600">{&profile.web}</p>
                      </div>
                    </div>

                    <div class="flex items-center space-x-3">
                      <Phone class="text-gray-500 w-5 h-5" />
                      <div>
                        <p class="text-gray-700 text-lg font-bold">{&translations["checkout_client_information_heading_phone"]}</p>
                        <p class="text-gray-600">{&profile.telephone}</p>
                      </div>
                    </div>

                    <div class="flex items-center space-x-3">
                      <Zap class="text-gray-500 w-5 h-5" />
                      <div>
                        <p class="text-gray-700 text-lg font-bold">{&translations["checkout_client_information_heading_ln_address"]}</p>
                        <p class="text-gray-600">{&profile.ln_address}</p>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="flex flex-col items-center justify-center space-y-6">
                  <div class="relative w-full max-w-xs aspect-square">
                    <img
                      src={profile.logo_url.clone()}
                      alt="Profile Logo"
                      class="border-2 border-dashed border-gray-300 bg-gray-100 rounded-lg object-cover w-full h-full"
                    />
                  </div>

                  <button
                    onclick={Callback::from(move |_| logo_popup_handle_clone.set(true))}
                    class="bg-fuente-buttons text-fuente-forms py-3 rounded-full px-10 font-semibold flex items-center space-x-2 hover:bg-opacity-90 transition duration-300"
                  >
                    <Upload class="w-5 h-5" />
                    <span>{&translations["profile_settings_upload"]}</span>
                  </button>
                </div>
            </div>
            <PopupSection close_handle={logo_popup_handle.clone()}>
                <EditLogoModal close_handle={logo_popup_handle.clone()} />
            </PopupSection>
        </div>
    }
}

#[function_component(EditLogoModal)]
pub fn edit_avatar(props: &PopupProps) -> Html {
    let close_handle = props.close_handle.clone();
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_ctx = use_context::<NostrProps>().expect("No RelayPool Context found");
    let profile = user_ctx.profile().expect("No user profile found");
    let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
    let user_keys = nostr_keys.clone();
    let sender = relay_ctx.send_note.clone();
    let logo_url = use_state(|| None);
    let url_handle = logo_url.clone();
    let url_clone = logo_url.clone();

    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let mut new_profile = profile.clone();
        new_profile.logo_url = (*url_clone)
            .clone()
            .unwrap_or_else(|| new_profile.logo_url.clone());
        let db = CommerceProfileIdb::new(new_profile.clone(), &user_keys)
            .expect("Failed to create profile");
        let note = db.signed_note();
        sender.emit(note.clone());
        user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
        close_handle.set(false);
    });
    html! {
        <form {onsubmit}
            class="w-full flex flex-col gap-2 bg-fuente-dark rounded-3xl p-4 items-center">
            <ImageUploadInput {url_handle} {nostr_keys} classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")} input_id="user-profile-upload" />
            <button
                type="submit"
                class="bg-fuente text-sm text-white font-bold p-2 rounded-3xl px-4 w-fit shadow-xl"
                >{"Save"}
            </button>
        </form>
    }
}
#[function_component(MyBusinessAddress)]
fn my_business_address() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No NostrProps found");
    let translations = language_ctx.translations();
    let user_ctx = use_context::<CommerceDataStore>().expect("No CommerceDataStore found");
    let profile = user_ctx.profile().expect("No user profile found");
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
                    {&profile.lookup.display_name()}
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
                    {format!("Latitude: {:.2} Longitude: {:.2}", &profile.lookup.lat_as_f64(), &profile.lookup.long_as_f64())}
                  </p>
                </div>
              </div>
            </div>
        </div>
    }
}

#[function_component(EditCommerceModal)]
pub fn edit_profile_menu(props: &PopupProps) -> Html {
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let profile = user_ctx.profile().expect("No user profile found");
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();
    let handle = props.close_handle.clone();
    let profile_clone = profile.clone();

    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let mut new_profile = profile_clone.clone();
        new_profile.name = form.input_value("name").expect("Failed to get name");
        new_profile.telephone = form
            .input_value("telephone")
            .expect("Failed to get telephone");
        new_profile.web = form.input_value("web").expect("Failed to get web");
        let db =
            CommerceProfileIdb::new(new_profile.clone(), &keys).expect("Failed to create profile");
        let note = db.signed_note();
        sender.emit(note.clone());
        user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
        handle.set(false);
    });
    html! {
        <form {onsubmit}
            class="w-full h-full flex flex-col gap-4 rounded-3xl p-4 bg-fuente-dark">
                <EditProfileInputs commerce_data={profile.clone()} />
                <button
                    type="submit"
                    class="text-sm bg-fuente text-white font-bold p-2 px-4 rounded-3xl"
                    >{"Save"}</button>
        </form>
    }
}

#[function_component(EditProfileInputs)]
pub fn new_address_inputs(props: &CommerceProfileProps) -> Html {
    let CommerceProfileProps {
        commerce_data,
        rating: _,
    } = props;
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

#[function_component(EditAddressModal)]
pub fn edit_profile_menu(props: &PopupProps) -> Html {
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let profile = user_ctx.profile().expect("No user profile found");
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();
    let handle = props.close_handle.clone();

    let coordinate_state = use_state(|| None);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
    let props = yew::props!(NewAddressProps {
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
        let mut profile_clone = profile.clone();
        profile_clone.lookup = address.clone().expect("No address found");
        profile_clone.geolocation = coords.clone().expect("No coordinates found").into();
        let db = CommerceProfileIdb::new(profile_clone.clone(), &keys)
            .expect("Failed to create profile");
        let note = db.signed_note();
        sender.emit(note.clone());
        user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
        handle.set(false);
    });
    html! {
        <form  class="bg-fuente-dark rounded-3xl p-8 max-w-sm sm:max-w-md md:max-w-lg lg:max-w-xl xl:max-w-2xl" {onsubmit}>
            <NewAddressForm ..props />
        </form>
    }
}
