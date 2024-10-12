use js_sys::{Function, Object, Reflect};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement};
use yew::DragEvent;

use crate::{browser::web_utils::document, models::orders::OrderStatus};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraggableOptions {
    draggable: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    dropzone: String,
}
impl Into<JsValue> for DraggableOptions {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl From<JsValue> for DraggableOptions {
    fn from(value: JsValue) -> Self {
        serde_wasm_bindgen::from_value(value).unwrap()
    }
}
impl DraggableOptions {
    pub fn new(draggable: String, dropzone: String) -> Self {
        Self {
            draggable,
            dropzone,
        }
    }
    pub fn new_no_window(draggable: String, dropzone: String) -> Self {
        Self {
            draggable,
            dropzone,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragEventData<T> {
    pub data: T,
}
impl<T> From<DragEvent> for DragEventData<T>
where
    T: Clone + PartialEq + Serialize + DeserializeOwned + 'static,
{
    fn from(value: DragEvent) -> Self {
        serde_wasm_bindgen::from_value(value.into()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DroppableStopEvent {
    #[serde(rename = "dragEvent")]
    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub drag_event: Object,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub dropzone: HtmlElement,
}
impl DroppableStopEvent {
    pub fn source(&self) -> HtmlElement {
        let data: Object = Reflect::get(&self.drag_event, &"data".into())
            .unwrap()
            .into();
        let source: HtmlElement = Reflect::get(&data, &"source".into()).unwrap().into();
        source
    }
    pub fn source_container(&self) -> Element {
        let data: Object = Reflect::get(&self.drag_event, &"data".into())
            .unwrap()
            .into();
        let source: HtmlElement = Reflect::get(&data, &"originalSource".into())
            .unwrap()
            .into();

        source.parent_element().unwrap()
    }
}
#[wasm_bindgen(js_namespace = Draggable)]
extern "C" {
    pub type Droppable;
    #[wasm_bindgen(constructor)]
    pub fn new(elements: Vec<HtmlElement>, items: JsValue) -> Droppable;
    #[wasm_bindgen(method)]
    pub fn on(this: &Droppable, event: &str, callback: &Function);

    pub type Sortable;
    #[wasm_bindgen(constructor)]
    pub fn new(elements: Vec<HtmlElement>, items: JsValue) -> Sortable;
    #[wasm_bindgen(method)]
    pub fn on(this: &Sortable, event: &str, callback: &Function);
}
impl Droppable {
    pub fn init(elements: &str, draggable: &str, droppable: &str) -> Result<Self, JsValue> {
        let elements: Vec<HtmlElement> = document()
            .query_selector_all(elements)?
            .values()
            .into_iter()
            .map(|js_value| js_value.unwrap().dyn_into::<HtmlElement>().unwrap())
            .collect();
        let options = DraggableOptions::new(draggable.to_string(), droppable.to_string());
        let new = Self::new(elements, options.into());
        new.event_listener();
        Ok(new)
    }
    pub fn event_listener(&self) {
        let callback: Function = Closure::wrap(Box::new(|event: DragEvent| {
            gloo::console::log!(&event);
            let stop_event: DragEventData<DroppableStopEvent> = event.into();
            let source_col =
                OrderStatus::try_from(stop_event.data.source_container().id()).unwrap();
            let target_col = OrderStatus::try_from(stop_event.data.dropzone.id()).unwrap();
            gloo::console::log!(format!(
                "{} -> {}",
                source_col.to_string(),
                target_col.to_string()
            ));
        }) as Box<dyn FnMut(DragEvent)>)
        .into_js_value()
        .into();
        // self.on("droppable:dropped", &callback);
        self.on("droppable:stop", &callback);
    }
}
impl Sortable {
    pub fn init(elements: &str, draggable: &str) -> Result<Self, JsValue> {
        let elements: Vec<HtmlElement> = document()
            .query_selector_all(elements)?
            .values()
            .into_iter()
            .map(|js_value| js_value.unwrap().dyn_into::<HtmlElement>().unwrap())
            .collect();
        let options = DraggableOptions::new(draggable.to_string(), "".to_string());
        let new = Self::new(elements, options.into());
        new.event_listener();
        Ok(new)
    }
    pub fn event_listener(&self) {
        let callback: Function = Closure::wrap(Box::new(|event: DragEvent| {
            gloo::console::log!(event);
        }) as Box<dyn FnMut(DragEvent)>)
        .into_js_value()
        .into();
        self.on("droppable:dropped", &callback);
    }
}
