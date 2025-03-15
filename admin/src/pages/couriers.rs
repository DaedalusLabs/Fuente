use fuente::{
    contexts::LanguageConfigsStore,
    models::{AdminConfigurationType, AdminServerRequest, DriverProfile},
};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::notes::NostrNote;
use web_sys::{wasm_bindgen::JsCast, HtmlElement};
use yew::prelude::*;

use crate::ServerConfigsStore;

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
                <CourierDisplay />
            </div>
        </main>
    }
}

#[function_component(CourierDisplay)]
pub fn unregistered_couriers() -> Html {
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let unregistered_couriers: UseStateHandle<Vec<(NostrNote, DriverProfile)>> =
        use_state(|| vec![]);
    let registered_couriers: UseStateHandle<Vec<(NostrNote, DriverProfile)>> = use_state(|| vec![]);

    let unregistered = unregistered_couriers.clone();
    let registered = registered_couriers.clone();
    use_effect_with(config_ctx, move |configs| {
        let mut not_in_wl = configs.get_all_couriers();
        not_in_wl.retain(|(note, _)| !configs.get_couriers_whitelist().contains(&note.pubkey));
        let in_wl = configs.get_whitelisted_couriers();
        unregistered.set(not_in_wl);
        registered.set(in_wl);
        || {}
    });

    let unregistered = unregistered_couriers.clone();
    let registered = registered_couriers.clone();
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let keys = user_ctx.clone();
    let courier_whitelist = config_ctx.get_couriers_whitelist();
    let sender = relay_ctx.send_note.clone();
    let register_onclick = Callback::from(move |e: MouseEvent| {
        let keys = keys.get_identity().cloned().expect("No keys found");
        let mut new_whitelist = courier_whitelist.clone();
        let courier_id = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().id();
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
    let keys = user_ctx.clone();
    let courier_whitelist = config_ctx.get_couriers_whitelist();
    let unregister_onclick = Callback::from(move |e: MouseEvent| {
        let keys = keys.get_identity().cloned().expect("No keys found");
        let mut new_whitelist = courier_whitelist.clone();
        let courier_id = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().id();
        new_whitelist.retain(|id| id != &courier_id);
        let admin_request = AdminServerRequest::new(
            AdminConfigurationType::CourierWhitelist,
            serde_json::to_string(&new_whitelist).unwrap(),
        );
        let sender = relay_ctx.send_note.clone();
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
        <>
            <CourierRegistrationMobile
                whitelisted_couriers={(*registered).clone()}
                unregistered_couriers={(*unregistered).clone()}
                register_onclick={register_onclick.clone()}
                unregister_onclick={unregister_onclick.clone()}
            />
            <CourierRegistrationDesktop
                whitelisted_couriers={(*registered).clone()}
                unregistered_couriers={(*unregistered).clone()}
                register_onclick={register_onclick}
                unregister_onclick={unregister_onclick}
            />
        </>
    }
}

#[derive(Clone, PartialEq)]
pub enum CourierStatus {
    Registered,
    Unregistered,
}

#[derive(Properties, Clone, PartialEq)]
pub struct CourierRegistrationProps {
    pub whitelisted_couriers: Vec<(NostrNote, DriverProfile)>,
    pub unregistered_couriers: Vec<(NostrNote, DriverProfile)>,
    pub register_onclick: Callback<MouseEvent>,
    pub unregister_onclick: Callback<MouseEvent>,
}

#[function_component(CourierRegistrationMobile)]
pub fn order_history_desktop(props: &CourierRegistrationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let filter = use_state(|| CourierStatus::Registered);
    let CourierRegistrationProps {
        whitelisted_couriers,
        unregistered_couriers,
        register_onclick,
        unregister_onclick,
    } = props;
    let bg_color = match *filter {
        CourierStatus::Registered => "bg-green-100",
        CourierStatus::Unregistered => "bg-red-100",
    };
    html! {
        <div class="flex lg:hidden flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                <div class="flex flex-col gap-2 w-full h-full overflow-hidden">
                    <div class="grid grid-flow-col justify-stretch gap-2 w-full">
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(CourierStatus::Registered)
                            })}
                            class={classes!("border-green-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                            <p class={classes!("text-lg", "font-semibold", "text-center", "text-green-500", "cursor-pointer", "whitespace-nowrap")}>
                                {&translations["admin_settings_commerces_registered"]}
                            </p>
                        </div>
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(CourierStatus::Unregistered)
                            })}
                            class={classes!("border-red-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                            <p class={classes!("text-lg", "font-semibold", "text-center", "text-red-500", "cursor-pointer", "whitespace-nowrap")}>
                                {&translations["admin_settings_commerces_unregistered"]}
                            </p>
                        </div>
                    </div>
                    <div
                        class={classes!("flex-1", "rounded-2xl", "mt-2", "px-2", "py-2", "overflow-y-auto", "no-scrollbar", bg_color)}>
                        <div class="grid grid-cols-1 gap-4">
                        {
                            match *filter {
                                CourierStatus::Registered => {
                                    whitelisted_couriers.iter().map(|(note, profile)| {
                                        html! {
                                        <div class="flex flex-row justify-between items-center p-2 bg-white rounded-2xl">
                                          <div class="flex flex-row gap-2">
                                            <div class="flex flex-col gap-2">
                                              <p class="text-gray-500 font-light text-lg">{profile.nickname()}</p>
                                              <p class="text-gray-500 font-mono text-sm break-all">{&note.id}</p>
                                            </div>
                                          </div>
                                          <div class="flex flex-row gap-2">
                                              <button
                                                  id={note.pubkey.clone()}
                                                  onclick={
                                                    unregister_onclick.clone()
                                                  }
                                                  class="bg-green-500 text-white font-bold text-sm py-2 px-4 rounded-full"
                                              >
                                                  {"Restore"}
                                              </button>
                                          </div>
                                        </div>
                                        }
                                    }).collect::<Html>()
                                },
                                CourierStatus::Unregistered => {
                                    unregistered_couriers.iter().map(|(note, profile)| {
                                        html! {
                                        <div class="flex flex-row justify-between items-center p-2 bg-white rounded-2xl">
                                          <div class="flex flex-row gap-2">
                                            <div class="flex flex-col gap-2">
                                              <p class="text-gray-500 font-light text-lg">{profile.nickname()}</p>
                                              <p class="text-gray-500 font-mono text-sm break-all">{&note.id}</p>
                                            </div>
                                          </div>
                                          <div class="flex flex-row gap-2">
                                              <button
                                                  id={note.pubkey.clone()}
                                                  onclick={
                                                  register_onclick.clone()
                                                  }
                                                  class="bg-green-500 text-white font-bold text-sm py-2 px-4 rounded-full"
                                              >
                                                  {"Restore"}
                                              </button>
                                          </div>
                                        </div>
                                        }
                                    }).collect::<Html>()
                                },
                            }
                        }
                        </div>
                    </div>

                </div>
            </div>
        </div>
    }
}

#[function_component(CourierRegistrationDesktop)]
pub fn order_history_desktop(props: &CourierRegistrationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let CourierRegistrationProps {
        whitelisted_couriers,
        unregistered_couriers,
        register_onclick,
        unregister_onclick,
    } = props;
    html! {
        <div class="hidden lg:flex flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                <div class="flex flex-col gap-2 w-1/2 h-full overflow-hidden">
                    <div class="border-2 border-green-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-green-500 text-lg font-semibold text-center">{&translations["admin_settings_commerces_registered"]}</p>
                    </div>

                     <div class={"flex-1 rounded-2xl mt-2 px-2 py-2 overflow-y-auto no-scrollbar bg-green-100 border-2 border-green-500"}>
                        <div class="grid grid-cols-1 gap-4">
                        {whitelisted_couriers.iter().map(|(note, profile)| {
                            html! {
                            <div class="flex flex-row justify-between items-center p-2 bg-white rounded-2xl">
                              <div class="flex flex-row gap-2">
                                <div class="flex flex-col gap-2">
                                  <p class="text-gray-500 font-light text-lg">{profile.nickname()}</p>
                                  <p class="text-gray-500 font-mono text-sm break-all">{&note.id}</p>
                                </div>
                              </div>
                              <div class="flex flex-row gap-2">
                                  <button
                                      id={note.pubkey.clone()}
                                      onclick={
                                        unregister_onclick.clone()
                                      }
                                      class="bg-green-500 text-white font-bold text-sm py-2 px-4 rounded-full"
                                  >
                                      {"Restore"}
                                  </button>
                              </div>
                            </div>
                            }
                        }).collect::<Html>()}
                        </div>
                    </div>

                </div>

                <div class="flex flex-col gap-2 w-1/2 h-full overflow-hidden">
                    <div class="border-2 border-red-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-red-500 text-lg font-semibold text-center">{&translations["admin_settings_commerces_unregistered"]}</p>
                    </div>

                     <div class={"flex-1 rounded-2xl mt-2 px-2 py-2 overflow-y-auto no-scrollbar bg-red-100 border-2 border-red-500"}>
                        <div class="grid grid-cols-1 gap-4">
                        {unregistered_couriers.iter().map(|(note, profile)| {
                            html! {
                            <div class="flex flex-row justify-between items-center p-2 bg-white rounded-2xl">
                              <div class="flex flex-row gap-2">
                                <div class="flex flex-col gap-2">
                                  <p class="text-gray-500 font-light text-lg">{profile.nickname()}</p>
                                  <p class="text-gray-500 font-mono text-sm break-all">{&note.id}</p>
                                </div>
                              </div>
                              <div class="flex flex-row gap-2">
                                  <button
                                      id={note.pubkey.clone()}
                                      onclick={
                                      register_onclick.clone()
                                      }
                                      class="bg-green-500 text-white font-bold text-sm py-2 px-4 rounded-full"
                                  >
                                      {"Restore"}
                                  </button>
                              </div>
                            </div>
                            }
                        }).collect::<Html>()}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
