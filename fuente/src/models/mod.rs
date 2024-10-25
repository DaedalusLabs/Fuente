use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::IdbObjectStoreParameters;

pub mod address;
pub mod admin_configs;
pub mod commerce;
pub mod consumer_id;
pub mod consumer_profile;
pub mod driver;
pub mod gps;
pub mod ln_address;
pub mod lnd;
pub mod nostr_kinds;
pub mod products;
pub mod relays;
pub mod orders;
pub mod sync;
pub const TEST_PRIV_KEY: &str = "874f9683a6a2342693da781b3dd6cd3fcda7436375b5301a8a96f433b4795d2d";
pub const TEST_PUB_KEY: &str = "9fe3053c0c11b93261929ca6c167b1d955b56025f9025c40ecb1ef5ea0876d84";
pub const DRIVER_HUB_PRIV_KEY: &str = "874f9683a6a2342693da781b3dd6cd3fcda7436375b5301a8a96f433b4795d2d";
pub const DRIVER_HUB_PUB_KEY: &str = "9fe3053c0c11b93261929ca6c167b1d955b56025f9025c40ecb1ef5ea0876d84";

pub const DB_NAME_CONSUMER: &str = "consumer_db";
pub const DB_VERSION_CONSUMER: u32 = 4;
pub const DB_NAME_SHARED: &str = "nostr_db";
pub const DB_VERSION_SHARED: u32 = 4;
pub const DB_NAME_COMMERCE: &str = "commerce_db";
pub const DB_VERSION_COMMERCE: u32 = 4;


pub const STORE_NAME_CONFIGS: &str = "configs";
pub const STORE_NAME_COMMERCE: &str = "commerce";
pub const STORE_NAME_COMMERCE_KEYS: &str = "commerce_id";
pub const STORE_NAME_COMMERCE_PROFILES: &str = "commerce_profiles";

pub const STORE_NAME_CONSUMER_PROFILES: &str = "user_profiles";
pub const STORE_NAME_CONSUMER_ADDRESSES: &str = "consumer_address";

pub const STORE_NAME_USER_KEYS: &str = "user_id";
pub const STORE_NAME_USER_RELAYS: &str = "user_relays";
pub const STORE_NAME_PRODUCT_LISTS: &str = "product_lists";
pub const STORE_NAME_ORDER_HISTORY: &str = "order_history";


pub fn init_consumer_db() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    if let Some(idb_factory) = window.indexed_db()? {
        let idb_open_request = idb_factory.open_with_u32(DB_NAME_CONSUMER, DB_VERSION_CONSUMER)?;
        let on_upgrade_needed = Closure::once_into_js(move |event: web_sys::Event| {
            if let Err(e) = upgrade_consumer_db(event) {
                gloo::console::error!(&e);
            }
        });
        idb_open_request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        Ok(())
    } else {
        Err(JsValue::from_str("IndexedDB not supported"))
    }
}

fn upgrade_consumer_db(event: web_sys::Event) -> Result<(), JsValue> {
    if event.target().is_none() {
        return Err(JsValue::from_str("Error upgrading database"));
    };
    let target = event.target().unwrap();
    let db = target
        .dyn_into::<web_sys::IdbOpenDbRequest>()?
        .result()?
        .dyn_into::<web_sys::IdbDatabase>()?;
    db.create_object_store(STORE_NAME_USER_KEYS)?;
    let user_relay_params = IdbObjectStoreParameters::new();
    user_relay_params.set_key_path(&JsValue::from_str("url"));
    db.create_object_store_with_optional_parameters(STORE_NAME_USER_RELAYS, &user_relay_params)?;
    Ok(())
}

pub fn init_shared_db() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    if let Some(idb_factory) = window.indexed_db()? {
        let idb_open_request = idb_factory.open_with_u32(DB_NAME_SHARED, DB_VERSION_SHARED)?;
        let on_upgrade_needed = Closure::once_into_js(move |event: web_sys::Event| {
            if let Err(e) = upgrade_shared_db(event) {
                gloo::console::error!(&e);
            }
        });
        idb_open_request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        Ok(())
    } else {
        Err(JsValue::from_str("IndexedDB not supported"))
    }
}

fn upgrade_shared_db(event: web_sys::Event) -> Result<(), JsValue> {
    if event.target().is_none() {
        return Err(JsValue::from_str("Error upgrading database"));
    };
    let target = event.target().unwrap();
    let db = target
        .dyn_into::<web_sys::IdbOpenDbRequest>()?
        .result()?
        .dyn_into::<web_sys::IdbDatabase>()?;
    let _ = db.create_object_store(STORE_NAME_COMMERCE_PROFILES);
    let _ = db.create_object_store(STORE_NAME_PRODUCT_LISTS);
    let _ = db.create_object_store(STORE_NAME_CONSUMER_PROFILES);
    let _ = db.create_object_store(STORE_NAME_CONSUMER_ADDRESSES);
    let _ = db.create_object_store(STORE_NAME_ORDER_HISTORY);
    let _ = db.create_object_store(STORE_NAME_CONFIGS);
    Ok(())
}

pub fn init_commerce_db() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    if let Some(idb_factory) = window.indexed_db()? {
        let idb_open_request = idb_factory.open_with_u32(DB_NAME_COMMERCE, DB_VERSION_COMMERCE)?;
        let on_upgrade_needed = Closure::once_into_js(move |event: web_sys::Event| {
            if let Err(e) = upgrade_commerce_db(event) {
                gloo::console::error!(&e);
            }
        });
        idb_open_request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        Ok(())
    } else {
        Err(JsValue::from_str("IndexedDB not supported"))
    }
}

fn upgrade_commerce_db(event: web_sys::Event) -> Result<(), JsValue> {
    if event.target().is_none() {
        return Err(JsValue::from_str("Error upgrading database"));
    };
    let target = event.target().unwrap();
    let db = target
        .dyn_into::<web_sys::IdbOpenDbRequest>()?
        .result()?
        .dyn_into::<web_sys::IdbDatabase>()?;
    db.create_object_store(STORE_NAME_COMMERCE)?;
    db.create_object_store(STORE_NAME_COMMERCE_KEYS)?;
    db.create_object_store(STORE_NAME_COMMERCE_PROFILES)?;
    Ok(())
}
