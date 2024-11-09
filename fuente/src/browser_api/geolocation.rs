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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeolocationPosition {
    pub coords: GeolocationCoordinates,
    pub timestamp: f64,
}
impl GeolocationPosition {
    async fn geolocation_api() -> Result<web_sys::Geolocation, wasm_bindgen::JsValue> {
        let window =
            web_sys::window().ok_or(wasm_bindgen::JsValue::from_str("No window available"))?;
        let geolocation = window.navigator().geolocation()?;
        Ok(geolocation)
    }
    pub async fn locate() -> Result<Self, wasm_bindgen::JsValue> {
        let geolocation = Self::geolocation_api().await?;
        let (sender, receiver) = yew::platform::pinned::oneshot::channel::<GeolocationPosition>();
        let on_success: js_sys::Function =
            wasm_bindgen::closure::Closure::once_into_js(move |event: web_sys::Geolocation| {
                if let Ok(geo) = GeolocationPosition::try_from(event) {
                    let _ = sender.send(geo);
                }
            })
            .into();
        geolocation.get_current_position(&on_success)?;
        receiver
            .await
            .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
    }
    pub async fn clear_watch(watch_id: i32) -> Result<(), wasm_bindgen::JsValue> {
        let geolocation = Self::geolocation_api().await?;
        geolocation.clear_watch(watch_id);
        Ok(())
    }
    pub async fn watch_position(
    ) -> Result<(i32, async_channel::Receiver<Self>), wasm_bindgen::JsValue> {
        let geolocation = Self::geolocation_api().await?;
        let (sender, receiver) = async_channel::unbounded();
        let on_success: js_sys::Function =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Geolocation| {
                if let Ok(geo) = GeolocationPosition::try_from(event) {
                    gloo::console::log!("Received geolocation: ", format!("{:?}", geo));
                    let _ = sender.try_send(geo);
                }
            })
                as Box<dyn FnMut(web_sys::Geolocation)>)
            .into_js_value()
            .into();
        let watch_id = geolocation.watch_position(&on_success)?;
        Ok((watch_id, receiver))
    }
}
impl TryFrom<wasm_bindgen::JsValue> for GeolocationPosition {
    type Error = wasm_bindgen::JsValue;
    fn try_from(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl TryInto<wasm_bindgen::JsValue> for GeolocationPosition {
    type Error = wasm_bindgen::JsValue;
    fn try_into(self) -> Result<wasm_bindgen::JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}
impl TryFrom<web_sys::Geolocation> for GeolocationPosition {
    type Error = wasm_bindgen::JsValue;
    fn try_from(value: web_sys::Geolocation) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value.into())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn __get_geolocation() {
        let position = GeolocationPosition::locate().await;
        assert!(position.is_ok());
    }
}
