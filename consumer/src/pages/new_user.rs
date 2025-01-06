use fuente::{
    contexts::LanguageConfigsStore,
    mass::{templates::LoginPageTemplate, ImageUploadInput, NewAddressForm, NewAddressProps},
    models::{
        ConsumerAddress, ConsumerAddressIdb, ConsumerProfile, ConsumerProfileIdb, TEST_PUB_KEY,
    },
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

use crate::contexts::{ConsumerDataAction, ConsumerDataStore};

#[function_component(NewProfilePage)]
pub fn new_profile() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    html! {
        <LoginPageTemplate
            heading={translations["auth_register_heading"].clone()}
            sub_heading={translations["auth_register_heading_now"].clone()}
            title={translations["auth_register_title"].clone()}>
                    <NewProfileForm />
        </LoginPageTemplate>
    }
}

#[function_component(NewAddressPage)]
pub fn new_profile() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let user_ctx = use_context::<ConsumerDataStore>().expect("ConsumerDataStore not found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    let coordinate_state = use_state(|| None::<nostr_minions::browser_api::GeolocationCoordinates>);
    let nominatim_state = use_state(|| None);
    let map_state = use_state(|| None);
    let marker_state = use_state(|| None);
    let onclick = {
        let nominatim = nominatim_state.clone();
        let coordinate = coordinate_state.clone();
        Callback::from(move |_: MouseEvent| {
            if let (Some(address), Some(coords), Some(keys)) = (
                (*nominatim).clone(),
                (*coordinate).clone(),
                key_ctx.get_nostr_key(),
            ) {
                let address = ConsumerAddress::new(address, coords.into());
                let mut db_entry = ConsumerAddressIdb::new(address.clone(), &keys);
                db_entry.set_default(true);
                user_ctx.dispatch(ConsumerDataAction::NewAddress(db_entry));
            }
        })
    };
    let props = yew::props!(NewAddressProps {
        map_handle: map_state,
        marker_handle: marker_state,
        coord_handle: coordinate_state.clone(),
        nominatim_handle: nominatim_state.clone(),
        onclick,
    });
    html! {
        <LoginPageTemplate
            heading={translations["auth_register_heading"].clone()}
            sub_heading={translations["auth_register_heading_now"].clone()}
            title={translations["auth_register_title"].clone()}>
                <div class="bg-fuente-forms  w-fit p-4 rounded-3xl relative z-0  text-white mx-auto my-auto">
                    <NewAddressForm ..props />
                </div>
        </LoginPageTemplate>
    }
}
#[function_component(NewProfileForm)]
pub fn new_profile_form() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    let key_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let user_ctx = use_context::<ConsumerDataStore>().expect("No CryptoId Context found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let profile_pic_url = use_state(|| None);
    let url_handle = profile_pic_url.clone();

    let sender = relay_pool.send_note.clone();
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let user_ctx = user_ctx.clone();
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let nickname = form_element
            .input_value("name")
            .expect("Failed to get name");
        let email = form_element
            .input_value("email")
            .expect("Failed to get email");
        let telephone = form_element
            .input_value("phone")
            .expect("Failed to get telephone");

        let sender = sender.clone();
        let user_profile = ConsumerProfile::new(
            nickname,
            email,
            telephone,
            profile_pic_url.as_ref().cloned(),
        );
        let db = ConsumerProfileIdb::new(user_profile.clone(), &keys);
        let giftwrap = user_profile
            .giftwrapped_data(&keys, keys.public_key())
            .expect("Failed to giftwrap data");
        let server_registry = user_profile
            .registry_data(&keys, TEST_PUB_KEY.to_string())
            .expect("Failed to giftwrap data");
        sender.emit(giftwrap);
        sender.emit(server_registry);
        user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
    });
    let nostr_keys = key_ctx.get_nostr_key().expect("No user keys found");
    html! {
        <form {onsubmit}
            class="bg-fuente-forms py-[65px] px-5 rounded-3xl relative z-0">
            <div class="space-y-5">
                <div class="space-y-1">
                    <label
                        for="username"
                        class="text-white text-lg block text-left"
                    >
                        {&translations["auth_register_form_label_username"]}
                    </label>
                    <input
                        id="name"
                        name="name"
                        type="text"
                        class="p-3 w-full rounded-xl"
                    />
                </div>

                <div class="space-y-1">
                    <label
                        for="email"
                        class="text-white text-lg block text-left"
                    >{&translations["auth_register_form_label_email"]}
                    </label>
                    <input
                        id="email"
                        name="email"
                        type="email"
                        class="p-3 w-full rounded-xl"
                    />
                </div>

                <div class="space-y-1">
                    <label
                        for="phone"
                        class="text-white text-lg block text-left"
                    >{&translations["auth_register_form_label_phone"]}
                    </label>
                    <input
                        id="phone"
                        name="phone"
                        type="tel"
                        class="p-3 w-full rounded-xl"
                    />
                </div>
                <ImageUploadInput {url_handle} {nostr_keys} classes={classes!("min-w-32", "min-h-32", "h-32", "w-32")} input_id="new-user-image-upload"/>
            </div>

            <div class="space-y-5 flex flex-col mt-5">
                <input
                    type="submit"
                    class="bg-fuente-buttons p-3 rounded-3xl font-bold text-fuente hover:cursor-pointer w-2/4 mx-auto whitespace-normal"
                    value={translations["auth_register_link_button"].clone()}
                />
            </div>
        </form>
    }
}
