use fuente::{
    contexts::LanguageConfigsStore,
    mass::LoadingScreen,
    models::{AdminConfigurationType, AdminServerRequest, DriverProfile},
};
use lucide_yew::Trash;
use nostr_minions::{browser_api::HtmlForm, key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::notes::NostrNote;
use yew::prelude::*;

use crate::{ServerConfigsAction, ServerConfigsStore};

#[function_component(CourierWhitelistPage)]
pub fn exchange_rate_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    html! {
        <main class="container mx-auto overflow-hidden">
            <div class="flex flex-col h-full">
                <div class="flex flex-row justify-between items-center p-4 lg:py-10">
                    <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-0 lg:text-6xl tracking-tighter font-bold font-mplus">
                        {&translations["admin_settings_title_couriers"]}
                    </h1>
                </div>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-5 items-center">
                    <CourierWhitelistForm />
                    <CourierWhitelistProfiles />
                </div>
            </div>
        </main>
    }
}

#[function_component(CourierWhitelistProfiles)]
pub fn courier_whitelist_display() -> Html {
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    if config_ctx.loading() {
        return html! {
            <LoadingScreen />
        };
    }

    let sender = relay_ctx.send_note.clone();
    let keys = key_ctx.get_identity().cloned().expect("No keys found");
    let wl_profiles = config_ctx.get_whitelisted_couriers();
    let whitelist = config_ctx.get_couriers_whitelist();
    
    // Get all courier profiles, even those not in whitelist
    let all_couriers = config_ctx.get_all_couriers();
    
    // Identify which couriers are not in the whitelist (deleted)
    let deleted_couriers: Vec<(NostrNote, DriverProfile)> = all_couriers
        .iter()
        .filter(|(note, _)| !whitelist.contains(&note.pubkey))
        .cloned()
        .collect();

    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    html! {
        <div class="flex flex-col gap-4 overflow-hidden items-center w-full">
            <div class="flex-1 overflow-hidden items-center w-full">
                <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                    <div class="flex flex-col gap-2 w-full h-full overflow-hidden max-w-xl">
                        <div class="grid grid-flow-col justify-stretch gap-2 w-full">
                            <div
                                class={classes!("border-green-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                                <p class={classes!("text-lg", "font-semibold", "text-center", "text-green-500")}>
                                    {&translations["admin_settings_couriers_list"]}
                                </p>
                            </div>
                        </div>
                        <div
                            class={classes!("flex-1", "rounded-2xl", "mt-2", "px-2", "py-2", "overflow-y-auto", "no-scrollbar", "bg-green-100")}>
                            <div class="grid grid-cols-1 gap-4">
                            {
                                if whitelist.is_empty() {
                                    html! {
                                        <div class="flex justify-center items-center p-4">
                                            <p class="text-gray-500">{"No active couriers"}</p>
                                        </div>
                                    }
                                } else {
                                    whitelist.iter().map(|key| {
                                        html! {
                                          <div class="flex flex-row justify-between items-center p-2 bg-white rounded-2xl">
                                            <div class="flex flex-row gap-2">
                                              <div class="flex flex-col gap-2">
                                                <p class="text-fuente font-bold text-lg">{"Courier"}</p>
                                                <p class="text-gray-500 font-mono text-sm break-all">{key}</p>
                                              </div>
                                            </div>
                                            <div class="flex flex-row gap-2">

                                            <button
                                                onclick={
                                                    let sender = sender.clone();
                                                    let keys = keys.clone();
                                                    let wl = whitelist.clone();
                                                    let courier_key = key.clone();
                                                    let config_ctx = config_ctx.clone();
                                                    Callback::from(move |_| {
                                                        // Remove from whitelist
                                                        let mut new_whitelist = wl.clone();
                                                        new_whitelist.retain(|p| p != &courier_key);

                                                        gloo::console::log!("Deleting courier, new whitelist:", format!("{:?}", new_whitelist));

                                                        // Dispatch updates to context
                                                        config_ctx.dispatch(ServerConfigsAction::UpdateCouriersWhitelist(new_whitelist.clone()));

                                                        // Send whitelist to server
                                                        let admin_request = AdminServerRequest::new(
                                                            AdminConfigurationType::CourierWhitelist,
                                                            serde_json::to_string(&new_whitelist).unwrap(),
                                                        );

                                                        let sender = sender.clone();
                                                        let keys = keys.clone();
                                                        yew::platform::spawn_local(async move {
                                                            match admin_request.sign_data(&keys).await {
                                                                Ok(signed_request) => {
                                                                    gloo::console::log!("Sent whitelist update to server");
                                                                    sender.emit(signed_request);
                                                                },
                                                                Err(e) => {
                                                                    gloo::console::error!("Failed to sign request:", e.to_string());
                                                                }
                                                            }
                                                        });
                                                    })
                                                }
                                                class="bg-red-500 text-white font-bold text-sm py-2 px-4 rounded-full"
                                            >
                                                <Trash class="w-4 h-4" />
                                            </button>
                                            </div>
                                          </div>
                                        }
                                    }).collect::<Html>()
                                }
                            }
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="flex-1 overflow-hidden items-center w-full mt-4">
                <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                    <div class="flex flex-col gap-2 w-full h-full overflow-hidden max-w-xl">
                        <div class="grid grid-flow-col justify-stretch gap-2 w-full">
                            <div
                                class={classes!("border-red-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                                <p class={classes!("text-lg", "font-semibold", "text-center", "text-red-500")}>
                                    {"Deleted Couriers"}
                                </p>
                            </div>
                        </div>
                        <div
                            class={classes!("flex-1", "rounded-2xl", "mt-2", "px-2", "py-2", "overflow-y-auto", "no-scrollbar", "bg-red-100")}>
                            <div class="grid grid-cols-1 gap-4">
                            {
                                if deleted_couriers.is_empty() {
                                    html! {
                                        <div class="flex justify-center items-center p-4">
                                            <p class="text-gray-500">{"No deleted couriers"}</p>
                                        </div>
                                    }
                                } else {
                                    deleted_couriers.iter().map(|(note, profile)| {
                                        html! {
                                          <div class="flex flex-row justify-between items-center p-2 bg-white rounded-2xl">
                                            <div class="flex flex-row gap-2">
                                              <div class="flex flex-col gap-2">
                                                <p class="text-red-500 font-bold text-lg">{"Deleted Courier"}</p>
                                                <p class="text-gray-500 font-light text-lg">{profile.nickname()}</p>
                                                <p class="text-gray-500 font-mono text-sm break-all">{&note.pubkey}</p>
                                              </div>
                                            </div>
                                            <div class="flex flex-row gap-2">
                                                <button
                                                    onclick={
                                                        let sender = sender.clone();
                                                        let keys = keys.clone();
                                                        let wl = whitelist.clone();
                                                        let courier_key = note.pubkey.clone();
                                                        let config_ctx = config_ctx.clone();
                                                        Callback::from(move |_| {
                                                            let mut new_whitelist = wl.clone();
                                                            if !new_whitelist.contains(&courier_key) {
                                                                new_whitelist.push(courier_key.clone());
                                                            }
                                                            config_ctx.dispatch(ServerConfigsAction::UpdateCouriersWhitelist(new_whitelist.clone()));

                                                            let admin_request = AdminServerRequest::new(
                                                                AdminConfigurationType::CourierWhitelist,
                                                                serde_json::to_string(&new_whitelist).unwrap(),
                                                            );

                                                            let sender = sender.clone();
                                                            let keys = keys.clone();
                                                            yew::platform::spawn_local(async move {
                                                                match admin_request.sign_data(&keys).await {
                                                                    Ok(signed_request) => {
                                                                        sender.emit(signed_request);
                                                                    },
                                                                    Err(e) => {
                                                                        gloo::console::error!("Failed to sign request:", e.to_string());
                                                                    }
                                                                }
                                                            });
                                                        })
                                                    }
                                                    class="bg-green-500 text-white font-bold text-sm py-2 px-4 rounded-full"
                                                >
                                                    {"Restore"}
                                                </button>
                                            </div>
                                          </div>
                                        }
                                    }).collect::<Html>()
                                }
                            }
                            </div>
                        </div>
                    </div>
                </div>
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
    let keys = user_ctx.get_identity().cloned();
    let commerce_whitelist = config_ctx.get_couriers_whitelist();
    let config_ctx_clone = config_ctx.clone();

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
        let sender = sender.clone();
        let keys = keys.clone();
        yew::platform::spawn_local(async move {
            let signed_request = admin_request
                .sign_data(&keys)
                .await
                .expect("Failed to sign request");
            sender.emit(signed_request);
        });
    });
    html! {
        <form {onsubmit}
            class="rounded-2xl bg-white p-5 w-full md:max-w-lg mx-auto shadow-xl">
            <div class="space-y-5">
                <div class="space-y-2">
                    <label for="exchange_rate" class="block text-gray-500 font-bold text-center">{"New Courier Pubkey"}</label>
                    <input
                        type="text"
                        id="courier_id"
                        name="courier_id"
                        class="w-full rounded-lg border-2 border-fuente p-2"
                        value="" required={true} />
                </div>
                <div class="flex justify-center">
                    <input
                        type="submit"
                        value={translations["admin_settings_submit"].clone()}
                        class="bg-fuente-orange text-center text-white font-bold py-3 rounded-full w-full md:w-1/2 lg:mx-auto cursor-pointer"
                    />
                </div>
            </div>
        </form>
    }
}
