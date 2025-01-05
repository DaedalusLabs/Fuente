use fuente::{
    contexts::LanguageConfigsStore,
    models::{AdminConfigurationType, AdminServerRequest},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

use crate::ServerConfigsStore;

#[function_component(ExchangeRatePage)]
pub fn exchange_rate_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    html! {
        <>
        <div class="container mx-auto lg:py-10 flex flex-col lg:flex-row items-center lg:justify-between">
            <h3 class="text-fuente text-4xl pb-10 lg:pb-0 text-center lg:text-left lg:text-6xl font-bold tracking-tighter">
                {&translations["admin_settings_title_exchange"]}
            </h3>
        </div>
        <main class="container mx-auto flex-grow">
            <div class="flex flex-col lg:flex-row gap-5 lg:gap-10">
                <div class="flex-1">
                    <ExchangeRateDisplay />
                </div>
                <div class="flex-1">
                    <ExchangeRateForm />
                </div>
            </div>
        </main>
        </>
    }
}

#[function_component(ExchangeRateDisplay)]
pub fn exchange_rate_display() -> Html {
    let server_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let exchange_rate = server_ctx.get_exchange_rate();
    let sat_rate = exchange_rate / 100_000_000.0;
    html! {
        <div class="flex flex-col gap-2 p-8 items-center">
            <h3 class="text-2xl font-bold text-fuente">{"Current Exchange Rate"}</h3>
            <p class="text-xl font-semibold text-fuente">{format!("1 SAT = SRD {}", sat_rate)}</p>
            <p class="text-xl font-semibold text-fuente">{format!("1 BTC = SRD {}", exchange_rate)}</p>
        </div>
    }
}

#[function_component(ExchangeRateForm)]
pub fn exchange_rate_form() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();

    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let sender = relay_ctx.send_note.clone();

    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
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
            class="flex flex-col gap-2 p-8 items-center">
            <input  type="number" id="exchange_rate" name="exchange_rate" label={translations["admin_settings_title_exchange"].clone()}
                    placeholder={translations["admin_settings_btc_srd"].clone()}
                    class="py-3 px-5 rounded-xl border border-gray-500 w-full text-gray-500" step="0.01" value="" required={true} />
            <button type="submit"
                    class="bg-fuente-orange text-white text-center text-lg font-bold rounded-full w-fit px-8 py-3 mt-5" >
                    {&translations["admin_settings_submit"]}
            </button>
        </form>
    }
}
