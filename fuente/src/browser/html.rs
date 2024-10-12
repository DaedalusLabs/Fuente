use crate::browser::web_utils::document;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlFormElement, HtmlInputElement, HtmlSelectElement, SubmitEvent};
use yew::{Callback, UseStateHandle};
pub fn find_element_by_id<T>(id: &str) -> Result<T, JsValue>
where
    T: JsCast,
{
    document()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<T>()
        .map_err(|_| JsValue::from_str("Failed to cast element"))
}
pub fn query_selector<T>(selector: &str) -> Result<T, JsValue>
where
    T: JsCast,
{
    document()
        .query_selector(selector)?
        .unwrap()
        .dyn_into::<T>()
        .map_err(|_| JsValue::from_str("Failed to cast element"))
}
pub fn html_input_value(event: UseStateHandle<String>) -> Callback<Event> {
    Callback::from(move |e: Event| {
        let value = e
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value();
        event.set(value);
    })
}

pub struct HtmlForm {
    form: HtmlFormElement,
}
impl HtmlForm {
    pub fn new(submit_event: SubmitEvent) -> Result<Self, JsValue> {
        let form = submit_event.target();
        if form.is_none() {
            return Err(JsValue::from_str("Form not found"));
        }
        let form = form.unwrap().dyn_into::<HtmlFormElement>()?;
        Ok(HtmlForm { form })
    }
    pub fn input<T>(&self, name: &str) -> Result<T, JsValue> 
    where
        T: JsCast,
    {
        let input = self.form.get_with_name(name);
        if input.is_none() {
            return Err(JsValue::from_str("Input not found"));
        }
        Ok(input.unwrap().dyn_into::<T>()?)
    }
    pub fn input_value(&self, name: &str) -> Result<String, JsValue> {
        Ok(self.input::<HtmlInputElement>(name)?.value())
    }
    pub fn select_value(&self, name: &str) -> Result<String, JsValue> {
        Ok(self.input::<HtmlSelectElement>(name)?.value())
    }
    pub fn textarea_value(&self, name: &str) -> Result<String, JsValue> {
        Ok(self.input::<web_sys::HtmlTextAreaElement>(name)?.value())
    }
}
pub fn html_onchange_select_value(event: Event) -> Result<String, JsValue> {
    let select = event
        .target()
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .map_err(|_| JsValue::from_str("Failed to cast element"))?;
    Ok(select.value())
}
pub fn html_input_id_value(element_id: &str) -> Result<String, JsValue> {
    let input = find_element_by_id::<HtmlInputElement>(element_id)?;
    Ok(input.value())
}
pub fn html_select_id_value(element_id: &str) -> Result<String, JsValue> {
    let select = find_element_by_id::<HtmlSelectElement>(element_id)?;
    Ok(select.value())
}
pub fn html_textarea_id_value(element_id: &str) -> Result<String, JsValue> {
    let textarea = find_element_by_id::<web_sys::HtmlTextAreaElement>(element_id)?;
    Ok(textarea.value())
}
