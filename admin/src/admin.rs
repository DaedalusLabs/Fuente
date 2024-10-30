use admin::config_context::{ServerConfigsProvider, ServerConfigsStore};
use fuente::{
    browser::html::HtmlForm,
    contexts::{
        key_manager::{NostrIdProvider, NostrIdStore},
        relay_pool::{NostrProps, RelayProvider},
    },
    mass::{
        atoms::{
            forms::{SimpleFormButton, SimpleInput},
            layouts::{LoadingScreen, MainLayout},
        },
        molecules::login::AdminLoginPage,
    },
    models::{
        admin_configs::{AdminConfigurationType, AdminServerRequest},
        init_shared_db,
        nostr_kinds::{NOSTR_KIND_COMMERCE_PROFILE, NOSTR_KIND_COURIER_PROFILE, NOSTR_KIND_SERVER_CONFIG},
        relays::UserRelay,
        ADMIN_WHITELIST,
    },
};
use html::ChildrenProps;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew_router::BrowserRouter;

fn main() {
    yew::Renderer::<App>::new().render();
}
#[function_component(App)]
fn app() -> Html {
    use_effect_with((), move |_| {
        init_shared_db().unwrap();
        || {}
    });
    html! {
        <BrowserRouter>
           <AppContext>
               <MainLayout>
                   <LoginCheck>
                      <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Welcome to the Admin Panel"}</h2>
                      <ExchangeRateForm />
                      <ExchangeRateDisplay />
                      <CommerceWhitelistForm />
                      <CourierWhitelistForm />
                   </LoginCheck>
               </MainLayout>
           </AppContext>
        </BrowserRouter>
    }
}

#[function_component(RelayProviderComponent)]
fn relay_pool_component(props: &ChildrenProps) -> Html {
    html! {
            {props.children.clone()}
    }
}

#[function_component(AppContext)]
fn app_context(props: &ChildrenProps) -> Html {
    let relays = vec![
        UserRelay {
            url: "wss://relay.arrakis.lat".to_string(),
            read: true,
            write: true,
        },
        UserRelay {
            url: "wss://relay.illuminodes.com".to_string(),
            read: true,
            write: true,
        },
    ];
    html! {
        <RelayProvider {relays}>
            <NostrIdProvider>
                <ServerConfigsProvider>
                    {props.children.clone()}
                </ServerConfigsProvider>
            </NostrIdProvider>
        </RelayProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let server_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    if !key_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    let keys = key_ctx.get_key();
    if keys.is_none() {
        return html! {
            <div class="flex justify-center items-center flex-1">
                <AdminLoginPage />
            </div>
        };
    }
    if !server_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    if !ADMIN_WHITELIST.contains(&keys.unwrap().get_public_key().as_str()) {
        return html! {
            <div class="flex justify-center items-center flex-1">
                <h2 class="text-2xl px-8 py-4 font-bold text-center">{"You are not authorized to access this page"}</h2>
            </div>
        };
    }
    html! {
        {props.children.clone()}
    }
}

#[function_component(ExchangeRateDisplay)]
fn exchange_rate_display() -> Html {
    let server_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let exchange_rate = server_ctx.get_exchange_rate();
    gloo::console::info!(format!("Exchange rate is {}", exchange_rate));

    html! {
        <></>
    }
}

#[function_component(ExchangeRateForm)]
fn exchange_rate_form() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");

    let sender = relay_ctx.send_note.clone();
    let keys = user_ctx.get_key();
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
            <SimpleFormButton>
                {"Set Exchange Rate"}
            </SimpleFormButton>
        </form>
    }
}
#[function_component(CommerceWhitelistForm)]
fn exchange_rate_form() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let subscriber = relay_ctx.subscribe.clone();
    use_effect_with((), move |_| {
        let filter = nostro2::relays::NostrFilter::default().new_kind(NOSTR_KIND_COMMERCE_PROFILE);
        subscriber.emit(filter.subscribe());
        || {}
    });
    let unique_notes = relay_ctx.unique_notes.clone();
    let config_handler = config_ctx.clone();
    let unregistered_keys = use_state(|| vec![]);
    let unregister = unregistered_keys.clone();
    use_effect_with(unique_notes, move |notes| {
        if let Some(note) = notes.last() {
            if note.get_kind() == NOSTR_KIND_COMMERCE_PROFILE {
                if !config_handler.check_commerce_whitelist(&note.get_pubkey()) {
                    let mut keys = (*unregister).clone();
                    keys.push(note.get_pubkey());
                    unregister.set(keys);
                }
            }
        }
        || {}
    });
    let sender = relay_ctx.send_note.clone();
    let keys = user_ctx.get_key();
    let commerce_whitelist = config_ctx.get_commerce_whitelist();
    let onclick = Callback::from(move |e: MouseEvent| {
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
    html! {
        <div class="flex flex-col gap-8 p-8 items-center">
            <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Commerce Whitelist"}</h2>
            {{
                unregistered_keys.iter().map(|key| {
                    html! {
                        <div class="flex flex-row gap-8">
                            <p>{key.clone()}</p>
                            <button onclick={onclick.clone()}
                                    id={key.clone()}
                                    class="bg-green-500 text-white p-2 rounded">{"Register"}</button>
                        </div>
                    }
                }).collect::<Html>()
            }}
        </div>
    }
}

#[function_component(CourierWhitelistForm)]
pub fn courier_whitelist_form() -> Html {
    let relay_ctx = use_context::<NostrProps>().expect("NostrProps not found");
    let user_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let config_ctx = use_context::<ServerConfigsStore>().expect("ServerConfigsStore not found");
    let sender = relay_ctx.send_note.clone();
    let keys = user_ctx.get_key();
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
