use fuente::{
    contexts::LanguageConfigsStore,
    mass::{
        templates::LoginPageTemplate, SimpleInput,
    },
    models::{CommerceProfile, CommerceProfileIdb},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

use crate::contexts::{CommerceDataAction, CommerceDataStore};

#[function_component(NewProfilePage)]
pub fn new_profile() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    html! {
        <LoginPageTemplate
            heading={translations["auth_register_heading"].clone()}
            sub_heading={translations["auth_register_heading_now"].clone()}
            title={translations["auth_register_title"].clone()}>
                <div class="bg-fuente-forms px-5 rounded-3xl relative z-0">
                    <NewProfileForm />
                </div>
        </LoginPageTemplate>
    }
}

#[function_component(NewProfileForm)]
pub fn edit_profile_menu() -> Html {
    let user_ctx = use_context::<CommerceDataStore>().expect("No user context found");
    let key_ctx = use_context::<NostrIdStore>().expect("No NostrProps found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();

    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();

    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form");
        let user_keys = keys.clone();
        let sender = sender.clone();
        let user_ctx = user_ctx.clone();
        let mut new_profile = CommerceProfile::default();
        new_profile.name = form.input_value("name").expect("Failed to get name");
        new_profile.telephone = form
            .input_value("telephone")
            .expect("Failed to get telephone");
        new_profile.web = form.input_value("web").expect("Failed to get web");
        new_profile.ln_address = form
            .input_value("ln_address")
            .expect("Failed to get ln_address");

        let db = CommerceProfileIdb::new(new_profile.clone(), &user_keys)
            .expect("Failed to create profile");
        let note = db.signed_note();
        sender.emit(note.clone());
        user_ctx.dispatch(CommerceDataAction::UpdateCommerceProfile(db));
    });
    html! {
        <form {onsubmit}
            class="w-full h-full flex flex-col gap-4 py-10">
            <ProfileInputs />
            <div class="space-y-5 flex flex-col mt-5">
                <input
                    type="submit"
                    class="bg-fuente-light p-3 rounded-3xl font-bold text-white hover:cursor-pointer w-2/4 mx-auto whitespace-nowrap"
                    value={translations["auth_register_link_button"].clone()}
                />
            </div>
        </form>
    }
}

#[function_component(ProfileInputs)]
fn profile_inputs() -> Html {
    html! {
        <div class="flex flex-col gap-2">
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
            <SimpleInput
                id="web"
                name="web"
                label="Website"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleInput
                id="ln_address"
                name="ln_address"
                label="Lightning Address"
                value=""
                input_type="text"
                required={true}
                />
        </div>
    }
}
