use gloo::console::error;
use serde::de::DeserializeOwned;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{IdbObjectStore, IdbTransactionMode};
use yew::platform::pinned::oneshot::{self, Receiver};

use super::web_utils::js_array_to_vec;

pub trait IdbStoreManager {
    fn db_name() -> &'static str;
    fn db_version() -> u32;
    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue>;
    fn store_name() -> &'static str;
    fn document_key(&self) -> JsValue;
    fn save_to_store(self) -> Result<Receiver<()>, JsValue>
    where
        Self: TryInto<JsValue, Error = JsValue> + Sized,
    {
        let object_store_request = Self::request_store_open()?;
        let key = self.document_key();
        let js_value: JsValue = self.try_into()?;
        let (success_sender, success_receiver) = oneshot::channel();
        spawn_local(async move {
            if let Ok(request) = object_store_request.await {
                let request = request
                    .put_with_key(js_value.as_ref(), key.as_ref())
                    .unwrap();
                let req_clone = request.clone();
                let on_success = Closure::once_into_js(move |_event: web_sys::Event| {
                    let _result: JsValue = req_clone.result().unwrap();
                });
                request.set_onsuccess(Some(on_success.dyn_ref().unwrap()));
                let _ = success_sender.send(());
            }
        });

        Ok(success_receiver)
    }
    fn save_value_to_store(value: JsValue, key: &str) -> Result<Receiver<()>, JsValue> {
        let object_store_request = Self::request_store_open()?;
        let key = JsValue::from_str(key);
        let (sender, receiver) = oneshot::channel();
        spawn_local(async move {
            let object_store = object_store_request
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))
                .unwrap();
            let request = object_store.put_with_key(&value.into(), &key).unwrap();
            let req_clone = request.clone();
            let on_success = Closure::once_into_js(move |_event: web_sys::Event| {
                let _result: JsValue = req_clone.result().unwrap();
                let _ = sender.send(());
            });
            request.set_onsuccess(Some(on_success.dyn_ref().unwrap()));
        });
        Ok(receiver)
    }
    fn delete_from_store(&self) -> Result<Receiver<()>, JsValue> {
        let object_store_request = Self::request_store_open()?;
        let key = self.document_key();
        let (sender, receiver) = oneshot::channel();
        spawn_local(async move {
            let object_store = object_store_request
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))
                .unwrap();
            let request = object_store.delete(&key.as_ref()).unwrap();
            let req_clone = request.clone();
            let on_success = Closure::once_into_js(move |_event: web_sys::Event| {
                let _result: JsValue = req_clone.result().unwrap();
                let _ = sender.send(());
            });
            request.set_onsuccess(Some(on_success.dyn_ref().unwrap()));
        });
        Ok(receiver)
    }
    fn retrieve<T>(key: &str) -> Result<Receiver<T>, JsValue>
    where
        T: TryFrom<JsValue> + 'static,
    {
        let object_store_request = Self::request_store_open()?;
        let key = JsValue::from_str(key);
        let (success_sender, success_receiver) = oneshot::channel::<T>();
        spawn_local(async move {
            let (sender, receiver) = oneshot::channel();
            if let Ok(object_store) = object_store_request.await {
                let request = object_store.get(key.as_ref()).unwrap();
                let req_clone = request.clone();
                let on_success = Closure::once_into_js(move |_event: web_sys::Event| {
                    if let Ok(result) = req_clone.result() {
                        if result.is_null() || result.is_undefined() {
                            return;
                        }
                        if let Ok(result) = result.try_into() {
                            let _ = sender.send(result);
                        }
                    }
                });
                request.set_onsuccess(Some(on_success.dyn_ref().unwrap()));
                if let Ok(idb_doc) = receiver.await {
                    let _ = success_sender.send(idb_doc);
                }
            }
        });
        Ok(success_receiver)
    }
    fn retrieve_all_from_store<T>() -> Result<Receiver<Vec<T>>, JsValue>
    where
        T: TryFrom<JsValue, Error = JsValue> + 'static + DeserializeOwned,
    {
        let (sender, receiver) = oneshot::channel::<Vec<T>>();
        spawn_local({
            let object_store_request = Self::request_store_open().unwrap();
            async move {
                let object_store = object_store_request.await.unwrap();
                let request = object_store.get_all().unwrap();
                let req_clone = request.clone();
                let on_success = Closure::once_into_js(move |_event: web_sys::Event| {
                    let result: JsValue = req_clone.result().unwrap();
                    let result: Vec<T> = js_array_to_vec(result).unwrap();
                    let _ = sender.send(result.into());
                });
                request.set_onsuccess(Some(on_success.dyn_ref().unwrap()));
            }
        });
        Ok(receiver)
    }

    fn request_store_open() -> Result<Receiver<IdbObjectStore>, JsValue> {
        let idb_open_request = Self::request_db_open()?;
        let store_name_str = Self::store_name();
        let idb_clone = idb_open_request.clone();
        let (sender, receiver) = oneshot::channel();
        let on_success = Closure::once_into_js(move |_: web_sys::Event| {
            let db: web_sys::IdbDatabase = idb_clone.result().unwrap().dyn_into().unwrap();
            if let Ok(transaction) =
                db.transaction_with_str_and_mode(&store_name_str, IdbTransactionMode::Readwrite)
            {
                if let Ok(object_store) = transaction.object_store(&store_name_str) {
                    let _ = sender.send(object_store);
                }
            }
        });
        let on_error = Closure::once_into_js(move |event: web_sys::Event| {
            if let Err(e) = Self::upgrade_db(event) {
                error!(&e);
            }
        });
        idb_open_request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        idb_open_request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        idb_open_request.set_onupgradeneeded(Some(on_error.as_ref().unchecked_ref()));
        Ok(receiver)
    }
    fn request_db_open() -> Result<web_sys::IdbOpenDbRequest, JsValue> {
        let window = crate::browser::web_utils::window();
        if let Some(idb_factory) = window.indexed_db()? {
            let idb_open_request =
                idb_factory.open_with_u32(Self::db_name(), Self::db_version())?;
            let on_upgrade_needed = Closure::once_into_js(move |event: web_sys::Event| {
                if let Err(e) = Self::upgrade_db(event) {
                    error!(&e);
                }
            });
            idb_open_request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
            Ok(idb_open_request)
        } else {
            Err(JsValue::from_str("IndexedDB not supported"))
        }
    }
}
