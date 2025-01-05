use fuente::{
    contexts::LanguageConfigsStore,
    mass::{DriverDetailsComponent, SimpleInput},
    models::{AdminConfigurationType, AdminServerRequest},
};
use lucide_yew::Trash;
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use yew::prelude::*;

use crate::ServerConfigsStore;

#[function_component(CourierWhitelistPage)]
pub fn exchange_rate_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    html! {
        <>
        <div class="container mx-auto lg:py-10 flex flex-col lg:flex-row items-center lg:justify-between">
            <h3 class="text-fuente text-4xl pb-10 lg:pb-0 text-center lg:text-left lg:text-6xl font-bold tracking-tighter">
                {&translations["admin_settings_title_couriers"]}
            </h3>
        </div>
        <main class="container mx-auto flex-grow">
            <div class="flex flex-col lg:flex-row gap-5 lg:gap-10">
                <div class="flex-1">
                    <CourierWhitelistForm />
                </div>
                <div class="flex-1">
                    <CourierWhitelistProfiles />
                </div>
            </div>
        </main>
        </>
    }
}

#[function_component(CourierWhitelistProfiles)]
pub fn courier_whitelist_display() -> Html {
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let sender = relay_ctx.send_note.clone();
    let keys = key_ctx.get_nostr_key().expect("No keys found");
    let wl_profiles = config_ctx.get_whitelisted_couriers();
    let profiles = wl_profiles.iter().filter_map(|profile| {
        let pubkey = profile.0.pubkey.clone();
        Some((pubkey, profile.1.clone()))
    });
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    html! {
        <div class="flex flex-col gap-4 p-8 items-center w-full">
            <h3 class="text-2xl font-bold text-fuente">{&translations["admin_settings_couriers_list"]}</h3>
            <div class="flex flex-col gap-2 w-full">
                {for profiles.map(|(pubkey, driver)| {
                    let key_clone = keys.clone();
                    let wl_clone = wl_profiles.clone();
                    let sender = sender.clone();
                    let id_clone = pubkey.clone();
                    let remove_onclick = Callback::from(move |_: MouseEvent| {
                        let mut new_whitelist = wl_clone.clone();
                        new_whitelist.retain(|wl_id| *wl_id.0.pubkey != id_clone);
                        let admin_request = AdminServerRequest::new(
                            AdminConfigurationType::CourierWhitelist,
                            serde_json::to_string(&new_whitelist).unwrap(),
                        );
                        let signed_request = admin_request
                            .sign_data(&key_clone)
                            .expect("Failed to sign request");
                        sender.emit(signed_request);
                    });
                    html! {
                        <div class="flex flex-row gap-4 w-full">
                            <DriverDetailsComponent {pubkey} {driver} />
                            <button onclick={remove_onclick} >
                                <Trash class="w-6 h-6 text-red-500" />
                            </button>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

#[function_component(CourierWhitelistForm)]
pub fn courier_whitelist_form() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
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
            class="flex flex-col gap-2 p-8 items-center">
            <h3 class="text-2xl font-bold text-fuente">{&translations["admin_settings_couriers_new"]}</h3>
            <SimpleInput
                id="courier_id"
                name="courier_id"
                label="Courier ID"
                value=""
                input_type="text"
                required={true}
                />
            <button type="submit"
                    class="bg-fuente-orange text-white text-center text-lg font-bold rounded-full w-fit px-8 py-3 mt-5" >
                    {&translations["admin_settings_submit"]}
            </button>
        </form>
    }
}
