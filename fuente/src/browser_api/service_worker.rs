use wasm_bindgen::JsValue;

pub struct AppServiceWorker {
    sw: web_sys::ServiceWorkerContainer,
}
impl AppServiceWorker {
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let sw = window.navigator().service_worker();
        Ok(Self { sw })
    }
    pub async fn install(&self, file_path: &str) -> Result<(), JsValue> {
        let register = self.sw.register(file_path);
        let register = wasm_bindgen_futures::JsFuture::from(register).await?;
        gloo::console::info!(register);
        Ok(())
    }
}


