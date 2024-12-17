use fuente::{
    mass::{SimpleFormButton, SimpleInput},
    models::{
        ConsumerProfile, ConsumerProfileIdb, NOSTR_KIND_PRESIGNED_URL_REQ,
        NOSTR_KIND_PRESIGNED_URL_RESP, NOSTR_KIND_SERVER_REQUEST, TEST_PUB_KEY,
    },
};
use nostr_minions::{
    browser_api::{HtmlDocument, HtmlForm},
    key_manager::NostrIdStore,
    relay_pool::NostrProps,
};
use nostro2::notes::NostrNote;
use upload_things::{UtPreSignedUrl, UtUpload};
use wasm_bindgen::JsCast;
use web_sys::{FileReader, FormData, HtmlInputElement};
use yew::{platform::spawn_local, prelude::*};

use crate::contexts::{ConsumerDataAction, ConsumerDataStore};

#[function_component(NewProfilePage)]
pub fn new_profile() -> Html {
    let key_ctx = use_context::<NostrIdStore>().expect("No CryptoId Context found");
    let user_ctx = use_context::<ConsumerDataStore>().expect("No CryptoId Context found");
    let relay_pool = use_context::<NostrProps>().expect("No RelayPool Context found");

    let profile_pic_url = use_state(|| None);
    let url_handle = profile_pic_url.clone();
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    use_effect_with(relay_pool.unique_notes.clone(), move |notes| {
        if let Some(last_note) = notes.last() {
            if last_note.kind == NOSTR_KIND_PRESIGNED_URL_RESP {
                let decrypted_note = keys
                    .decrypt_nip_04_content(&last_note)
                    .expect("Failed to decrypt note");
                let presigned_url: UtPreSignedUrl = (&decrypted_note)
                    .try_into()
                    .expect("Failed to parse presigned url");
                gloo::console::log!("Presigned URL: {:?}", &presigned_url.url);
                let document = HtmlDocument::new().expect("Failed to get document");
                let input: HtmlInputElement = document
                    .find_element_by_id("file-input")
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
                            });
                        }
                    },
                )
                    as Box<dyn FnMut(web_sys::ProgressEvent)>);

                reader.set_onloadend(Some(closure.as_ref().unchecked_ref()));
                reader.read_as_array_buffer(&file).unwrap();
                closure.forget(); // Forget the closure to keep it alive
            }
        }
        || {}
    });

    let sender = relay_pool.send_note.clone();
    let keys = key_ctx.get_nostr_key().expect("No user keys found");
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let user_ctx = user_ctx.clone();
        let form_element = HtmlForm::new(e).expect("Failed to get form element");
        let nickname = form_element
            .input_value("name")
            .expect("Failed to get name");
        let email = form_element
            .input_value("email")
            .expect("Failed to get email");
        let telephone = form_element
            .input_value("telephone")
            .expect("Failed to get telephone");

        let sender = sender.clone();
        let user_profile = ConsumerProfile::new(
            nickname,
            email,
            telephone,
            profile_pic_url.as_ref().cloned(),
        );
        let db = ConsumerProfileIdb::new(user_profile.clone(), &keys);
        let giftwrap = user_profile
            .giftwrapped_data(&keys, keys.public_key())
            .expect("Failed to giftwrap data");
        let server_registry = user_profile
            .registry_data(&keys, TEST_PUB_KEY.to_string())
            .expect("Failed to giftwrap data");
        sender.emit(giftwrap);
        sender.emit(server_registry);
        user_ctx.dispatch(ConsumerDataAction::NewProfile(db));
    });
    let user_keys = key_ctx.get_nostr_key().expect("No user keys found");
    let sender = relay_pool.send_note.clone();
    let onchange = Callback::from(move |e: yew::Event| {
        let input = e
            .target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .expect("Failed to get input element");
        let file = input.files().unwrap().get(0).unwrap();
        let file_req = upload_things::UtRequest::from(&file);
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
    });

    html! {
        <form {onsubmit} class="flex flex-col gap-8 p-8 flex-1 items-center">
                <SimpleInput
                    id="name"
                    name="name"
                    label="Name"
                    value=""
                    input_type="text"
                    required={true}
                    />
                <SimpleInput
                    id="email"
                    name="email"
                    label="Email"
                    value=""
                    input_type="email"
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
                <input {onchange} type="file" id="file-input" />
                <SimpleFormButton>
                    {"Save"}
                </SimpleFormButton>
        </form>
    }
}
