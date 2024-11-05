use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlFormElement, HtmlInputElement, HtmlSelectElement, SubmitEvent};

pub struct HtmlDocument {
    document: web_sys::Document,
}
impl HtmlDocument {
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or(JsValue::from_str("No window available"))?;
        let document = window
            .document()
            .ok_or(JsValue::from_str("No document available"))?;
        Ok(Self { document })
    }
    pub fn find_element_by_id<T>(&self, id: &str) -> Result<T, JsValue>
    where
        T: JsCast,
    {
        self.document
            .get_element_by_id(id)
            .ok_or(JsValue::from_str("Element not found"))?
            .dyn_into::<T>()
            .map_err(|_| JsValue::from_str("Failed to cast element"))
    }
    pub fn query_selector<T>(&self, selector: &str) -> Result<T, JsValue>
    where
        T: JsCast,
    {
        self.document
            .query_selector(selector)?
            .ok_or(JsValue::from_str("Elements not found"))?
            .dyn_into::<T>()
            .map_err(|_| JsValue::from_str("Failed to cast element"))
    }
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
