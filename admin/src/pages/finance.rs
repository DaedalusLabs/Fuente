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
        <main class="container mx-auto overflow-hidden">
            <div class="flex flex-col h-full">
                <div class="flex flex-row justify-between items-center p-4 lg:py-10">
                    <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-0 lg:text-6xl tracking-tighter font-bold font-mplus">
                        {&translations["admin_settings_title_exchange"]}
                    </h1>
                </div>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-5 lg:gap-10 border-2 border-fuente rounded-2xl w-full lg:max-w-5xl mx-auto">
                    <div class="flex-1">
                        <ExchangeRateDisplay />
                    </div>
                    <div class="flex-1">
                        <ExchangeRateForm />
                    </div>
                </div>
            </div>
        </main>
    }
}

#[function_component(ExchangeRateDisplay)]
pub fn exchange_rate_display() -> Html {
    let server_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let exchange_rate = server_ctx.get_exchange_rate();
    let sat_rate = exchange_rate / 100_000_000.0;
    html! {
        <div class="flex flex-col gap-5 p-5 md:max-w-sm lg:max-w-xs mx-auto">
            <div class="space-y-2">
                <p class="text-gray-500 text-lg font-bold">{"Bitcoin Exchange"}</p>
                <p class="text-fuente font-bold text-2xl">{format!("1 BTC = SRD {}", exchange_rate)}</p>
            </div>
            <div class="space-y-2">
                <p class="text-gray-500 text-md"><span class="font-bold">{"Sat Price: "}</span>{ format!("1 SAT = SRD {}", sat_rate) }</p>
            </div>
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
    let keys = user_ctx.clone();

    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let keys = keys.clone();
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let exchange_rate = form_element
            .input_value("exchange_rate")
            .expect("Failed to get exchange rate");
        let admin_request =
            AdminServerRequest::new(AdminConfigurationType::ExchangeRate, exchange_rate);
        let sender = sender.clone();
        yew::platform::spawn_local(async move {
            let signed_request = admin_request
                .sign_data(&keys.get_identity().expect("No identity found"))
                .await
                .expect("Failed to sign request");
            sender.emit(signed_request);
        });
    });

    html! {
        <form {onsubmit}
            class="rounded-2xl bg-white p-5 md:max-w-sm lg:max-w-xs mx-auto">
            <div class="space-y-2">
                <label for="exchange_rate" class="text-gray-500 font-light text-sm">{"Change the Exchange"}</label>
                <input
                    type="number"
                    id="exchange_rate" name="exchange_rate"
                    placeholder={translations["admin_settings_btc_srd"].clone()}
                    class="w-full rounded-lg border-2 border-fuente p-2"
                    step="0.01" value="" required={true} />
                <div class="flex justify-center">
                    <input
                        type="submit"
                        value={translations["admin_settings_submit"].clone()}
                        class="bg-fuente-orange text-center text-white font-bold text-sm py-3 rounded-full w-full md:w-1/2 lg:mx-auto cursor-pointer"
                    />
                </div>
            </div>
        </form>
    }
}
