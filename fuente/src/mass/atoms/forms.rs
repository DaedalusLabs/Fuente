use html::ChildrenProps;
use lucide_yew::{Globe, Plus};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use upload_things::{UtPreSignedUrl, UtUpload};
use wasm_bindgen::JsCast;
use web_sys::{FileReader, FormData, HtmlInputElement};
use yew::{platform::spawn_local, prelude::*};
use yew_router::{
    hooks::{use_navigator, use_route},
    Routable,
};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SimpleInputProps {
    pub id: String,
    pub name: String,
    pub input_type: String,
    pub value: String,
    pub required: bool,
    pub label: String,
}

#[function_component(SimpleInput)]
pub fn simple_input(props: &SimpleInputProps) -> Html {
    let value = props.value.clone();
    let id = props.id.clone();
    let name = props.name.clone();
    let input_type = props.input_type.clone();
    let required = props.required;
    let label = props.label.clone();
    html! {
        <div class="space-y-1">
            <label
                class="text-white text-lg block text-left"
                for={id.clone()}>{label}</label>
            <input
                {id}
                {name}
                type={input_type}
                {value}
                {required}
                class="p-3 w-full rounded-xl"
                />
        </div>
    }
}

#[function_component(SimpleTextArea)]
pub fn simple_textarea(props: &SimpleInputProps) -> Html {
    let value = props.value.clone();
    let id = props.id.clone();
    let name = props.name.clone();
    let required = props.required;
    let label = props.label.clone();
    html! {
        <div class="w-full">
            <label 
                class="text-white text-lg block text-left"
                for={id.clone()}>{label}</label>
            <textarea
                {id}
                {name}
                {value}
                {required}
                class="p-3 w-full rounded-xl"
                />
        </div>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct SimpleSelectProps {
    pub id: String,
    pub name: String,
    pub label: String,
    pub children: Children,
}

#[function_component(SimpleSelect)]
pub fn simple_select(props: &SimpleSelectProps) -> Html {
    let id = props.id.clone();
    let name = props.name.clone();
    let required = true;
    let label = props.label.clone();
    html! {
        <div class="w-full">
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <select
                {id}
                {name}
                {required}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
                >
                {props.children.clone()}
            </select>
        </div>
    }
}

#[function_component(SimpleFormButton)]
pub fn simple_button(props: &ChildrenProps) -> Html {
    html! {
        <button type={"submit"}
            class="bg-fuente-light text-white font-mplus p-4 mx-16 rounded-3xl
            focus:outline-none focus:shadow-outline hover:bg-fuente-dark m-8"
            >
            {props.children.clone()}
        </button>
    }
}

#[function_component(MoneyInput)]
pub fn money_input(props: &SimpleInputProps) -> Html {
    let value = props.value.clone();
    let id = props.id.clone();
    let name = props.name.clone();
    let required = props.required;
    let label = props.label.clone();
    html! {
        <div class="w-full">
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <input
                {id}
                {name}
                type={"number"}
                {value}
                {required}
                step={"0.01"}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
                />
        </div>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct AppLinkProps<T>
where
    T: Routable,
{
    pub children: Children,
    pub class: String,
    pub selected_class: String,
    pub route: T,
}

#[function_component(AppLink)]
pub fn sidebar_link<T>(props: &AppLinkProps<T>) -> Html
where
    T: Routable + 'static,
{
    let navigator = use_navigator();
    if navigator.is_none() {
        return html! {};
    }
    let navigator = navigator.unwrap();
    let current_route = use_route::<T>().unwrap();

    let onclick = {
        let route = props.route.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| navigator.push(&route))
    };
    let class = if current_route == props.route {
        props.selected_class.clone()
    } else {
        props.class.clone()
    };
    html! {
        <button {onclick} {class}>
            {props.children.clone()}
        </button>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct ImageUploadInputProps {
    pub url_handle: UseStateHandle<Option<String>>,
    pub nostr_keys: NostrKeypair,
    pub classes: Classes,
    pub input_id: String,
}

use nostr_minions::{browser_api::HtmlDocument, relay_pool::NostrProps};

use crate::{
    contexts::{AppLocale, LanguageConfigsAction, LanguageConfigsStore},
    models::{
        NOSTR_KIND_PRESIGNED_URL_REQ, NOSTR_KIND_PRESIGNED_URL_RESP, NOSTR_KIND_SERVER_REQUEST,
        TEST_PUB_KEY,
    },
};
#[function_component(ImageUploadInput)]
pub fn image_upload_input(props: &ImageUploadInputProps) -> Html {
    let ImageUploadInputProps {
        url_handle,
        nostr_keys,
        classes,
        input_id,
    } = props.clone();
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let user_keys = nostr_keys.clone();
    let url_clone = url_handle.clone();
    let is_loading_new = use_state(|| false);
    let loading_handle = is_loading_new.clone();

    // Clone input_id for use in the effect
    let input_id_for_effect = input_id.clone();

    use_effect_with(relay_pool.unique_notes.clone(), move |notes| {
        if let Some(last_note) = notes.last() {
            if last_note.kind == NOSTR_KIND_PRESIGNED_URL_RESP {
                gloo::console::log!("Processing presigned URL response");

                let decrypted_note = match user_keys.decrypt_nip_04_content(&last_note) {
                    Ok(note) => note,
                    Err(e) => {
                        gloo::console::error!("Failed to decrypt note:", e.to_string());
                        return;
                    }
                };

                let presigned_url: UtPreSignedUrl = match (&decrypted_note).try_into() {
                    Ok(url) => url,
                    Err(e) => {
                        gloo::console::error!("Failed to parse presigned url:", e.to_string());
                        return;
                    }
                };

                // Get document and input safely
                let document = match HtmlDocument::new() {
                    Ok(doc) => doc,
                    Err(e) => {
                        gloo::console::error!("Failed to get document:", e);
                        return;
                    }
                };

                let input: HtmlInputElement =
                    match document.find_element_by_id(&input_id_for_effect) {
                        Ok(input) => input,
                        Err(e) => {
                            gloo::console::error!("Failed to find input element:", e);
                            return;
                        }
                    };

                let files = match input.files() {
                    Some(files) => files,
                    None => {
                        gloo::console::error!("No files found");
                        return;
                    }
                };

                let file = match files.get(0) {
                    Some(file) => file,
                    None => {
                        gloo::console::error!("No file selected");
                        return;
                    }
                };

                // Create form data
                let form_data = match FormData::new() {
                    Ok(form) => form,
                    Err(e) => {
                        gloo::console::error!("Failed to create form data:", e);
                        return;
                    }
                };

                if let Err(e) = form_data.append_with_blob("file", &file) {
                    gloo::console::error!("Failed to append file to form data:", e);
                    return;
                }

                // Create reader and set up upload
                let reader = match FileReader::new() {
                    Ok(reader) => reader,
                    Err(e) => {
                        gloo::console::error!("Failed to create file reader:", e);
                        return;
                    }
                };

                let reader_handle = reader.clone();
                let loading_handle = loading_handle.clone();
                let url_handle_clone = url_handle.clone();
                let loading_handle_clone = loading_handle.clone();
                let presigned_url_clone = presigned_url.clone();
                let form_data_clone = form_data.clone();

                let closure = web_sys::wasm_bindgen::closure::Closure::wrap(Box::new(
                    move |_: web_sys::ProgressEvent| {
                        if let Ok(_) = reader_handle.result() {
                            let url_setter = url_handle_clone.clone();
                            let loading_setter = loading_handle_clone.clone();
                            let url = presigned_url_clone.clone();
                            let form_data = form_data_clone.clone();

                            spawn_local(async move {
                                let url_req = match url.try_into_request(form_data) {
                                    Ok(req) => req,
                                    Err(e) => {
                                        gloo::console::error!("Failed to create request:", e);
                                        loading_setter.set(false);
                                        return;
                                    }
                                };

                                match nostr_minions::browser_api::BrowserFetch::request::<UtUpload>(
                                    &url_req,
                                )
                                .await
                                {
                                    Ok(upload_url) => {
                                        gloo::console::log!(
                                            "Upload successful, setting URL:",
                                            &upload_url.url
                                        );
                                        url_setter.set(Some(upload_url.url));
                                    }
                                    Err(e) => {
                                        gloo::console::error!("Upload failed:", e);
                                    }
                                }
                                loading_setter.set(false);
                            });
                        }
                    },
                )
                    as Box<dyn FnMut(web_sys::ProgressEvent)>);

                reader.set_onloadend(Some(closure.as_ref().unchecked_ref()));
                if let Err(e) = reader.read_as_array_buffer(&file) {
                    gloo::console::error!("Failed to read file:", e);
                    return;
                }
                closure.forget();
            }
        }
    });
    let user_keys = nostr_keys.clone();
    let sender = relay_pool.send_note.clone();
    let loading_handle = is_loading_new.clone();
    let onchange = Callback::from(move |e: yew::Event| {
        gloo::console::log!("Starting file upload process");
        loading_handle.set(true);
        let input = e
            .target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .expect("Failed to get input element");
        let file = input.files().unwrap().get(0).unwrap();
        let file_req = upload_things::UtRequest::from(&file);

        gloo::console::log!("Created file request:", &file_req.to_string());
        let mut req_note = NostrNote {
            content: file_req.to_string(),
            kind: NOSTR_KIND_PRESIGNED_URL_REQ,
            pubkey: user_keys.public_key(),
            ..Default::default()
        };
        user_keys.sign_nostr_event(&mut req_note);
        let mut giftwrap = NostrNote {
            content: req_note.to_string(),
            kind: NOSTR_KIND_SERVER_REQUEST,
            pubkey: user_keys.public_key(),
            ..Default::default()
        };
        user_keys
            .sign_nip_04_encrypted(&mut giftwrap, TEST_PUB_KEY.to_string())
            .unwrap();
        sender.emit(giftwrap);
        gloo::console::log!("Sent request for presigned URL");
    });
    let mut default_classes = classes!(
        "flex",
        "items-center",
        "justify-center",
        "cursor-pointer",
        "border-4",
        "border-dashed",
        "border-blue-500",
        "rounded-xl"
    );
    default_classes.extend(classes.clone());
    let mut with_url = default_classes.clone();
    with_url.extend(classes!("bg-transparent", "absolute"));
    let mut image_classes = classes!("rounded-xl", "absolute");
    image_classes.extend(classes);
    html! {
        <div class="flex justify-center items-center">
        {match url_clone.as_ref() {
            Some(url) => {
                html! {
                     <div class="relative w-full h-full">
                        <img src={url.clone()} class="w-full h-full object-cover" />
                        <label for={input_id.clone()} class="absolute inset-0 flex items-center justify-center">
                            <input {onchange} id={input_id.clone()} type="file" accept="image/*" class="hidden" />
                            {match *is_loading_new {
                                true => html! {
                                    <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
                                },
                                false => html! {
                                }
                            }}
                        </label>
                    </div>
                }
            }
            None => html! {
                <label for={input_id.clone()} class={default_classes}>
                    <input {onchange} id={input_id.clone()} type="file" accept="image/*" class="hidden" />
                    {match *is_loading_new {
                        true => html! {
                            <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
                        },
                        false => html! {
                            <Plus class="w-8 h-8 text-blue-500" />
                        }
                    }}
                </label>
            }
        }}
        </div>
    }
}
#[function_component(LanguageToggle)]
pub fn language_toggle() -> Html {
    let language_ctx =
        use_context::<LanguageConfigsStore>().expect("LanguageConfigsStore not found");
    let current_locale = language_ctx.current_locale();
    let translations = language_ctx.translations();

    html! {
        <div class="p-6 space-y-6">
            <div class="flex items-center space-x-3 border-b pb-2">
                <Globe class="text-fuente w-6 h-6" />
                <h2 class="text-2xl font-bold text-fuente">
                  {&translations["profile_settings_language"]}
                </h2>
            </div>
            <div class="flex flex-wrap gap-2">
                <button
                    onclick={
                        let language_ctx = language_ctx.clone();
                        Callback::from(move |_| {
                            language_ctx.dispatch(LanguageConfigsAction::ChangeLocale(AppLocale::English));
                        })
                    }
                    class={classes!(
                        "px-4",
                        "py-2",
                        "rounded-lg",
                        if matches!(current_locale, AppLocale::English) {
                            "bg-fuente text-white"
                        } else {
                            "bg-gray-100 hover:bg-gray-200"
                        }
                    )}
                >
                    {"EN"}
                </button>
                <button
                    onclick={
                        let language_ctx = language_ctx.clone();
                        Callback::from(move |_| {
                            language_ctx.dispatch(LanguageConfigsAction::ChangeLocale(AppLocale::Dutch));
                        })
                    }
                    class={classes!(
                        "px-4",
                        "py-2",
                        "rounded-lg",
                        if matches!(current_locale, AppLocale::Dutch) {
                            "bg-fuente text-white"
                        } else {
                            "bg-gray-100 hover:bg-gray-200"
                        }
                    )}
                >
                    {"NL"}
                </button>
            </div>
        </div>
    }
}
