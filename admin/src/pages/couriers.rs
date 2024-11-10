use fuente::{
    browser_api::HtmlForm,
    contexts::{NostrIdStore, NostrProps},
    mass::{SimpleFormButton, SimpleInput},
    models::{AdminConfigurationType, AdminServerRequest},
};
use yew::prelude::*;

use crate::ServerConfigsStore;

#[function_component(CourierWhitelistForm)]
pub fn courier_whitelist_form() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let sender = relay_ctx.send_note.clone();
    let keys = user_ctx.get_nostr_key();
    let commerce_whitelist = config_ctx.get_couriers_whitelist();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = HtmlForm::new(e).expect("Failed to get form element");
        let courier_id = form
            .input_value("courier_id")
            .expect("Failed to get commerce id");
        let keys = keys.clone().expect("No keys found");
        let mut new_whitelist = commerce_whitelist.clone();
        new_whitelist.push(courier_id);
        let admin_request = AdminServerRequest::new(
            AdminConfigurationType::CourierWhitelist,
            serde_json::to_string(&new_whitelist).unwrap(),
        );
        let signed_request = admin_request
            .sign_data(&keys)
            .expect("Failed to sign request");
        sender.emit(signed_request);
    });
    html! {
        <form {onsubmit}
            class="flex flex-col gap-8 p-8 items-center">
            <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Courier Whitelist"}</h2>
            <SimpleInput
                id="courier_id"
                name="courier_id"
                label="Courier ID"
                value=""
                input_type="text"
                required={true}
                />
            <SimpleFormButton>
                {"Add Courier"}
            </SimpleFormButton>
        </form>
    }
}
