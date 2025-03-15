use crate::{
    contexts::LanguageConfigsStore,
    mass::{
        templates::LoginPageTemplate, Toast, ToastAction, ToastContext, ToastProvider, ToastType,
    },
};
use nostr_minions::{
    browser_api::{HtmlForm, IdbStoreManager},
    key_manager::{NostrIdAction, NostrIdStore, UserIdentity},
};
use nostro2::keypair::NostrKeypair;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, PartialEq)]
enum AuthPage {
    Login,
    Register,
    Landing,
}

#[derive(Properties, Clone, PartialEq)]
pub struct AuthPageProps {
    pub login_handle: Callback<MouseEvent>,
}

#[function_component(LoginPage)]
pub fn login_template() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");

    let translations = language_ctx.translations();
    let login_type = use_state(|| AuthPage::Landing);
    let title = match *login_type {
        AuthPage::Login => &translations["auth_login_title"],
        AuthPage::Register => &translations["auth_register_title"],
        AuthPage::Landing => &translations["auth_login_title"],
    };
    let heading = match *login_type {
        AuthPage::Login => &translations["auth_login_heading_shop"],
        AuthPage::Register => &translations["auth_register_heading"],
        AuthPage::Landing => &translations["auth_login_heading_shop"],
    };
    html! {
        <ToastProvider>
            <LoginPageTemplate
                heading={heading.to_string()}
                sub_heading={translations["auth_register_heading_now"].clone()}
                title={title.to_string()}>
                    {match *login_type {
                        AuthPage::Login => html! {<LoginForm  />},
                        AuthPage::Register => html! {<RegisterUserForm  />},
                        AuthPage::Landing => html! {<LoginLanding login_handle={login_type.setter()} />},
                    }}
            </LoginPageTemplate>
        </ToastProvider>
    }
}

#[derive(Properties, Clone, PartialEq)]
struct AuthPageSetProps {
    login_handle: UseStateSetter<AuthPage>,
}

#[function_component(LoginLanding)]
fn landing_login(props: &AuthPageSetProps) -> Html {
    html! {
        <div class="bg-fuente-forms py-[65px] px-5 rounded-3xl relative z-0 flex flex-col gap-16">
            <div class="space-y-2 flex flex-col items-center">
            <h3 class="text-white text-2xl font-bold text-center">{"Start shopping now"}</h3>
            <button
                onclick={
                    let setter = props.login_handle.clone();
                    Callback::from(move |_| setter.set(AuthPage::Register))}
                class="bg-fuente-light p-3 rounded-3xl font-bold text-white hover:cursor-pointer w-fit mx-auto"
            >
                {"Create an account"}
            </button>
            </div>
            <div class="space-y-2 flex flex-col justify-center w-full">
            <h3 class="text-white text-lg font-bold text-center">{"Already have an account?"}</h3>
            <button
                onclick={
                let setter = props.login_handle.clone();
                Callback::from(move |_| setter.set(AuthPage::Login))}
                class="bg-fuente-light p-3 rounded-3xl font-bold text-white hover:cursor-pointer w-fit mx-auto"
            >
                {"Login with Nostr"}
            </button>
            </div>
        </div>
    }
}

#[function_component(LoginForm)]
pub fn import_user_form() -> Html {
    let user_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let toast_ctx = use_context::<ToastContext>().expect("No Toast Context found");
    let translations = language_ctx.translations();
    let onclick = {
        let user_ctx = user_ctx.clone();
        let toast_ctx = toast_ctx.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let document =
                nostr_minions::browser_api::HtmlDocument::new().expect("Failed to get document");
            let user_keys_str = document
                .find_element_by_id::<HtmlInputElement>("private_key")
                .expect("Failed to get password")
                .value();
            let Ok(mut user_keys) = NostrKeypair::try_from(&user_keys_str) else {
                let toast = Toast {
                    message: "Invalid private key".to_string(),
                    toast_type: ToastType::Error,
                };
                toast_ctx.dispatch(ToastAction::Show(toast));
                return;
            };
            user_keys.make_extractable();
            let user_ctx = user_ctx.clone();
            let toast_ctx = toast_ctx.clone();
            spawn_local(async move {
                let Ok(user_identity) = UserIdentity::from_new_keys(user_keys).await else {
                    let toast = Toast {
                        message: "Failed to create user identity".to_string(),
                        toast_type: ToastType::Error,
                    };
                    toast_ctx.dispatch(ToastAction::Show(toast));
                    return;
                };
                let Ok(keys) = user_identity.get_user_keys().await else {
                    let toast = Toast {
                        message: "Failed to get user keys".to_string(),
                        toast_type: ToastType::Error,
                    };
                    toast_ctx.dispatch(ToastAction::Show(toast));
                    return;
                };
                user_ctx.dispatch(NostrIdAction::LoadIdentity(
                    keys.public_key(),
                    user_identity,
                ));
            });
        })
    };

    let user_ctx = user_ctx.clone();
    let extension_login = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        let user_ctx = user_ctx.clone();
        let toast_ctx = toast_ctx.clone();
        spawn_local(async move {
            let Ok(user_identity) = UserIdentity::new_extension_identity().await else {
                let toast = Toast {
                    message: "Failed to create user identity".to_string(),
                    toast_type: ToastType::Error,
                };
                toast_ctx.dispatch(ToastAction::Show(toast));
                return;
            };
            let Some(keys) = user_identity.get_pubkey().await else {
                let toast = Toast {
                    message: "Failed to get user keys".to_string(),
                    toast_type: ToastType::Error,
                };
                toast_ctx.dispatch(ToastAction::Show(toast));
                return;
            };

            if user_identity.clone().save_to_store().await.is_err() {
                let toast = Toast {
                    message: "Failed to save user identity".to_string(),
                    toast_type: ToastType::Error,
                };
                toast_ctx.dispatch(ToastAction::Show(toast));
                return;
            };
            user_ctx.dispatch(NostrIdAction::LoadIdentity(keys, user_identity));
        });
    });

    html! {
      <form class="bg-fuente-forms py-[65px] px-5 rounded-3xl relative z-0">
          <div class="space-y-1">
              <label
                  for="private_key"
                  class="text-white text-lg block text-left"
              >
                  {&translations["auth_login_form_label"]}
              </label>
              <input
                  id="private_key"
                  name="private_key"
                  label="Private Key"
                  input_type="text"
                  class="p-2 w-full rounded-xl"
                  required={true}
                  />
          </div>


          <div class="space-y-7 flex flex-col mt-5">
            <input
                {onclick}
                type="submit"
                class="bg-fuente-light p-3 rounded-3xl font-bold text-white hover:cursor-pointer w-fit mx-auto"
                value={translations["auth_login_link_button"].clone()}
            />
            <p class="text-white text-center font-thin">{"or"}</p>
            <button
                type="button"
                onclick={extension_login}
              class="bg-fuente-light p-3 rounded-3xl font-bold text-white hover:cursor-pointer w-fit mx-auto align-center">
              {&translations["auth_login_extension"]}
              </button>
          </div>
      </form>
    }
}

