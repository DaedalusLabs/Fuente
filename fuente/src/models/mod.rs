use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use crate::browser_api::IdbStoreManager;

pub mod address;
pub mod admin_configs;
pub mod commerce;
pub mod consumer_profile;
pub mod driver;
pub mod gps;
pub mod ln_address;
pub mod lnd;
pub mod nostr_kinds;
pub mod orders;
pub mod products;
// pub mod sync;
pub const TEST_PRIV_KEY: &str = "874f9683a6a2342693da781b3dd6cd3fcda7436375b5301a8a96f433b4795d2d";
pub const TEST_PUB_KEY: &str = "9fe3053c0c11b93261929ca6c167b1d955b56025f9025c40ecb1ef5ea0876d84";
pub const DRIVER_HUB_PRIV_KEY: &str =
    "874f9683a6a2342693da781b3dd6cd3fcda7436375b5301a8a96f433b4795d2d";
pub const DRIVER_HUB_PUB_KEY: &str =
    "9fe3053c0c11b93261929ca6c167b1d955b56025f9025c40ecb1ef5ea0876d84";

pub const DB_NAME_FUENTE: &str = "fuente_db";
pub const DB_VERSION_FUENTE: u32 = 1;

pub const DB_NAME_COMMERCE: &str = "commerce_db";
pub const DB_VERSION_COMMERCE: u32 = 1;

// ADMIN MODELS
pub const STORE_NAME_CONFIGS: &str = "configs";

// FUENTE MODELS

pub const STORE_NAME_COMMERCE: &str = "commerce";
pub const STORE_NAME_COMMERCE_KEYS: &str = "commerce_id";
pub const STORE_NAME_COMMERCE_PROFILES: &str = "commerce_profiles";
pub const STORE_NAME_CONSUMER_PROFILES: &str = "user_profiles";
pub const STORE_NAME_CONSUMER_ADDRESSES: &str = "consumer_address";
pub const STORE_NAME_PRODUCT_LISTS: &str = "product_lists";
pub const STORE_NAME_ORDER_HISTORY: &str = "order_history";

// a17bf270406aad26f8ab33ca7bddee38c4f1eb94e08847626902ecb491ac31ad
// eae521c94eff7292a1068c9f6614b06f5453089d84304af983f849079d3da9e2
pub const ADMIN_WHITELIST: [&str; 2] = [
    "decfef1c4a027fe815eda8ea5748aa0d3e971c4c377423a49d94fa0fc3e25575",
    "93effcf32813a65c82c6f97e3d514a77e48cbe14ef36e89453e20bd155809b33",
];

pub fn init_consumer_db() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    if let Some(idb_factory) = window.indexed_db()? {
        let idb_open_request = idb_factory.open_with_u32(DB_NAME_FUENTE, DB_VERSION_FUENTE)?;
        let on_upgrade_needed = Closure::once_into_js(move |event: web_sys::Event| {
            if event.target().is_none() {
                return Err(JsValue::from_str("Error upgrading database"));
            };
            let target = event.target().unwrap();
            let db = target
                .dyn_into::<web_sys::IdbOpenDbRequest>()?
                .result()?
                .dyn_into::<web_sys::IdbDatabase>()?;
            if let Err(e) = upgrade_fuente_db(db) {
                gloo::console::error!(&e);
            };
            Ok(())
        });
        idb_open_request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        Ok(())
    } else {
        Err(JsValue::from_str("IndexedDB not supported"))
    }
}

fn upgrade_fuente_db(db: web_sys::IdbDatabase) -> Result<(), JsValue> {
    address::ConsumerAddressIdb::create_data_store(&db)?;
    consumer_profile::ConsumerProfileIdb::create_data_store(&db)?;
    commerce::CommerceProfileIdb::create_data_store(&db)?;
    products::ProductMenuIdb::create_data_store(&db)?;
    orders::OrderStateIdb::create_data_store(&db)?;
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
