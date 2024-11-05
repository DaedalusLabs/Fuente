use js_sys::{Function, Object, Reflect};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::Element;

use js_sys::Date;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type CalendarEvent;
    #[wasm_bindgen(method, js_name = remove)]
    pub fn remove(this: &CalendarEvent);
}
#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type Calendar;
    #[wasm_bindgen(constructor, js_namespace = FullCalendar, js_name = Calendar)]
    pub fn new(calendar_element: &Element, options: JsValue) -> Calendar;
    #[wasm_bindgen(method, js_name = render)]
    pub fn render(this: &Calendar);
    #[wasm_bindgen(method, js_name = addEvent)]
    fn add_event(this: &Calendar, event: JsValue);
    #[wasm_bindgen(method, js_name = getEventById)]
    pub fn get_event_by_id(this: &Calendar, id: &str) -> Option<CalendarEvent>;
    #[wasm_bindgen(method, js_name = getEvents)]
    pub fn get_events(this: &Calendar) -> Vec<CalendarEvent>;
    #[wasm_bindgen(method, js_name = changeView)]
    pub fn change_view(this: &Calendar, view: &str, date: &str);
}
impl Calendar {
    pub fn remove_event(&self, id: &str) {
        if let Some(event) = self.get_event_by_id(id) {
            event.remove();
        }
    }
    pub fn clear_events(&self) {
        let events = self.get_events();
        for event in events {
            event.remove();
        }
    }
    pub fn add_or_replace_event(&self, event: FullCalendarEvent) {
        if let Some(old_event) = self.get_event_by_id(&event.get_id()) {
            old_event.remove();
        }
        let js_vale: JsValue = event.into();
        self.add_event(js_vale);
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct FullCalendarHeaderOptions {
    start: &'static str,
    center: &'static str,
    end: &'static str,
}
#[derive(Debug, Clone, Serialize)]
pub struct FullCalendarOptions {
    #[serde(rename = "initialView")]
    pub intial_view: &'static str,
    pub locale: &'static str,
    #[serde(rename = "expandRows")]
    pub expand_rows: bool,
    #[serde(rename = "allDaySlot")]
    pub all_day_slot: bool,
    pub selectable: bool,
    #[serde(rename = "firstDay")]
    pub first_day: u32,
    #[serde(rename = "slotDuration")]
    pub slot_duration: String,
    #[serde(rename = "headerToolbar")]
    pub header_toolbar: FullCalendarHeaderOptions,
    #[serde(rename = "selectLongPressDelay")]
    pub select_long_press_delay: u32,
}
impl FullCalendarOptions {
    pub fn event_closure<F, T>(self, function: F) -> JsValue
    where
        F: Fn(T) + 'static,
        T: wasm_bindgen::convert::FromWasmAbi + 'static,
    {
        let closure = Closure::<dyn Fn(_)>::new(function);
        let function: Function = closure.as_ref().dyn_ref::<Function>().unwrap().clone();
        closure.forget();
        let js_value: JsValue = self.into();
        let js_object: Object = js_value.into();
        let property = JsValue::from_str("eventClick");
        let _ = Reflect::set(&js_object, &property, &function.into());
        js_object.into()
    }
    pub fn select_closure<F, T>(self, function: F) -> JsValue
    where
        F: Fn(T) + 'static,
        T: wasm_bindgen::convert::FromWasmAbi + 'static,
    {
        let closure = Closure::<dyn Fn(_)>::new(function);
        let function: Function = closure.as_ref().dyn_ref::<Function>().unwrap().clone();
        closure.forget();
        let js_value: JsValue = self.into();
        let js_object: Object = js_value.into();
        let property = JsValue::from_str("select");
        let _ = Reflect::set(&js_object, &property, &function.into());
        js_object.into()
    }
    pub fn date_closure<F, T>(self, function: F) -> JsValue
    where
        F: Fn(T) + 'static,
        T: wasm_bindgen::convert::FromWasmAbi + 'static,
    {
        let closure = Closure::<dyn Fn(_)>::new(function);
        let function: Function = closure.as_ref().dyn_ref::<Function>().unwrap().clone();
        closure.forget();
        let js_value: JsValue = self.into();
        let js_object: Object = js_value.into();
        let property = JsValue::from_str("dateClick");
        let _ = Reflect::set(&js_object, &property, &function.into());
        js_object.into()
    }
}
impl Into<JsValue> for FullCalendarOptions {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self)
            .expect("Failed to convert FullCalendarOptions to JsValue")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullCalendarSelectEvent {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub start: Date,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub end: Date,
    #[serde(rename = "startStr")]
    pub start_str: String,
    #[serde(rename = "endStr")]
    pub end_str: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullCalendarEvent {
    id: String,
    title: String,
    #[serde(rename = "allDay")]
    all_day: bool,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    start: js_sys::Date,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    end: js_sys::Date,
    #[serde(rename = "startStr")]
    start_str: String,
    #[serde(rename = "endStr")]
    end_str: String,
    #[serde(rename = "backgroundColor")]
    background_color: String,
    #[serde(rename = "extendedProps")]
    extended_props: Value,
    display: String,
}
impl FullCalendarEvent {
    pub fn new(
        id: &str,
        title: &str,
        start: Date,
        end: Date,
        color: &str,
        extended_props: Value,
    ) -> Self {
        let locale_options: JsValue = js_sys::Object::new().into();

        FullCalendarEvent {
            id: id.to_string(),
            title: title.to_string(),
            all_day: false,
            start: start.clone(),
            end: end.clone(),
            background_color: color.to_string(),
            start_str: start.to_locale_string("es-ES", &locale_options).into(),
            end_str: end.to_locale_string("es-ES", &locale_options).into(),
            display: "block".to_string(),
            extended_props,
        }
    }
    pub fn new_background_event(
        id: &str,
        title: &str,
        start: Date,
        end: Date,
        color: &str,
        extended_props: Value,
    ) -> Self {
        let locale_options: JsValue = js_sys::Object::new().into();

        FullCalendarEvent {
            id: id.to_string(),
            title: title.to_string(),
            all_day: false,
            start: start.clone(),
            end: end.clone(),
            background_color: color.to_string(),
            start_str: start.to_locale_string("es-ES", &locale_options).into(),
            end_str: end.to_locale_string("es-ES", &locale_options).into(),
            display: "background".to_string(),
            extended_props,
        }
    }
    pub fn from_event(event: &web_sys::Event) -> Result<Self, JsValue> {
        let js_value: JsValue = event.into();
        let js_object: Object = js_value.into();
        let event_object = Reflect::get(&js_object, &JsValue::from_str("event"))?;
        Ok(event_object.into())
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_dates(&self) -> (Date, Date) {
        (self.start.clone(), self.end.clone())
    }
}
impl Into<JsValue> for FullCalendarEvent {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).expect("Failed to convert FullCalendarEvent to JsValue")
    }
}
impl From<JsValue> for FullCalendarEvent {
    fn from(value: JsValue) -> Self {
        serde_wasm_bindgen::from_value(value)
            .expect("Failed to convert JsValue to FullCalendarEvent")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullCalendarDateClickInfo {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    date: Date,
    #[serde(rename = "dateStr")]
    date_str: String,
}
impl FullCalendarDateClickInfo {
    pub fn date_str(&self) -> String {
        self.date_str.clone()
    }
}

impl Into<JsValue> for FullCalendarDateClickInfo {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self)
            .expect("Failed to convert FullCalendarDateClickInfo to JsValue")
    }
}

impl From<JsValue> for FullCalendarDateClickInfo {
    fn from(value: JsValue) -> Self {
        serde_wasm_bindgen::from_value(value)
            .expect("Failed to convert JsValue to FullCalendarDateClickInfo")
    }
}