#[function_component(RegisterUserForm)]
pub fn new_user_form() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    let user_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let new_keys = use_state(|| NostrKeypair::generate(true));
    let private_key = new_keys
        .get_secret_key()
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();
    let onsubmit = {
        let user_ctx = user_ctx.clone();
        let new_keys = (*new_keys).clone();
        Callback::from(move |e: SubmitEvent| {
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
                    .expect("Failed to get user keys")
                    .public_key();
                user_ctx.dispatch(NostrIdAction::LoadIdentity(keys, user_identity));
            });
        })
    };

    html! {
        <form   {onsubmit}
            class="bg-fuente-forms py-[65px] px-5 rounded-3xl relative z-0">
            <div class="space-y-5">
                <span class="w-full font-bold text-white flex flex-col gap-4">
                    <p>{"A new Nostr private key will be created for you, and saved to your device."}</p>
                    <p>{"This key encrypts all your data while using our platform."}</p>
                    <p>{"You can find your key in your profile settings to backup manually."}</p>
                    <p class="font-extrabold" >{"We do not have access to your key!"}</p>
                </span>
                <input type="hidden" class="hidden" name="username" type="text" value="Fuente Private Key" />
                <input type="hidden" class="hidden" name="password" id="password" type="password" value={private_key} />
            </div>
            <div class="space-y-5 flex flex-col mt-5">
                <input
                    type="submit"
                    class="bg-fuente-light p-3 rounded-3xl font-bold text-white hover:cursor-pointer w-fit mx-auto whitespace-normal text-nowrap"
                    value={translations["auth_login_accept"].clone()}
                />
            </div>
        </form>
    }
}

#[function_component(AdminLoginPage)]
pub fn admin_login() -> Html {
    html! {
        <div class="h-dvh w-dvw">
        <AdminLoginForm />
        </div>
    }
}
#[function_component(AdminLoginForm)]
pub fn import_user_form() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    let user_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let user_keys_str = form_element
            .input_value("password")
            .expect("Failed to get password");
        let user_keys = NostrKeypair::try_from(&user_keys_str).expect("Failed to create user keys");
        let user_ctx = user_ctx.clone();
        spawn_local(async move {
            let user_identity = UserIdentity::from_new_keys(user_keys)
                .await
                .expect("Failed to create user identity");
            let keys = user_identity.get_user_keys().await.unwrap().public_key();
            user_ctx.dispatch(NostrIdAction::LoadIdentity(keys, user_identity));
        });
    });

    html! {
        <LoginPageTemplate
            heading={"".to_string()}
            sub_heading={"".to_string()}
            title={translations["auth_login_title"].clone()}>
            <form {onsubmit} class="bg-fuente-forms py-[65px] px-5 rounded-3xl relative">
                <div class="space-y-1">
                    <label
                        for="password"
                        class="text-white text-lg block text-left"
                    >
                        {"Password"}
                    </label>
                    <input
                        id="password"
                        name="password"
                        type="password"
                        class="p-2 w-full rounded-xl"
                        required={true}
                    />
                </div>
                <div class="space-y-5 flex flex-col mt-5">
                    <input
                        type="submit"
                        class="bg-fuente-buttons p-3 rounded-3xl font-bold text-fuente hover:cursor-pointer w-2/4 mx-auto"
                        value={"Login".to_string()}
                    />
                </div>
            </form>
        </LoginPageTemplate>
    }
}
