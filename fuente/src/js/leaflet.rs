use js_sys::{Function, Object};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use yew::MouseEvent;

use crate::browser::geolocation::GeolocationCoordinates;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}
impl TryFrom<MouseEvent> for LatLng {
    type Error = JsValue;
    fn try_from(value: MouseEvent) -> Result<Self, Self::Error> {
        let coords = js_sys::Reflect::get(&value, &"latlng".into())?;
        Ok(coords.try_into()?)
    }
}
impl Into<JsValue> for LatLng {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<JsValue> for LatLng {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl From<&GeolocationCoordinates> for LatLng {
    fn from(value: &GeolocationCoordinates) -> Self {
        Self {
            lat: value.latitude,
            lng: value.longitude,
        }
    }
}
impl From<GeolocationCoordinates> for LatLng {
    fn from(value: GeolocationCoordinates) -> Self {
        Self {
            lat: value.latitude,
            lng: value.longitude,
        }
    }
}
impl Into<GeolocationCoordinates> for LatLng {
    fn into(self) -> GeolocationCoordinates {
        GeolocationCoordinates {
            latitude: self.lat,
            longitude: self.lng,
            altitude: None,
            accuracy: 0.0,
            altitude_accuracy: None,
            speed: None,
        }
    }
}
impl From<&LatLng> for GeolocationCoordinates {
    fn from(value: &LatLng) -> Self {
        GeolocationCoordinates {
            latitude: value.lat,
            longitude: value.lng,
            altitude: None,
            accuracy: 0.0,
            altitude_accuracy: None,
            speed: None,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafletIconOptions {
    #[serde(rename = "iconUrl")]
    icon_url: String,
    #[serde(rename = "iconSize")]
    icon_size: Vec<u8>,
}
impl Into<JsValue> for LeafletIconOptions {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<JsValue> for LeafletIconOptions {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl LeafletIconOptions {
    pub fn new(icon_url: &str, icon_size: Vec<u8>) -> Self {
        Self {
            icon_url: icon_url.to_string(),
            icon_size,
        }
    }
}
#[wasm_bindgen]
extern "C" {
    pub type L;
    #[wasm_bindgen(static_method_of = L)]
    pub fn map(id: &str) -> LeafletMap;
    #[wasm_bindgen(static_method_of = L, js_name = tileLayer)]
    pub fn tile_layer(url: &str, options: JsValue) -> TileLayer;
    #[wasm_bindgen(static_method_of = L, js_name = marker)]
    pub fn marker(coords: &JsValue, options: JsValue) -> NewMarker;
    #[wasm_bindgen(static_method_of = L, js_name = icon)]
    pub fn icon(options: JsValue) -> Icon;

    #[derive(Debug, Clone)]
    pub type Icon;
}
impl L {
    pub fn render_map(id: &str, coords: &GeolocationCoordinates) -> Result<LeafletMap, JsValue> {
        let lat_lng: LatLng = coords.into();
        let new_coords: JsValue = lat_lng.into();
        let map = L::map(id);
        map.get("doubleClickZoom").disable();
        map.set_view(&new_coords, 13);
        let map_options: JsValue = Object::new().into();
        L::tile_layer(
            "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
            map_options,
        )
        .addTo(&map);
        Ok(map)
    }
}
#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type LeafletMap;
    #[wasm_bindgen(constructor, js_namespace = L, js_name = map)]
    pub fn map(id: &str) -> LeafletMap;
    #[wasm_bindgen(method, js_name = setView)]
    pub fn set_view(this: &LeafletMap, coords: &JsValue, zoom: u8);
    #[wasm_bindgen(method, structural, indexing_getter)]
    pub fn get(this: &LeafletMap, prop: &str) -> Control;
    #[wasm_bindgen(method)]
    pub fn on(this: &LeafletMap, event: &str, callback: Function);
    #[wasm_bindgen(method)]
    pub fn remove(this: &LeafletMap);
    #[wasm_bindgen(method, js_name = fitBounds)]
    pub fn fit_bounds(this: &LeafletMap, coords: &JsValue);

    pub type Control;
    #[wasm_bindgen(method)]
    pub fn disable(this: &Control);

    pub type TileLayer;
    #[wasm_bindgen(method)]
    pub fn addTo(this: &TileLayer, map: &LeafletMap);


}
impl LeafletMap {
    pub fn add_leaflet_marker(&self, coords: &GeolocationCoordinates) -> Result<Marker, JsValue> {
        let lat_lng: LatLng = coords.into();
        let new_coords: JsValue = lat_lng.into();
        let marker_options = LeafletMarkerOptions::default();
        let marker = L::marker(&new_coords, marker_options.into()).addTo(self);
        Ok(marker)
    }
    pub fn add_custom_marker(&self, coords: &GeolocationCoordinates, icon_url: &str) -> Result<Marker, JsValue> {
        let lat_lng: LatLng = coords.into();
        let new_coords: JsValue = lat_lng.into();
        let icon_options = LeafletIconOptions::new(icon_url, vec![25, 41]);
        let icon = L::icon(icon_options.into());
        let marker_options = LeafletMarkerOptions {
            draggable: false,
            auto_pan: true,
            icon,
        };
        let marker = L::marker(&new_coords, marker_options.into()).addTo(self);
        marker.set_lat_lng(&new_coords);
        Ok(marker)
    }
    pub fn add_closure<T, A>(&self, event: &str, callback: T)
    where
        T: FnMut(A) + 'static,
        A: FromWasmAbi + 'static,
    {
        let map_closure = Closure::<dyn FnMut(A)>::new(callback);
        let map_function: Function = map_closure.into_js_value().into();
        self.on(event, map_function);
    }
}
#[wasm_bindgen]
extern "C" {
    pub type NewMarker;
    #[wasm_bindgen(method)]
    pub fn addTo(this: &NewMarker, map: &LeafletMap) -> Marker;

    #[derive(Debug, Clone, PartialEq)]
    pub type Marker;
    #[wasm_bindgen(method)]
    pub fn on(this: &Marker, event: &str, callback: Function);
    #[wasm_bindgen(method, js_name = setLatLng)]
    pub fn set_lat_lng(this: &Marker, coords: &JsValue) -> Marker;
    #[wasm_bindgen(method)]
    pub fn remove(this: &Marker);
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafletMarkerOptions {
    draggable: bool,
    #[serde(rename = "autoPan")]
    auto_pan: bool,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    icon: Icon,
}
impl Default for LeafletMarkerOptions {
    fn default() -> Self {
        Self {
            draggable: false,
            auto_pan: true,
            icon: L::icon(Object::new().into()),
        }
    }
}
impl Into<JsValue> for LeafletMarkerOptions {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<JsValue> for LeafletMarkerOptions {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}

