use admin::config_context::{ServerConfigsProvider, ServerConfigsStore};
use fuente::{
    browser::html::HtmlForm,
    contexts::{
        key_manager::{NostrIdProvider, NostrIdStore},
        relay_pool::{NostrProps, RelayProvider},
    },
    mass::{
        atoms::{
            forms::SimpleFormButton,
            layouts::{LoadingScreen, MainLayout},
        },
        molecules::login::AdminLoginPage,
    },
    models::{
        admin_configs::{AdminConfigurationType, AdminServerRequest},
        init_shared_db,
        nostr_kinds::NOSTR_KIND_SERVER_CONFIG,
        relays::UserRelay,
        ADMIN_WHITELIST,
    },
};
use html::ChildrenProps;
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
            <RelayProviderComponent>
                <NostrIdProvider>
                    <MainLayout>
                        <LoginCheck>
                            <AppContext>
                                <h2 class="text-2xl px-8 py-4 font-bold text-center">{"Welcome to the Admin Panel"}</h2>
                                <ExchangeRateForm />
                                <ExchangeRateDisplay />
                            </AppContext>
                        </LoginCheck>
                    </MainLayout>
                </NostrIdProvider>
            </RelayProviderComponent>
        </BrowserRouter>
    }
}

#[function_component(RelayProviderComponent)]
fn relay_pool_component(props: &ChildrenProps) -> Html {
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
            {props.children.clone()}
        </RelayProvider>
    }
}

#[function_component(AppContext)]
fn app_context(props: &ChildrenProps) -> Html {
    html! {
            <ServerConfigsProvider>
                {props.children.clone()}
            </ServerConfigsProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
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
    // use_effect_with(server_ctx.get_exchange_rate(), move |rate| {
    //     gloo::console::info!(format!("Exchange rate is {}", rate));
    //     || {}
    // });

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
