use fuente::{
    mass::{DriverDetailsComponent, SimpleFormButton, SimpleInput},
    models::{AdminConfigurationType, AdminServerRequest, DriverProfile, DRIVER_HUB_PRIV_KEY},
};
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::{notes::NostrNote, keypair::NostrKeypair};
use yew::prelude::*;

use crate::ServerConfigsStore;

#[function_component(CourierWhitelistPage)]
pub fn courier_whitelist_page() -> Html {
    html! {
        <div class="flex flex-row gap-4 p-8 items-center">
            <CourierWhitelistForm />
            <div class="flex flex-col gap-4">
            <CourierWhitelistDisplay />
            <CourierWhitelistProfiles />
            </div>
        </div>
    }
}

#[function_component(CourierWhitelistDisplay)]
pub fn courier_whitelist_display() -> Html {
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let sender = relay_ctx.send_note.clone();
    let keys = key_ctx.get_nostr_key().expect("No keys found");
    let wl = config_ctx.get_couriers_whitelist();
    html! {
        <>
            <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Courier Whitelist"}</h2>
            <div class="flex flex-col gap-4">
                {for wl.iter().map(|id| {
                    let key_clone = keys.clone();
                    let wl_clone = wl.clone();
                    let sender = sender.clone();
                    let id_clone = id.clone();
                   let remove_onclick = Callback::from(move |_: MouseEvent| {
                       let mut new_whitelist = wl_clone.clone();
                       new_whitelist.retain(|wl_id| *wl_id != id_clone);
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
                        <div class="flex flex-row gap-4">
                            <p>{id}</p>
                            <button onclick={remove_onclick} class="bg-red-500 text-white rounded-lg px-4 py-2">
                                {"Remove"}
                            </button>
                        </div>
                    }
                })}
            </div>
        </>
    }
}

#[function_component(CourierWhitelistProfiles)]
pub fn courier_whitelist_display() -> Html {
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let wl_profiles = config_ctx.get_whitelisted_couriers();
    let driver_keys = NostrKeypair::new(DRIVER_HUB_PRIV_KEY).unwrap();
    let profiles = wl_profiles.iter().filter_map(|profile| {
        let pubkey = profile.pubkey.clone();
        let cleartext = driver_keys.decrypt_nip_04_content(&profile).unwrap();
        let giftwrapped: NostrNote = cleartext.try_into().unwrap();
        return match DriverProfile::try_from(giftwrapped.clone()) {
            Err(_) => None,
            Ok(profile) => Some((pubkey, profile)),
        };
    });
    html! {
        <>
            <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Courier Profiles"}</h2>
            <div class="flex flex-col gap-4">
                {for profiles.map(|(pubkey, driver)| {
                    html! {
                        <DriverDetailsComponent {pubkey} {driver} />
                    }
                })}
            </div>
        </>
    }
}

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
