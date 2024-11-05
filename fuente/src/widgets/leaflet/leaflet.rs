use crate::browser_api::{GeolocationCoordinates, GeolocationPosition};
use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use yew::MouseEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}
impl TryFrom<&MouseEvent> for LatLng {
    type Error = JsValue;
    fn try_from(event: &MouseEvent) -> Result<Self, Self::Error> {
        let lat_lng = js_sys::Reflect::get(event, &"latlng".into())?;
        Ok(lat_lng.try_into()?)
    }
}
impl TryFrom<MouseEvent> for LatLng {
    type Error = JsValue;
    fn try_from(event: MouseEvent) -> Result<Self, Self::Error> {
        let lat_lng = js_sys::Reflect::get(&event, &"latlng".into())?;
        Ok(lat_lng.try_into()?)
    }
}
impl TryInto<JsValue> for LatLng {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
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
            accuracy: 0.0,
            altitude: None,
            altitude_accuracy: None,
            latitude: self.lat,
            longitude: self.lng,
            speed: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafletMapOptions {
    pub zoom: u8,
    #[serde(rename = "zoomControl")]
    pub zoom_control: bool,
    #[serde(rename = "scrollWheelZoom")]
    pub scroll_wheel_zoom: bool,
    #[serde(rename = "doubleClickZoom")]
    pub double_click_zoom: bool,
    #[serde(rename = "dragging")]
    pub dragging: bool,
    pub center: Option<LatLng>,
    #[serde(rename = "minZoom")]
    pub min_zoom: Option<u8>,
    #[serde(rename = "maxZoom")]
    pub max_zoom: Option<u8>,
}

impl Default for LeafletMapOptions {
    fn default() -> Self {
        Self {
            zoom: 13,
            zoom_control: true,
            scroll_wheel_zoom: true,
            double_click_zoom: true,
            dragging: true,
            center: None,
            min_zoom: None,
            max_zoom: None,
        }
    }
}

impl TryInto<JsValue> for LeafletMapOptions {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileLayerOptions {
    pub attribution: String,
    #[serde(rename = "maxZoom")]
    pub max_zoom: Option<u8>,
    #[serde(rename = "minZoom")]
    pub min_zoom: Option<u8>,
    pub opacity: f64,
}

impl Default for TileLayerOptions {
    fn default() -> Self {
        Self {
            attribution: "&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors".to_string(),
            max_zoom: Some(19),
            min_zoom: None,
            opacity: 1.0,
        }
    }
}

impl TryInto<JsValue> for TileLayerOptions {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
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
    #[wasm_bindgen(static_method_of = L, js_name = "map")]
    pub fn map_with_options(id: &str, options: JsValue) -> LeafletMap;
}
impl L {
    pub fn render_default_map(id: &str, coords: &GeolocationCoordinates) -> Result<LeafletMap, JsValue> {
        let lat_lng: LatLng = coords.into();
        let mut map_options = LeafletMapOptions::default();
        map_options.center = Some(lat_lng.clone());
        let js_options: JsValue = map_options.try_into()?;
        let map = L::map_with_options(id, js_options);
        let tile_options = TileLayerOptions::default();
        let js_tile_options: JsValue = tile_options.try_into()?;

        L::tile_layer(
            "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
            js_tile_options,
        )
        .addTo(&map);
        Ok(map)
    }
    pub fn render_map_with_options(
        id: &str,
        map_options: LeafletMapOptions,
    ) -> Result<LeafletMap, JsValue> {
        let js_options: JsValue = map_options.try_into()?;
        let map = L::map_with_options(id, js_options);
        let tile_options = TileLayerOptions::default();
        let js_tile_options: JsValue = tile_options.try_into()?;
        L::tile_layer(
            "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
            js_tile_options,
        )
        .addTo(&map);
        Ok(map)
    }
}
#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type LeafletMap;
    pub type Control;
    pub type TileLayer;

    #[wasm_bindgen(constructor, js_namespace = L, js_name = map)]
    pub fn map(id: &str) -> LeafletMap;
    #[wasm_bindgen(method, js_name = setView)]
    pub fn set_view(this: &LeafletMap, coords: &JsValue, zoom: u8);
    #[wasm_bindgen(method, structural, indexing_getter)]
    pub fn get(this: &LeafletMap, prop: &str) -> Control;
    #[wasm_bindgen(method)]
    pub fn on(this: &LeafletMap, event: &str, callback: Function);

    #[wasm_bindgen(method)]
    pub fn getZoom(this: &LeafletMap) -> f64;
    #[wasm_bindgen(method)]
    pub fn setZoom(this: &LeafletMap, zoom: f64);
    #[wasm_bindgen(method)]
    pub fn zoomIn(this: &LeafletMap);
    #[wasm_bindgen(method)]
    pub fn zoomOut(this: &LeafletMap);
    // Pane methods
    #[wasm_bindgen(method, js_name = "createPane")]
    pub fn create_pane(this: &LeafletMap, name: &str);
    #[wasm_bindgen(method, js_name = "getPane")]
    pub fn get_pane(this: &LeafletMap, name: &str) -> web_sys::Element;

    #[wasm_bindgen(method)]
    pub fn disable(this: &Control);
    #[wasm_bindgen(method)]
    pub fn addTo(this: &TileLayer, map: &LeafletMap);

    // Geolocation methods
    #[wasm_bindgen(method, js_name = "locate")]
    pub fn locate(this: &LeafletMap);

    #[wasm_bindgen(method, js_name = "locate")]
    pub fn locate_with_options(this: &LeafletMap, options: JsValue);

