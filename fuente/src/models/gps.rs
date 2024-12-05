use serde::{Deserialize, Serialize};

use minions::browser_api::GeolocationCoordinates;

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
