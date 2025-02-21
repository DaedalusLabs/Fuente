use driver::{
    contexts::{
        CommerceDataProvider, DriverDataAction, DriverDataProvider, DriverDataStore,
        OrderHubProvider, OrderHubStore,
    },
    router::DriverPages,
};
use fuente::{
    contexts::{AdminConfigsProvider, AdminConfigsStore, LanguageConfigsProvider},
    mass::{templates::LoginPageTemplate, LoadingScreen, LoginPage, SimpleInput, ToastProvider},
    models::{
        init_commerce_db, init_consumer_db, DriverProfile, DriverProfileIdb, DRIVER_HUB_PUB_KEY,
    },
};
use html::ChildrenProps;
use nostr_minions::{
    browser_api::HtmlForm,
    init_nostr_db,
    key_manager::{NostrIdAction, NostrIdProvider, NostrIdStore, UserIdentity},
    relay_pool::{NostrProps, RelayProvider, UserRelay},
};
use nostro2::keypair::NostrKeypair;
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
        // spawn_local(async move {
        //     let sw = nostr_minions::browser_api::AppServiceWorker::new()
        //         .expect("Error initializing service worker");
        //     sw.install("serviceWorker.js")
        //         .await
        //         .expect("Error installing service worker");
        // });
        || {}
    });
    html! {
        <LanguageConfigsProvider>
        <BrowserRouter>
            <RelayPoolComponent>
                <LoginContext>
                    <ToastProvider>
                        <LoginCheck>
                            <AppContext>
                                <ProfileCheck>
                                    <DriverPages />
                                </ProfileCheck>
                            </AppContext>
                        </LoginCheck>
                    </ToastProvider>
                </LoginContext>
            </RelayPoolComponent>
        </BrowserRouter>
        </LanguageConfigsProvider>
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

#[function_component(WhitelistWaitScreen)]
fn whitelist_wait_screen() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("NostrIdStore not found");
    let pubkey = key_ctx.get_nostr_key().unwrap().public_key();

    html! {
        <div class="flex flex-col gap-8 justify-center items-center flex-1 inset-0 py-8 px-16 fixed">
            <img src="/public/assets/img/logo.png" class="max-w-64 max-h-64"/>
            <div class="flex flex-col items-center gap-4 text-center">
                <h2 class="text-2xl font-bold text-fuente">{"Waiting for Admin Approval"}</h2>
                <p class="text-gray-600">{"Your account is pending admin approval. Please provide this public key to the administrator:"}</p>
                <div class="bg-gray-100 p-4 rounded-lg">
                    <p class="font-mono text-sm break-all">{pubkey}</p>
                </div>
            </div>
        </div>
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
            <LoginPage />
        };
    }
    let wl = admin_ctx.get_courier_whitelist();
    let pubkey = key_ctx.get_nostr_key().unwrap().public_key();
    if !wl.contains(&pubkey) {
        gloo::console::error!("User not in whitelist", &pubkey);
        return html! {<WhitelistWaitScreen />};
    }
    html! {
        {props.children.clone()}
    }
}

#[function_component(ProfileCheck)]
fn login_check(props: &ChildrenProps) -> Html {
    let user_ctx = use_context::<DriverDataStore>().expect("DriverDataStore not found");
    let order_ctx = use_context::<OrderHubStore>().expect("OrderHubProvider not found");
    if !user_ctx.finished_loading() || !order_ctx.finished_loading() {
        return html! {<LoadingScreen />};
    }
    if user_ctx.get_profile().is_none() {
        return html! {
            <LoginPageTemplate
                heading=""
                sub_heading=""
                title="Create Profile"
                >
                <NewProfile />
            </LoginPageTemplate>
        };
    }
    html! {
        {props.children.clone()}
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
            NostrKeypair::new_extractable(&user_keys_str).expect("Failed to create user keys");
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
        <form {onsubmit} class="flex flex-col gap-8 p-8 items-center bg-fuente-forms">
            <SimpleInput
                id="password"
                name="password"
                label="Private Key"
                value=""
                input_type="password"
                required={true}
                />
            <button type="submit"
                class="bg-fuente text-white text-center text-lg font-bold rounded-full w-full py-3 mt-5">
                {"Log In"}
            </button>
        </form>
    }
}
#[function_component(NewProfile)]
pub fn new_profile_form() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let user_ctx = use_context::<DriverDataStore>().expect("No CryptoId Context found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let sender = relay_pool.send_note.clone();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let user_ctx = user_ctx.clone();
        let keys = key_ctx.get_nostr_key().expect("No user keys found");
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let nickname = form_element
            .input_value("name")
            .expect("Failed to get name");
        let telephone = form_element
            .input_value("telephone")
            .expect("Failed to get telephone");
        let sender = sender.clone();
        let user_profile = DriverProfile::new(nickname, telephone);
        let db = DriverProfileIdb::new(user_profile.clone(), &keys);

        // Fix the giftwrapped_data calls by providing the proper parameters
        let giftwrap = user_profile
            .giftwrapped_data(&keys, keys.public_key(), keys.public_key())
            .expect("Failed to giftwrap data");
        let pool_copy = user_profile
            .giftwrapped_data(
                &keys,
                DRIVER_HUB_PUB_KEY.to_string(),
                DRIVER_HUB_PUB_KEY.to_string(),
            )
            .expect("Failed to giftwrap data");

        sender.emit(giftwrap);
        sender.emit(pool_copy);
        user_ctx.dispatch(DriverDataAction::NewProfile(db));
    });

    html! {
        <form {onsubmit}
            class="w-fit ml-auto flex flex-col gap-2 bg-fuente-forms rounded-3xl items-center p-5 space-y-5">
                <SimpleInput
                    id="name"
                    name="name"
                    label="Name"
                    value=""
                    input_type="text"
                    required={true}
                    />
                <SimpleInput
                    id="telephone"
                    name="telephone"
                    label="Telephone"
                    value=""
                    input_type="tel"
                    required={true}
                    />
                <button
                    type="submit"
                    class="bg-fuente-light p-3 rounded-3xl font-bold text-white hover:cursor-pointer w-2/4 mx-auto whitespace-normal text-nowrap">
                    {"Save"}
                </button>
        </form>
    }
}
