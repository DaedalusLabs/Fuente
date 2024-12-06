use driver::{
    contexts::{CommerceDataProvider, DriverDataProvider, DriverDataStore, OrderHubProvider},
    pages::NewProfileForm,
    router::DriverPages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore},
    mass::{LoadingScreen, MainLayout, SimpleFormButton, SimpleInput},
    models::{init_commerce_db, init_consumer_db},
};
use html::ChildrenProps;
use nostr_minions::{
    browser_api::{clipboard_copy, HtmlDocument, HtmlForm},
    init_nostr_db,
    key_manager::{NostrIdAction, NostrIdProvider, NostrIdStore, UserIdentity},
    relay_pool::{RelayProvider, UserRelay},
};
use nostro2::userkeys::UserKeys;
use yew::{platform::spawn_local, prelude::*};
use yew_router::BrowserRouter;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    use_effect_with((), move |_| {
        init_nostr_db().expect("Error initializing Nostr database");
        init_consumer_db().expect("Error initializing Fuente database");
        init_commerce_db().expect("Error initializing Commerce database");
        || {}
    });
    html! {
        <BrowserRouter>
            <RelayPoolComponent>
                <LoginContext>
                    <MainLayout>
                    <LoginCheck>
                        <AppContext>
                            <ProfileCheck>
                                <DriverPages />
                            </ProfileCheck>
                        </AppContext>
                    </LoginCheck>
                    </MainLayout>
                </LoginContext>
            </RelayPoolComponent>
        </BrowserRouter>
    }
}

#[function_component(RelayPoolComponent)]
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

#[function_component(LoginContext)]
fn app_context(props: &ChildrenProps) -> Html {
    html! {
        <NostrIdProvider>
            <AdminConfigsProvider >
                        {props.children.clone()}
            </AdminConfigsProvider>
        </NostrIdProvider>
    }
}

#[function_component(AppContext)]
fn app_context(props: &ChildrenProps) -> Html {
    html! {
            <DriverDataProvider>
                <CommerceDataProvider>
                    <OrderHubProvider>
                        {props.children.clone()}
                    </OrderHubProvider>
                </CommerceDataProvider>
            </ DriverDataProvider>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let admin_ctx = use_context::<AdminConfigsStore>().expect("AdminConfigsProvider not found");
    if !key_ctx.finished_loading() || !admin_ctx.is_loaded() {
        return html! {<LoadingScreen />};
    }
    if key_ctx.get_nostr_key().is_none() {
        return html! {
            <DriverLoginPage />
        };
    }
    let wl = admin_ctx.get_courier_whitelist();
    let pubkey = key_ctx.get_nostr_key().unwrap().get_public_key();
    if !wl.contains(&pubkey) {
        return html! {
            <div class="flex flex-col gap-2 justify-center items-center flex-1">
                <h1>{"You are not authorized to use this service"}</h1>
                <p>{"Your public key: "}</p>
                <p>{pubkey}</p>
            </div>
        };
    }
    html! {
        {props.children.clone()}
    }
}

#[function_component(ProfileCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let user_ctx = use_context::<DriverDataStore>().expect("DriverDataStore not found");
    if !user_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    if user_ctx.get_profile().is_none() {
        return html! {
            <div class="flex justify-center items-center flex-1">
                <NewProfileForm />
            </div>
        };
    }
    html! {
        {props.children.clone()}
    }
}

#[function_component(DriverLoginPage)]
pub fn admin_login() -> Html {
    let onclick = Callback::from(move |_| {
        let new_keys = UserKeys::generate_extractable();
        let new_keys_str = new_keys.get_secret_key();
        let mut hex_str = String::new();
        for byte in new_keys_str.iter() {
            hex_str.push_str(&format!("{:02x}", byte));
        }
        clipboard_copy(&hex_str);
        let input_element = HtmlDocument::new()
            .expect("Failed to get document")
            .find_element_by_id::<web_sys::HtmlInputElement>("password")
            .expect("Failed to get element");
        input_element.set_value(&hex_str);
    });
    html! {
        <div class="flex flex-col h-full w-full items-center justify-center gap-4">
            <div class="flex items-center justify-center select-none">
                <img src="/public/assets/img/logo.png" alt="Fuente Logo" class="w-24 h-24" />
            </div>
            <DriverLoginForm />
            <button class="text-lg bg-fuente-light rounded-lg w-fit h-fit py-2 px-4 text-white font-bold " {onclick}>
                {"NewKey"}
            </button>
            <p class="text-sm text-gray-500 text-wrap max-w-sm">{"Click the NewKey button to generate a new key and copy it to your clipboard"}</p>
        </div>
    }
}
#[function_component(DriverLoginForm)]
pub fn import_user_form() -> Html {
    let user_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let user_keys_str = form_element
            .input_value("password")
            .expect("Failed to get password");
        let user_keys =
            UserKeys::new_extractable(&user_keys_str).expect("Failed to create user keys");
        let user_ctx = user_ctx.clone();
        spawn_local(async move {
            let user_identity = UserIdentity::from_new_keys(user_keys)
                .await
                .expect("Failed to create user identity");
            let keys = user_identity.get_user_keys().await.unwrap();
            user_ctx.dispatch(NostrIdAction::LoadIdentity(user_identity, keys));
        });
    });

    html! {
        <form {onsubmit} class="flex flex-col gap-8 p-8 items-center">
            <SimpleInput
                id="password"
                name="password"
                label="Private Key"
                value=""
                input_type="password"
                required={true}
                />
            <SimpleFormButton>
                {"Log In"}
            </SimpleFormButton>
        </form>
    }
}
