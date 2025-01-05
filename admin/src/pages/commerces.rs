use fuente::{
    contexts::LanguageConfigsStore,
    models::{AdminConfigurationType, AdminServerRequest, CommerceProfile},
};
use nostr_minions::{key_manager::NostrIdStore, relay_pool::NostrProps};
use nostro2::notes::NostrNote;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::ServerConfigsStore;

#[function_component(CommercesPage)]
pub fn exchange_rate_page() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    html! {
        <>
        <div class="container mx-auto lg:py-10 flex flex-col lg:flex-row items-center lg:justify-between">
            <h3 class="text-fuente text-4xl pb-10 lg:pb-0 text-center lg:text-left lg:text-6xl font-bold tracking-tighter">
                {&translations["admin_settings_title_commerces"]}
            </h3>
        </div>
        <CommerceDisplay />
        </>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct CommerceDetailsProps {
    pub commerce_id: NostrNote,
    pub onclick: Callback<MouseEvent>,
    pub action: String,
}

#[function_component(CommerceDetails)]
fn commerce_details(props: &CommerceDetailsProps) -> Html {
    let CommerceDetailsProps {
        commerce_id,
        onclick,
        action,
    } = props;
    if let Ok(profile) = CommerceProfile::try_from(commerce_id.clone()) {
        html! {
        <div
            // onclick={props.on_click.clone()} id={order_id}
            class="bg-white shadow py-2 px-5 rounded-2xl space-y-1 w-full">
            <p class="text-fuente font-bold text-md">{profile.name}</p>
            <p class="text-sm text-muted-foreground">{&profile.description}</p>
            <p class="text-gray-500 text-xs">{&profile.telephone}</p>
            <p class="text-gray-500 text-xs">{&profile.web}</p>
            <p class="text-gray-500 text-xs">{format!("{}", commerce_id.pubkey[..16].to_string())}</p>
            <div class="flex items-center justify-center p-6 md:p-8 bg-muted">
              <button {onclick} id={commerce_id.pubkey.clone()}
                    class="bg-fuente-orange text-white text-center text-lg font-bold rounded-full w-full px-8 py-3 mt-5" >
                    {action}
              </button>
            </div>
        </div>
            }
    } else {
        html! {}
    }
}

#[function_component(CommerceDisplay)]
pub fn unregistered_commerces() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("ServerConfigsStore not found");
    let translations = language_ctx.translations();
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let unregistered_commerces: UseStateHandle<Vec<NostrNote>> = use_state(|| vec![]);
    let registered_commerces: UseStateHandle<Vec<NostrNote>> = use_state(|| vec![]);

    let unregistered = unregistered_commerces.clone();
    let registered = registered_commerces.clone();
    use_effect_with(config_ctx, move |configs| {
        gloo::console::log!("CommerceDisplay effect");
        let not_in_wl = configs.get_unregistered_commerces();
        let in_wl = configs.get_whitelisted_commerces();
        unregistered.set(not_in_wl);
        registered.set(in_wl);
        || {}
    });

    let unregistered = unregistered_commerces.clone();
    let registered = registered_commerces.clone();
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let sender = relay_ctx.send_note.clone();
    let keys = user_ctx.get_nostr_key();
    let commerce_whitelist = config_ctx.get_commerce_whitelist();
    let register_onclick = Callback::from(move |e: MouseEvent| {
        let keys = keys.clone().expect("No keys found");
        let mut new_whitelist = commerce_whitelist.clone();
        let commerce_id = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().id();
        new_whitelist.push(commerce_id);
        let admin_request = AdminServerRequest::new(
            AdminConfigurationType::CommerceWhitelist,
            serde_json::to_string(&new_whitelist).unwrap(),
        );
        let signed_request = admin_request
            .sign_data(&keys)
            .expect("Failed to sign request");
        sender.emit(signed_request);
    });
    let sender = relay_ctx.send_note.clone();
    let keys = user_ctx.get_nostr_key();
    let commerce_whitelist = config_ctx.get_commerce_whitelist();
    let unregister_onclick = Callback::from(move |e: MouseEvent| {
        let keys = keys.clone().expect("No keys found");
        let mut new_whitelist = commerce_whitelist.clone();
        let commerce_id = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().id();
        new_whitelist.retain(|id| id != &commerce_id);
        let admin_request = AdminServerRequest::new(
            AdminConfigurationType::CommerceWhitelist,
            serde_json::to_string(&new_whitelist).unwrap(),
        );
        let signed_request = admin_request
            .sign_data(&keys)
            .expect("Failed to sign request");
        sender.emit(signed_request);
    });
    html! {
        <main class="container mx-auto mt-10 max-h-full pb-4 overflow-y-clip no-scrollbar">
            <div class="flex flex-col md:flex-row gap-10 mt-10 min-h-96">
                <div class="flex-flex-col gap-4">
                    <div class="border-2 border-red-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-red-500 text-lg font-semibold text-center">{&translations["admin_settings_commerces_unregistered"]}</p>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 auto-cols-fr gap-8 bg-red-100 rounded-2xl mt-2 px-2 py-2 w-full max-h-[24rem] sm:max-h-[36rem] overflow-y-auto no-scrollbar" >
                        {for unregistered.iter().map(|id| {
                            html! {
                                <CommerceDetails commerce_id={id.clone()} onclick={register_onclick.clone()} action={translations["admin_settings_commerces_register"].clone()} />
                            }
                        })}
                    </div>
                </div>

                <div class="flex-flex-col gap-4">
                    <div class="border-2 border-green-500 rounded-2xl py-3 px-2 h-fit w-fit">
                        <p class="text-green-500 text-lg font-semibold text-center">{&translations["admin_settings_commerces_registered"]}</p>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 auto-cols-fr gap-8 bg-green-100 rounded-2xl mt-2 max-h-[24rem] max-h-[36rem] px-2 py-2 w-full overflow-y-auto no-scrollbar" >
                        {for registered.iter().map(|id| {
                            html! {
                                <CommerceDetails commerce_id={id.clone()} onclick={unregister_onclick.clone()} action={translations["admin_settings_commerces_unregister"].clone()} />
                            }
                        })}
                    </div>

                </div>

            </div>
        </main>
    }
}
