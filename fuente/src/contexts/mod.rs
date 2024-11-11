use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use crate::browser_api::IdbStoreManager;

mod admin_configs;
mod key_manager;
mod relay_pool;
pub use admin_configs::*;
pub use key_manager::*;
pub use relay_pool::*;

pub const DB_NAME: &str = "nostr";
pub const DB_VERSION: u32 = 3;
pub const STORE_NAME_NOSTR_IDS: &str = "nostr_ids";
pub const STORE_NAME_NOSTR_RELAYS: &str = "nostr_relays";

pub fn init_nostr_db() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    if let Some(idb_factory) = window.indexed_db()? {
        let idb_open_request = idb_factory.open_with_u32(DB_NAME, DB_VERSION)?;
        let on_upgrade_needed = Closure::once_into_js(move |event: web_sys::Event| {
            if let Err(e) = upgrade_nostr_db(event) {
                gloo::console::error!(&e);
            }
        });
        idb_open_request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        Ok(())
    } else {
        Err(JsValue::from_str("IndexedDB not supported"))
    }
}
fn upgrade_nostr_db(event: web_sys::Event) -> Result<(), JsValue> {
    if event.target().is_none() {
        return Err(JsValue::from_str("Error upgrading database"));
    };
    let target = event.target().unwrap();
    let db = target
        .dyn_into::<web_sys::IdbOpenDbRequest>()?
        .result()?
        .dyn_into::<web_sys::IdbDatabase>()?;
    let db_store_names = db.object_store_names();
    if !db_store_names.contains(STORE_NAME_NOSTR_IDS) {
        key_manager::UserIdentity::create_data_store(&db)?;
    }
    if !db_store_names.contains(STORE_NAME_NOSTR_RELAYS) {
        relay_pool::UserRelay::create_data_store(&db)?;
    }
    Ok(())
}
