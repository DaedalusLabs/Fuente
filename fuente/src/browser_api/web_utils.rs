use gloo::console::log;
use js_sys::Function;
use serde::de::DeserializeOwned;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::ServiceWorkerContainer;

pub fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}
pub fn document() -> web_sys::Document {
    window().document().unwrap()
}
pub fn navigator() -> web_sys::Navigator {
    window().navigator()
}
pub fn js_array_to_vec<T>(array_buffer: JsValue) -> Result<Vec<T>, String>
where
    T: DeserializeOwned + TryFrom<JsValue, Error = JsValue>,
{
    let cast_vec = array_buffer.dyn_into::<js_sys::Array>().unwrap();
    let structs: Vec<T> = cast_vec
        .iter()
        .map(|x| {
            let js_value = x.dyn_into::<JsValue>().unwrap();
            js_value.try_into().unwrap()
        })
        .collect();
    Ok(structs)
}
pub fn vec_to_js_array<T>(values: &[T]) -> JsValue
where
    T: AsRef<str>,
{
    return JsValue::from(
        values
            .iter()
            .map(|x| JsValue::from_str(x.as_ref()))
            .collect::<js_sys::Array>(),
    );
}
pub fn confirm_user_action(message: &str) -> Result<bool, JsValue> {
    window().confirm_with_message(message)
}
pub fn clipboard_copy(message: &str) {
    let promise = window().navigator().clipboard().write_text(message);
    spawn_local(async move {
        let _ = JsFuture::from(promise).await;
    });
}

fn sw() -> web_sys::ServiceWorkerContainer {
    web_sys::window().unwrap().navigator().service_worker()
}

async fn install_service_worker(sw: &ServiceWorkerContainer) {
    let register = sw.register("serviceWorker.js");
    let register = JsFuture::from(register).await;
    match register {
        Ok(res) => {
            log!("Service worker registered:", res);
        }
        Err(e) => {
            log!("Service worker registration failed:", e);
        }
    }
}

pub async fn init_service_worker() {
    let sw = sw();
    install_service_worker(&sw).await;
}

pub fn update_document_title(title: &str) {
    document().set_title(title);

    let closure: Function = Closure::<dyn FnMut()>::new(move || {
        document().set_title("Portal SALUD");
    })
    .into_js_value()
    .into();
    window()
        .add_event_listener_with_callback("focus", &closure)
        .unwrap();
}

pub fn play_notification_sound() {
    let audio = document().get_element_by_id("notification-sound").unwrap();
    let audio: web_sys::HtmlAudioElement = audio.dyn_into().unwrap();
    let _ = audio.play().unwrap();
}

pub fn get_object_key(event: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    let key = JsValue::from_str(key);
    if js_sys::Reflect::has(&event, &key).unwrap() {
        let value = js_sys::Reflect::get(&event, &key).unwrap();
        Ok(value)
    } else {
        Err(JsValue::from_str("Key not found"))
    }
}
