use fuente::{
    mass::SimpleFormButton,
    models::{AdminConfigurationType, AdminServerRequest},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

use crate::ServerConfigsStore;

#[function_component(ExchangeRatePage)]
pub fn exchange_rate_page() -> Html {
    html! {
        <div class="w-full h-full flex flex-col gap-8 p-8 items-center">
            <ExchangeRateDisplay />
            <ExchangeRateForm />
        </div>
    }
}

#[function_component(ExchangeRateDisplay)]
pub fn exchange_rate_display() -> Html {
    let server_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let exchange_rate = server_ctx.get_exchange_rate();
    let sat_rate = exchange_rate / 100_000_000.0;
    html! {
        <>
            <h1>{"Current Exchange Rate"}</h1>
            <p>{format!("1 BTC = {} SRD", exchange_rate)}</p>
            <p>{format!("1 Satoshi = {} SRD", sat_rate)}</p>
        </>
    }
}

#[function_component(ExchangeRateForm)]
pub fn exchange_rate_form() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");

    let sender = relay_ctx.send_note.clone();
    let keys = user_ctx.get_nostr_key();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let keys = keys.clone().expect("No keys found");
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let exchange_rate = form_element
            .input_value("exchange_rate")
            .expect("Failed to get exchange rate");
        let admin_request =
            AdminServerRequest::new(AdminConfigurationType::ExchangeRate, exchange_rate);
        let signed_request = admin_request
            .sign_data(&keys)
            .expect("Failed to sign request");
        sender.emit(signed_request);
    });
    html! {
        <form {onsubmit}
            class="flex flex-col gap-8 p-8 items-center">
            <input  type="number" id="exchange_rate" name="exchange_rate" label="Exchange Rate"
                    step="0.01" value="" required={true} />
            <SimpleFormButton >
                {"Set Exchange Rate"}
            </SimpleFormButton>
        </form>
    }
}
