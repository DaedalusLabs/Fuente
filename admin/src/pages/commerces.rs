use fuente::{
    contexts::{NostrIdStore, NostrProps},
    models::{
        AdminConfigurationType, AdminServerRequest, CommerceProfile, NOSTR_KIND_COMMERCE_PROFILE,
    },
};
use nostro2::notes::SignedNote;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::ServerConfigsStore;

#[derive(Clone, PartialEq, Properties)]
pub struct CommerceDetailsProps {
    pub commerce_id: SignedNote,
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
        let display_name = profile.name();
        let description = profile.description();
        let telephone = profile.telephone();
        let email = profile.web();
        html! {
        <div class="w-full max-w-3xl mx-auto h-fit shadow-xl">
          <div class="flex flex-col md:flex-row">
            <div class="flex-grow p-6 md:p-8">
              <div class="space-y-2">
                <p class="text-lg font-semibold">{display_name}</p>
                <p class="text-sm text-muted-foreground">{description}</p>
                <p class="text-sm">
                  <span class="font-medium">{"Tel:"}</span> {telephone}
                </p>
                <p class="text-sm">
                  <span class="font-medium">{"Email:"}</span> {email}
                </p>
              </div>
            </div>
            <div class="flex items-center justify-center p-6 md:p-8 bg-muted">
              <button {onclick} id={commerce_id.get_pubkey()}
                class="w-full md:w-auto text-lg">
                    {action}
              </button>
            </div>
          </div>
        </div>

            }
    } else {
        html! {}
    }
}

#[function_component(CommerceDisplay)]
pub fn unregistered_commerces() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let subscriber = relay_ctx.subscribe.clone();
    let unregistered_commerces: UseStateHandle<Vec<SignedNote>> = use_state(|| vec![]);
    let unregistered = unregistered_commerces.clone();
    let registered_commerces: UseStateHandle<Vec<SignedNote>> = use_state(|| vec![]);
    let registered = registered_commerces.clone();
    
    use_effect_with(config_ctx, move |configs| {
        gloo::console::log!("CommerceDisplay effect");
        let not_in_wl = configs.get_unregistered_commerces();
        let in_wl = configs.get_whitelisted_commerces();
        unregistered.set(not_in_wl);
        registered.set(in_wl);
        || {}
    });

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
        <div class="flex flex-row gap-8 p-8">
            <div class="flex flex-col gap-8">
                <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Unregistered Commerces"}</h2>
                {{
                    unregistered_commerces.iter().map(|key| {
                        html! {
                            <CommerceDetails commerce_id={(*key).clone()} onclick={register_onclick.clone()} action={"Register"} />
                        }
                    }).collect::<Html>()
                }}
            </div>
            <div class="flex flex-col gap-8">
                <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Registered Commerces"}</h2>
                {{
                    registered_commerces.iter().map(|key| {
                        html! {
                            <CommerceDetails commerce_id={(*key).clone()} onclick={unregister_onclick.clone()} action={"Unregister"} />
                        }
                    }).collect::<Html>()
                }}
            </div>
        </div>
    }
}
