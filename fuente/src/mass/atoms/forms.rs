use html::ChildrenProps;
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
        <div class="w-full">
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <input
                {id}
                {name}
                type={input_type}
                {value}
                {required}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
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
            <label class="text-xs font-bold text-neutral-400"
                for={id.clone()}>{label}</label>
            <textarea
                {id}
                {name}
                {value}
                {required}
                class="w-full border-b-2 border-neutral-400 p-0 py-2 pr-2 text-sm
                truncate bg-transparent border-0 focus:outline-none focus:border-b-2 focus:bg-transparent
                focus:ring-0 focus:ring-transparent tracking-widest focus:border-fuente-dark"
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
}

use nostr_minions::{browser_api::HtmlDocument, relay_pool::NostrProps};

use crate::models::{
    NOSTR_KIND_PRESIGNED_URL_REQ, NOSTR_KIND_PRESIGNED_URL_RESP, NOSTR_KIND_SERVER_REQUEST,
    TEST_PUB_KEY,
};
#[function_component(ImageUploadInput)]
pub fn image_upload_input(props: &ImageUploadInputProps) -> Html {
    let ImageUploadInputProps {
        url_handle,
        nostr_keys,
        classes,
    } = props.clone();
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");
    let user_keys = nostr_keys.clone();
    let url_clone = url_handle.clone();
    let is_loading_new = use_state(|| false);
    let loading_handle = is_loading_new.clone();
    use_effect_with(relay_pool.unique_notes.clone(), move |notes| {
        if let Some(last_note) = notes.last() {
            gloo::console::log!("Received note kind:", last_note.kind);
            if last_note.kind == NOSTR_KIND_PRESIGNED_URL_RESP {
                gloo::console::log!("Processing presigned URL response");
    
                let decrypted_note = match user_keys.decrypt_nip_04_content(&last_note) {
                    Ok(note) => note,
                    Err(e) => {
                        gloo::console::error!("Failed to decrypt note:", e.to_string());
                        return ();  // Changed from return;
                    }
                };
    
                let presigned_url: UtPreSignedUrl = match (&decrypted_note).try_into() {
                    Ok(url) => url,
                    Err(e) => {
                        gloo::console::error!("Failed to parse presigned url:", e.to_string());
                        return ();  // Changed from return;
                    }
                };
    
                let document = HtmlDocument::new().expect("Failed to get document");
                let input: HtmlInputElement = document
                    .find_element_by_id("imageUpload")
                    .expect("Failed to get file input");
                let files = input.files().expect("Failed to get files");
                let file = files.get(0).expect("Failed to get file");
                let form_data = FormData::new().expect("Failed to create form data");
                form_data.append_with_blob("file", &file).unwrap();

                let reader = FileReader::new().expect("Failed to create reader");
                let reader_handle = reader.clone();
                let closure = web_sys::wasm_bindgen::closure::Closure::wrap(Box::new(
                    move |_: web_sys::ProgressEvent| {
                        if let Ok(_) = reader_handle.result() {
                            let url = presigned_url.clone();
                            let form_data = form_data.clone();
                            let url_handle = url_handle.clone();
                            let loading_handle = loading_handle.clone();
                            spawn_local(async move {
                                let url_req =
                                    url.try_into_request(form_data).expect("Failed to convert");
                                let upload_url =
                                    nostr_minions::browser_api::BrowserFetch::request::<UtUpload>(
                                        &url_req,
                                    )
                                    .await
                                    .expect("Failed to fetch");
                                url_handle.set(Some(upload_url.url.clone()));
                                loading_handle.set(false);
                            });
                        }
                    },
                ) as Box<dyn FnMut(web_sys::ProgressEvent)>);
    
                reader.set_onloadend(Some(closure.as_ref().unchecked_ref()));
                reader.read_as_array_buffer(&file).unwrap();
                closure.forget(); // Forget the closure to keep it alive
            }
        }
        // Changed this line to return unit type
        ()
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
                     <div class="relative">
                    <img src={url.clone()} class={image_classes} />
                    <label for="imageUpload" class={with_url}>
                        <input {onchange} id="imageUpload" type="file" accept="image/*" class="hidden" />
                        {match *is_loading_new {
                            true => html! {
                                <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
                            },
                            false => html! {
                                <span class="text-gray-500">{"CHANGE"}</span>
                            }
                        }}
                    </label>
                    </div>
                }
            }
            None => html! {
                <label for="imageUpload" class={default_classes}>
                    <input {onchange} id="imageUpload" type="file" accept="image/*" class="hidden" />
                    {match *is_loading_new {
                        true => html! {
                            <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
                        },
                        false => html! {
                            <span class="text-gray-500">{"IMAGE UPLOAD"}</span>
                        }
                    }}
                </label>
            }
        }}
        </div>
    }
}
