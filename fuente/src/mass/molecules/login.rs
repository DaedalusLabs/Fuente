use crate::{
    browser_api::{clipboard_copy, HtmlForm},
    contexts::{NostrIdAction, NostrIdStore, UserIdentity},
    mass::atoms::{
        CopyIcon, {SimpleFormButton, SimpleInput},
    },
};
use nostro2::userkeys::UserKeys;
use yew::{platform::spawn_local, prelude::*};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct LoginProps {
    login_handle: UseStateHandle<LoginType>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoginType {
    NewUser,
    ImportUser,
}

#[function_component(NewUserPage)]
pub fn new_user() -> Html {
    let new_user_state = use_state(|| LoginType::NewUser);
    html! {
        <div class="flex flex-col w-full h-full">
            <LoginTypeSelector login_handle={new_user_state.clone()} />
            {match &*new_user_state {
                LoginType::NewUser => html! {
                    <>
                        <NewUserForm />
                    </>
                },
                LoginType::ImportUser => html! {
                    <>
                        <ImportUserForm />
                    </>
                },
            }}
        </div>
    }
}

#[function_component(LoginTypeSelector)]
pub fn popup_selector(props: &LoginProps) -> Html {
    let new_user_state = props.login_handle.clone();
    let state_clone = new_user_state.clone();
    let selected_class = "border-b-4 border-purple-900 text-sm font-bold py-4 px-8 mx-8";
    let unselected_class = "text-sm font-bold py-4 px-8 mx-8";
    html! {
        <div class="w-full h-1/3 shadow-xl rounded-b-3xl bg-neutral-50 flex flex-col justify-end relative">
            <div class="absolute inset-0 flex items-center justify-center select-none z-0">
                <img src="/public/assets/img/logo.png" alt="Suriname Logo" class="w-32 h-32" />
            </div>
            <div class="flex flex-row gap-2 z-10 justify-between">
                <button
                    class={if *new_user_state == LoginType::NewUser { selected_class } else { unselected_class }}
                    onclick={Callback::from(move |_| {
                        gloo::console::log!("New User");
                        state_clone.set(LoginType::NewUser);
                    })} >
                    {"Sign-up"}
                </button>
                <button
                    class={if *new_user_state == LoginType::ImportUser { selected_class } else { unselected_class }}
                    onclick={Callback::from(move |_| {
                        gloo::console::log!("Import User");
                        new_user_state.set(LoginType::ImportUser);
                    })}
                    >
                    {"Recover"}
                </button>
            </div>
        </div>
    }
}

#[function_component(NewUserForm)]
pub fn new_user_form() -> Html {
    let user_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let new_keys = UserKeys::generate_extractable();
    let private_key = new_keys
        .get_secret_key()
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let user_ctx = user_ctx.clone();
        let new_keys = new_keys.clone();
        spawn_local(async move {
            let user_identity = UserIdentity::from_new_keys(new_keys)
                .await
                .expect("Failed to create user identity");
            let keys = user_identity
                .get_user_keys()
                .await
                .expect("Failed to get user keys");
            user_ctx.dispatch(NostrIdAction::LoadIdentity(user_identity, keys));
        });
    });
    let key_clone = private_key.clone();
    html! {
        <form {onsubmit} class="flex flex-col gap-8 flex-1 p-8 items-center">
                <SimpleInput
                    id="private_key"
                    name="private_key"
                    label="Private Key"
                    value={private_key.clone()}
                    input_type="text"
                    required={true}
                    />
                <button
                    type="button"
                    onclick={Callback::from(move |_: MouseEvent| {
                         clipboard_copy(&key_clone);
                    })}
                    class="flex flex-row-reverse gap-0.5 text-blue-400 items-center text-sm w-full">
                        {"Copy Key"}
                        <CopyIcon class="w-6 h-6" />
                </button>
                <span class="w-full font-bold text-neutral-500">
                    <p>{"This key encrypts all your data."}</p>
                    <p>{"You must keep this safe."}</p>
                    <p>{"We do not have access to your keys!"}</p>
                </span>
                <input type="hidden" class="hidden" name="password" id="password" type={"password"} value={private_key} />
                <SimpleFormButton>
                    {"Understood!"}
                </SimpleFormButton>
        </form>
    }
}

#[function_component(ImportUserForm)]
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
            let keys = user_identity
                .get_user_keys()
                .await
                .expect("Failed to get user keys");
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

#[function_component(AdminLoginPage)]
pub fn admin_login() -> Html {
    html! {
        <div class="flex flex-col h-full sm:max-w-sm md:max-w-md lg:max-w-lg items-center justify-center gap-4">
            <div class="flex items-center justify-center select-none">
                <img src="/public/assets/img/logo.png" alt="Fuente Logo" class="w-24 h-24" />
            </div>
            <AdminLoginForm />
        </div>
    }
}
#[function_component(AdminLoginForm)]
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
