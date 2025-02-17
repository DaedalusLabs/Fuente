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
        <main class="container mx-auto overflow-hidden">
            <div class="flex flex-col h-full">
                <div class="flex flex-row justify-between items-center p-4 lg:py-10">
                    <h1 class="text-fuente text-4xl text-center lg:text-left py-4 lg:py-0 lg:text-6xl tracking-tighter font-bold font-mplus">
                        {&translations["admin_settings_title_commerces"]}
                    </h1>
                </div>
                <CommerceDisplay />
            </div>
        </main>
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
            class="py-2 px-5 space-y-1 w-full border-b border-gray-400 last-of-type:border-b-0">
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
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let unregistered_commerces: UseStateHandle<Vec<NostrNote>> = use_state(|| vec![]);
    let registered_commerces: UseStateHandle<Vec<NostrNote>> = use_state(|| vec![]);

    let unregistered = unregistered_commerces.clone();
    let registered = registered_commerces.clone();
    use_effect_with(config_ctx, move |configs| {
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
        <>
            <CommerceRegistrationMobile
                whitelisted_commerces={(*registered).clone()}
                unregistered_commerces={(*unregistered).clone()}
                register_onclick={register_onclick.clone()}
                unregister_onclick={unregister_onclick.clone()}
            />
            <CommerceRegistrationDesktop
                whitelisted_commerces={(*registered).clone()}
                unregistered_commerces={(*unregistered).clone()}
                register_onclick={register_onclick}
                unregister_onclick={unregister_onclick}
            />
        </>
    }
}

#[derive(Clone, PartialEq)]
pub enum CommerceStatus {
    Registered,
    Unregistered,
}

#[derive(Properties, Clone, PartialEq)]
pub struct CommerceRegistrationProps {
    pub whitelisted_commerces: Vec<NostrNote>,
    pub unregistered_commerces: Vec<NostrNote>,
    pub register_onclick: Callback<MouseEvent>,
    pub unregister_onclick: Callback<MouseEvent>,
}

#[function_component(CommerceRegistrationMobile)]
pub fn order_history_desktop(props: &CommerceRegistrationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let filter = use_state(|| CommerceStatus::Registered);
    let CommerceRegistrationProps {
        whitelisted_commerces,
        unregistered_commerces,
        register_onclick,
        unregister_onclick,
    } = props;
    let bg_color = match *filter {
        CommerceStatus::Registered => "bg-green-100",
        CommerceStatus::Unregistered => "bg-red-100",
    };
    html! {
        <div class="flex lg:hidden flex-1 overflow-hidden">
            <div class="flex flex-1 justify-evenly gap-4 h-full p-4 overflow-hidden">
                <div class="flex flex-col gap-2 w-full h-full overflow-hidden">
                    <div class="grid grid-flow-col justify-stretch gap-2 w-full">
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(CommerceStatus::Registered)
                            })}
                            class={classes!("border-green-500", "border-2", "rounded-2xl", "py-3", "px-2", "w-full")}>
                            <p class={classes!("text-lg", "font-semibold", "text-center", "text-green-500", "cursor-pointer", "whitespace-nowrap")}>
                                {&translations["admin_settings_commerces_registered"]}
                            </p>
                        </div>
                        <div
                            onclick={Callback::from({
                                let filter = filter.clone();
                                move |_| filter.set(CommerceStatus::Unregistered)
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
                                CommerceStatus::Registered => {
                                    whitelisted_commerces.iter().map(|id| {
                                        html! { <CommerceDetails commerce_id={id.clone()} onclick={unregister_onclick.clone()} action={translations["admin_settings_commerces_unregister"].clone()} />}
                                    }).collect::<Html>()
                                },
                                CommerceStatus::Unregistered => {
                                    unregistered_commerces.iter().map(|id| {
                                        html! { <CommerceDetails commerce_id={id.clone()} onclick={register_onclick.clone()} action={translations["admin_settings_commerces_register"].clone()} />}
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

#[function_component(CommerceRegistrationDesktop)]
pub fn order_history_desktop(props: &CommerceRegistrationProps) -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No language context found");
    let translations = language_ctx.translations();
    let CommerceRegistrationProps {
        whitelisted_commerces,
        unregistered_commerces,
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
                        {whitelisted_commerces.iter().map(|id| {
                            html! { <CommerceDetails commerce_id={id.clone()} onclick={unregister_onclick.clone()} action={translations["admin_settings_commerces_unregister"].clone()} />}
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
                        {unregistered_commerces.iter().map(|id| {
                            html! { <CommerceDetails commerce_id={id.clone()} onclick={register_onclick.clone()} action={translations["admin_settings_commerces_register"].clone()} />}
                        }).collect::<Html>()}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

