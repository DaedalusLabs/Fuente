use gloo::utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};

use crate::browser::{geolocation::GeolocationCoordinates, web_utils::get_object_key};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafletCoordinates {
    lat: f64,
    lng: f64,
}
impl LeafletCoordinates {
    pub fn from_event(event: web_sys::MouseEvent) -> Self {
        let object = event.dyn_ref::<JsValue>().unwrap();
        let leaflet_event: LeafletCoordinates = get_object_key(object, "latlng")
            .unwrap()
            .into_serde()
            .unwrap();
        leaflet_event
    }
    pub fn lat_lng(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&[self.lat, self.lng]).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CoordinateStrings {
    pub latitude: String,
    pub longitude: String,
}
impl Default for CoordinateStrings {
    fn default() -> Self {
        CoordinateStrings {
            latitude: "".to_string(),
            longitude: "".to_string(),
        }
    }
}
impl From<GeolocationCoordinates> for CoordinateStrings {
    fn from(coords: GeolocationCoordinates) -> Self {
        CoordinateStrings {
            latitude: coords.latitude.to_string(),
            longitude: coords.longitude.to_string(),
        }
    }
}
impl Into<GeolocationCoordinates> for CoordinateStrings {
    fn into(self) -> GeolocationCoordinates {
        GeolocationCoordinates {
            latitude: self.latitude.parse().unwrap(),
            longitude: self.longitude.parse().unwrap(),
            speed: None,
            accuracy: 0.0,
            altitude_accuracy: None,
            altitude: None,
        }
    }
}

impl From<LeafletCoordinates> for CoordinateStrings {
    fn from(coords: LeafletCoordinates) -> Self {
        CoordinateStrings {
            latitude: coords.lat.to_string(),
            longitude: coords.lng.to_string(),
        }
    }
}

impl Into<LeafletCoordinates> for CoordinateStrings {
    fn into(self) -> LeafletCoordinates {
        LeafletCoordinates {
            lat: self.latitude.parse().unwrap(),
            lng: self.longitude.parse().unwrap(),
        }
    }
}
