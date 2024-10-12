use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize)]
pub struct ToastifyOptions {
    text: String,
    duration: u32,
    close: bool,
    gravity: &'static str,  // `top` or `bottom`
    position: &'static str, // `left`, `center` or `right`
    #[serde(rename = "stopOnFocus")]
    stop_on_focus: bool, // Prevents dismissing of toast on hover
    #[serde(rename = "className")]
    class_name: &'static str,
}

impl ToastifyOptions {
    pub fn show(self) {
        let options: JsValue = self.into();
        toasts(&options).show_toast();
    }
    pub fn new_login(text: String) -> Self {
        ToastifyOptions {
            text,
            duration: 21000,
            close: true,
            gravity: "top",
            position: "left",
            stop_on_focus: true,
            class_name: "success-toast",
        }
    }
    pub fn new_success(text: &'static str) -> Self {
        ToastifyOptions {
            text: text.to_string(),
            duration: 2100,
            close: true,
            gravity: "top",
            position: "left",
            stop_on_focus: true,
            class_name: "success-toast",
        }
    }
    pub fn new_failure(text: &'static str) -> Self {
        ToastifyOptions {
            text: text.to_string(),
            duration: 2100,
            close: true,
            gravity: "top",
            position: "left",
            stop_on_focus: true,
            class_name: "failure-toast",
        }
    }
}

impl Into<JsValue> for ToastifyOptions {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).expect("Failed to serialize Toast")
    }
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type Toastify;
    #[wasm_bindgen(js_namespace = window, js_name = Toastify)]
    pub fn toasts(options: &JsValue) -> Toastify;
    #[wasm_bindgen(method, js_name = showToast)]
    pub fn show_toast(this: &Toastify);
}
