#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GeolocationCoordinates {
    pub accuracy: f64,
    pub altitude: Option<f64>,
    #[serde(rename = "altitudeAccuracy")]
    pub altitude_accuracy: Option<f64>,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: Option<f64>,
}
impl Into<wasm_bindgen::JsValue> for GeolocationCoordinates {
    fn into(self) -> wasm_bindgen::JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl From<wasm_bindgen::JsValue> for GeolocationCoordinates {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        serde_wasm_bindgen::from_value(value).unwrap()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GeolocationPosition {
    pub coords: GeolocationCoordinates,
    pub timestamp: f64,
}
impl GeolocationPosition {
    fn default_position_options() -> web_sys::PositionOptions {
        let options = web_sys::PositionOptions::new();
        options.set_enable_high_accuracy(true);
        options.set_maximum_age(0);
        options
    }
    fn geolocation_api() -> anyhow::Result<web_sys::Geolocation> {
        let window = web_sys::window().ok_or(anyhow::anyhow!("No window available"))?;
        window
            .navigator()
            .geolocation()
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }
    fn log_error() -> Option<js_sys::Function> {
        Some(
            wasm_bindgen::closure::Closure::once_into_js(|e: web_sys::Geolocation| {
                gloo::console::error!("Error getting geolocation: {:?}", e);
            })
            .into(),
        )
    }
    pub async fn get_current_position() -> anyhow::Result<Self> {
        let geo_api = Self::geolocation_api()?;
        let (sender, receiver) = yew::platform::pinned::oneshot::channel::<GeolocationPosition>();
        let on_success: js_sys::Function =
            wasm_bindgen::closure::Closure::once_into_js(move |event: web_sys::Geolocation| {
                if let Ok(geo) = GeolocationPosition::try_from(event) {
                    let _ = sender.send(geo);
                }
            })
            .into();
        geo_api
            .get_current_position_with_error_callback_and_options(
                &on_success,
                Self::log_error().as_ref(),
                &Self::default_position_options(),
            )
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(receiver.await?)
    }
    pub fn watch_position<T>(closure: T) -> anyhow::Result<String>
    where
        T: FnMut(web_sys::Event) + 'static,
    {
        let geo_api = Self::geolocation_api()?;
        let onsuccess: js_sys::Function = wasm_bindgen::closure::Closure::new(closure)
            .into_js_value()
            .into();
        let watch_id = geo_api
            .watch_position_with_error_callback_and_options(
                &onsuccess,
                Self::log_error().as_ref(),
                &Self::default_position_options(),
            )
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(watch_id.to_string())
    }
}
impl TryFrom<wasm_bindgen::JsValue> for GeolocationPosition {
    type Error = anyhow::Error;
    fn try_from(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value).map_err(|e| anyhow::anyhow!("{:?}", e))?)
    }
}
impl TryInto<wasm_bindgen::JsValue> for GeolocationPosition {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<wasm_bindgen::JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self).map_err(|e| anyhow::anyhow!("{:?}", e))?)
    }
}
impl TryFrom<web_sys::Geolocation> for GeolocationPosition {
    type Error = anyhow::Error;
    fn try_from(value: web_sys::Geolocation) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value.into()).map_err(|e| anyhow::anyhow!("{:?}", e))?)
    }
}
