use js_sys::Object;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Date {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    date: js_sys::Date,
}

impl Date {
    pub fn value_of(&self) -> f64 {
        self.date.value_of()
    }
    pub fn locale_date_string(&self) -> String {
        let locale_opts = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &locale_opts,
            &JsValue::from_str("localeMatcher"),
            &JsValue::from_str("best fit"),
        );
        self.date.to_locale_date_string("es", &locale_opts).into()
    }
    pub fn locale_time_string(&self) -> String {
        self.date.to_locale_time_string("es").into()
    }
    pub fn new(date: Option<String>) -> Self {
        let date = match date {
            Some(date) => js_sys::Date::new(&JsValue::from_str(&date)),
            None => js_sys::Date::new_0(),
        };
        Self { date }
    }
    pub fn get_date(&self) -> u32 {
        self.date.get_date()
    }
    pub fn get_day(&self) -> u32 {
        self.date.get_day()
    }
    pub fn get_full_year(&self) -> u32 {
        self.date.get_full_year()
    }
    pub fn get_hours(&self) -> u32 {
        self.date.get_hours()
    }
    pub fn get_iso_string(&self) -> String {
        self.date.to_iso_string().into()
    }
    pub fn get_minutes(&self) -> u32 {
        self.date.get_minutes()
    }
    pub fn get_month(&self) -> u32 {
        self.date.get_month()
    }
    pub fn get_seconds(&self) -> u32 {
        self.date.get_seconds()
    }
    pub fn set_hours(&self, hours: u32) {
        self.date.set_utc_hours(hours);
    }
    pub fn get_offset(&self) -> f64 {
        self.date.get_timezone_offset()
    }
    pub fn to_iso_string(&self) -> String {
        self.date.to_iso_string().into()
    }
    pub fn to_locale_date_string(&self) -> String {
        self.date.to_locale_date_string("es", &Object::new().into()).into()
    }
    pub fn to_locale_time_string(&self) -> String {
        self.date.to_locale_time_string("es").into()
    }
    pub fn to_locale_string(&self) -> String {
        self.date.to_locale_string("es", &Object::new().into()).into()
    }
    pub fn get_time_input_string(&self) -> String {
        let hour = self.get_hours();
        let minute = self.get_minutes();
        format!("{:02}:{:02}", hour, minute)
    }
    pub fn to_string(&self) -> String {
        self.date.to_string().into()
    }
    pub fn now() -> Self {
        let date = js_sys::Date::new_0();
        Self { date }
    }
}

impl From<JsValue> for Date {
    fn from(val: JsValue) -> Self {
        Self {
            date: js_sys::Date::from(val),
        }
    }
}
impl Into<JsValue> for Date {
    fn into(self) -> JsValue {
        self.date.into()
    }
}
impl AsRef<wasm_bindgen::JsValue> for Date {
    fn as_ref(&self) -> &wasm_bindgen::JsValue {
        self.date.as_ref()
    }
}

impl JsCast for Date {
    fn instanceof(val: &JsValue) -> bool {
        js_sys::Date::instanceof(val)
    }
    fn unchecked_from_js(val: JsValue) -> Self {
        Self {
            date: js_sys::Date::unchecked_from_js(val),
        }
    }
    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}
