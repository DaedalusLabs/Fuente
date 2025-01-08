use crate::{
    contexts::LanguageConfigsStore,
    mass::templates::LoginPageTemplate,
};
use lucide_yew::Copy;
use nostr_minions::{
    browser_api::{clipboard_copy, HtmlForm},
    key_manager::{NostrIdAction, NostrIdStore, UserIdentity},
};
use nostro2::keypair::NostrKeypair;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, PartialEq)]
enum AuthPage {
    Login,
    Register,
}

#[derive(Properties, Clone, PartialEq)]
pub struct AuthPageProps {
    pub login_handle: Callback<MouseEvent>,
}

#[function_component(LoginPage)]
pub fn login_template() -> Html {
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    let login_type = use_state(|| AuthPage::Login);
    let register: Callback<MouseEvent> = {
        let login_type = login_type.clone();
        Callback::from(move |_| login_type.set(AuthPage::Register))
    };
    let login: Callback<MouseEvent> = {
        let login_type = login_type.clone();
        Callback::from(move |_| login_type.set(AuthPage::Login))
    };
    let title = match *login_type {
        AuthPage::Login => &translations["auth_login_title"],
        AuthPage::Register => &translations["auth_register_title"],
    };
    let heading = match *login_type {
        AuthPage::Login => &translations["auth_login_heading_shop"],
        AuthPage::Register => &translations["auth_register_heading"],
    };
    html! {
        <LoginPageTemplate 
            heading={heading.to_string()} 
            sub_heading={translations["auth_register_heading_now"].clone()} 
            title={title.to_string()}>
                {match *login_type {
                    AuthPage::Login => html! {<LoginForm login_handle={register} />},
                    AuthPage::Register => html! {<RegisterUserForm login_handle={login} />},
                }}
        </LoginPageTemplate>
    }
}
#[function_component(LoginForm)]
pub fn import_user_form(props: &AuthPageProps) -> Html {
    let AuthPageProps { login_handle } = props;
    let user_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    let onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        let document =
            nostr_minions::browser_api::HtmlDocument::new().expect("Failed to get document");
        let user_keys_str = document
            .find_element_by_id::<HtmlInputElement>("private_key")
            .expect("Failed to get password")
            .value();
        let user_keys =
            NostrKeypair::new_extractable(&user_keys_str).expect("Failed to create user keys");
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

          <div class="space-y-5 flex flex-col mt-5">
              <a
                  class="text-center text-white font-thin underline cursor-pointer hover:text-cyan-400">
                  {&translations["auth_login_link_key"]}
              </a>
              <a  onclick={login_handle}
                  class="text-center text-white font-thin underline cursor-pointer hover:text-fuente-buttons">
                  {&translations["auth_login_link_register"]}
              </a>
              <input
                  {onclick}
                  type="submit"
                  class="bg-fuente-buttons p-3 rounded-3xl font-bold text-fuente hover:cursor-pointer w-2/4 mx-auto"
                  value={translations["auth_login_link_button"].clone()}
              />
          </div>
      </form>
    }
}

#[function_component(RegisterUserForm)]
pub fn new_user_form(props: &AuthPageProps) -> Html {
    let AuthPageProps { login_handle } = props;
    let language_ctx = use_context::<LanguageConfigsStore>().expect("No Language Context found");
    let translations = language_ctx.translations();
    let user_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let new_keys = NostrKeypair::generate(true);
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
        <form   {onsubmit}
            class="bg-fuente-forms py-[65px] px-5 rounded-3xl relative z-0">
            <div class="space-y-5">
                <div class="space-y-1">
                    <label
                        for="private_key"
                        class="text-white text-lg block text-left"
                    >
                        {&translations["auth_login_form_label"]}
                    </label>
                    <span class="w-full font-bold flex gap-2">
                        <input
                            id="private_key"
                            name="private_key"
                            label="Private Key"
                            value={private_key.clone()}
                            class="p-2 w-full rounded-xl"
                            type="text"
                            required={true}
                            disabled={true}
                            />
                        <button
                            type="button"
                            onclick={Callback::from(move |_: MouseEvent| {
                                 clipboard_copy(&key_clone);
                            })}>
                            <Copy class="w-8 h-8 text-white" />
                        </button>
                    </span>
                </div>
                <span class="w-full font-bold text-white">
                    <p>{"This key encrypts all your data."}</p>
                    <p>{"You must keep this safe."}</p>
                    <p>{"We do not have access to your keys!"}</p>
                </span>
                <input type="hidden" class="hidden" name="password" id="password" type={"password"} value={private_key} />
            </div>
            <div class="space-y-5 flex flex-col mt-5">
                <a  onclick={login_handle}
                    class="text-center text-white font-thin underline cursor-pointer hover:text-fuente-buttons">{"I have an account - Login"}</a>
                <input
                    type="submit"
                    class="bg-fuente-buttons p-3 rounded-3xl font-bold text-fuente hover:cursor-pointer w-2/4 mx-auto whitespace-normal"
                    value={translations["auth_register_link_button"].clone()}
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