    #[wasm_bindgen(method, js_name = "stopLocate")]
    pub fn stop_locate(this: &LeafletMap);

    #[wasm_bindgen(method, js_name = "watchLocation")]
    pub fn watch_location(this: &LeafletMap);

    #[wasm_bindgen(method, js_name = "fitBounds")]
    pub fn fit_bounds(this: &LeafletMap, bounds: JsValue);
}

// Add a struct for locate options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafletLocateOptions {
    pub watch: bool,
    pub set_view: bool,
    pub max_zoom: f64,
    pub timeout: u32,
    pub maximum_age: u32,
    pub enable_high_accuracy: bool,
}

impl Default for LeafletLocateOptions {
    fn default() -> Self {
        Self {
            watch: false,
            set_view: true,
            max_zoom: 16.0,
            timeout: 10000,
            maximum_age: 0,
            enable_high_accuracy: false,
        }
    }
}

impl TryInto<JsValue> for LeafletLocateOptions {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone)]
    pub type Icon;

    #[wasm_bindgen(static_method_of = L, js_name = "icon")]
    pub fn create_icon(options: &JsValue) -> Icon;
}

impl LeafletMap {
    pub fn add_leaflet_marker(&self, coords: &GeolocationCoordinates) -> Result<Marker, JsValue> {
        let lat_lng: LatLng = coords.into();
        let new_coords: JsValue = lat_lng.try_into()?;
        let marker_options = LeafletMarkerOptions::default();
        let marker = L::marker(&new_coords, marker_options.try_into()?).addTo(self);
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

    pub fn zoom_level(&self) -> f64 {
        self.getZoom()
    }
    pub fn set_zoom_level(&self, zoom: f64) {
        self.setZoom(zoom)
    }
    pub fn zoom_in(&self) {
        self.zoomIn()
    }
    pub fn zoom_out(&self) {
        self.zoomOut()
    }

    pub fn create_map_pane(&self, name: &str) {
        self.create_pane(name)
    }
    pub fn get_map_pane(&self, name: &str) -> Option<web_sys::Element> {
        match self.get_pane(name) {
            pane if pane.is_undefined() => None,
            pane => Some(pane),
        }
    }

    pub fn setup_location_tracking(&self, marker: &Marker) {
        let marker = marker.clone();
        self.add_closure("locationfound", move |event: JsValue| {
            web_sys::console::log_1(&"Location found event received".into());

            // Try to convert JsValue to our GeolocationPosition
            if let Ok(position) = GeolocationPosition::try_from(event) {
                let geo_coords = position.coords;
                let lat_lng = LatLng {
                    lat: geo_coords.latitude,
                    lng: geo_coords.longitude,
                };

                if let Ok(js_coords) = lat_lng.try_into() {
                    marker.set_lat_lng(&js_coords);
                }
            }
        });
    }

    pub fn start_locate(&self, options: Option<LeafletLocateOptions>) {
        web_sys::console::log_1(&"Starting location tracking...".into());
        match options {
            Some(opts) => {
                if let Ok(js_opts) = opts.try_into() {
                    self.locate_with_options(js_opts);
                    web_sys::console::log_1(&"Location tracking started with options".into());
                }
            }
            None => {
                self.locate();
                web_sys::console::log_1(&"Location tracking started without options".into());
            }
        }
    }

    pub fn stop_location_watch(&self) {
        web_sys::console::log_1(&"Stopping location tracking...".into());
        self.stop_locate();
    }
    pub fn add_marker_with_icon(
        &self,
        coords: &GeolocationCoordinates,
        icon_options: IconOptions,
    ) -> Result<Marker, JsValue> {
        let lat_lng: LatLng = coords.into();
        let new_coords: JsValue = lat_lng.try_into()?;

        // Create icon
        let icon_js = icon_options.try_into()?;
        let icon = L::create_icon(&icon_js);

        // Create marker options
        let marker_options: JsValue = LeafletMarkerOptions::default().try_into()?;
        js_sys::Reflect::set(&marker_options, &"icon".into(), &icon)?;

        let marker = L::marker(&new_coords, marker_options).addTo(self);
        Ok(marker)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconOptions {
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    #[serde(rename = "iconSize")]
    pub icon_size: Option<Vec<i32>>, // [width, height]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "iconAnchor")]
    pub icon_anchor: Option<Vec<i32>>, // [x, y]
}

impl Default for IconOptions {
    fn default() -> Self {
        Self {
            icon_url: "/default-marker.png".to_string(),
            icon_size: Some(vec![25, 41]), // Default Leaflet marker size
            icon_anchor: Some(vec![12, 41]), // Default Leaflet marker anchor
        }
    }
}

impl TryInto<JsValue> for IconOptions {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
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
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafletMarkerOptions {
    pub draggable: bool,
    #[serde(rename = "autoPan")]
    pub auto_pan: bool,
    #[serde(rename = "title")]
    pub title: Option<String>,
    #[serde(rename = "alt")]
    pub alt: Option<String>,
    #[serde(rename = "opacity")]
    pub opacity: f64,
    #[serde(rename = "riseOnHover")]
    pub rise_on_hover: bool,
}
impl Default for LeafletMarkerOptions {
    fn default() -> Self {
        Self {
            draggable: false,
            auto_pan: true,
            title: None,
            alt: None,
            opacity: 1.0,
            rise_on_hover: true,
        }
    }
}
impl TryInto<JsValue> for LeafletMarkerOptions {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}
impl TryFrom<JsValue> for LeafletMarkerOptions {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
