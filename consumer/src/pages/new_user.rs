use fuente::{
    browser_api::HtmlForm,
    contexts::{NostrIdStore, NostrProps},
    mass::{SimpleFormButton, SimpleInput},
    models::{ConsumerProfile, ConsumerProfileIdb},
};
use yew::prelude::*;

use crate::contexts::{ConsumerDataAction, ConsumerDataStore};

#[function_component(NewProfilePage)]
pub fn new_profile() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let user_ctx = use_context::<ConsumerDataStore>().expect("No CryptoId Context found");
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
        let email = form_element
            .input_value("email")
            .expect("Failed to get email");
        let telephone = form_element
            .input_value("telephone")
            .expect("Failed to get telephone");
        let sender = sender.clone();
        let user_profile = ConsumerProfile::new(nickname, email, telephone);
        let db = ConsumerProfileIdb::new(user_profile.clone(), &keys);
        let giftwrap = user_profile
            .giftwrapped_data(&keys, keys.get_public_key())
            .expect("Failed to giftwrap data");
        sender.emit(giftwrap);
        user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
    });

    html! {
        <form {onsubmit} class="flex flex-col gap-8 p-8 flex-1 items-center">
                <SimpleInput
                    id="name"
                    name="name"
                    label="Name"
                    value=""
                    input_type="text"
                    required={true}
                    />
                <SimpleInput
                    id="email"
                    name="email"
                    label="Email"
                    value=""
                    input_type="email"
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
