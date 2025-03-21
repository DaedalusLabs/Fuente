use nostr_minions::browser_api::IdbStoreManager;
use web_sys::wasm_bindgen::{closure::Closure, JsCast, JsValue};

mod address;
mod admin_configs;
mod commerce;
mod consumer_profile;
mod driver;
mod favorites;
mod gps;
mod nostr_kinds;
mod orders;
mod products;
mod ratings;
pub use address::*;
pub use admin_configs::*;
pub use commerce::*;
pub use consumer_profile::*;
pub use driver::*;
pub use favorites::*;
pub use gps::*;
pub use nostr_kinds::*;
pub use orders::*;
pub use products::*;
pub use ratings::*;

pub const TEST_PUB_KEY: &str = "9fe3053c0c11b93261929ca6c167b1d955b56025f9025c40ecb1ef5ea0876d84";
pub const DRIVER_HUB_PRIV_KEY: &str =
    "874f9683a6a2342693da781b3dd6cd3fcda7436375b5301a8a96f433b4795d2d";
pub const DRIVER_HUB_PUB_KEY: &str =
    "9fe3053c0c11b93261929ca6c167b1d955b56025f9025c40ecb1ef5ea0876d84";

pub const DB_NAME_FUENTE: &str = "fuente_db";
pub const DB_VERSION_FUENTE: u32 = 7;

pub const DB_NAME_COMMERCE: &str = "commerce_db";
pub const DB_VERSION_COMMERCE: u32 = 6;

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
pub const STORE_NAME_CONSUMER_FAVORITES: &str = "consumer_favorites";
pub const STORE_NAME_COURIER_PROFILES: &str = "courier_profiles";

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
    if !db
        .object_store_names()
        .contains(STORE_NAME_CONSUMER_PROFILES)
    {
        consumer_profile::ConsumerProfileIdb::create_data_store(&db)?;
        gloo::console::log!("Consumer profile store created");
    }
    if !db
        .object_store_names()
        .contains(STORE_NAME_CONSUMER_ADDRESSES)
    {
        address::ConsumerAddressIdb::create_data_store(&db)?;
        gloo::console::log!("Consumer address store created");
    }
    if !db
        .object_store_names()
        .contains(STORE_NAME_COMMERCE_PROFILES)
    {
        commerce::CommerceProfileIdb::create_data_store(&db)?;
        gloo::console::log!("Commerce profile store created");
    }
    if !db.object_store_names().contains(STORE_NAME_PRODUCT_LISTS) {
        products::ProductMenuIdb::create_data_store(&db)?;
        gloo::console::log!("Product list store created");
    }
    if !db.object_store_names().contains(STORE_NAME_ORDER_HISTORY) {
        orders::OrderStateIdb::create_data_store(&db)?;
        gloo::console::log!("Order history store created");
    }
    if !db
        .object_store_names()
        .contains(STORE_NAME_CONSUMER_FAVORITES)
    {
        FavoriteStore::create_data_store(&db)?;
        gloo::console::log!("Favorites store created");
    }
    if !db.object_store_names().contains("stats") {
        PlatformStatIdb::create_data_store(&db)?;
        gloo::console::log!("Favorites store created");
    }
    if !db
        .object_store_names()
        .contains(STORE_NAME_COURIER_PROFILES)
    {
        driver::DriverProfileIdb::create_data_store(&db)?;
        gloo::console::log!("Driver profile store created");
    }
    gloo::console::log!("Fuente database upgraded");
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
    let store_names = db.object_store_names();
    if !store_names.contains(STORE_NAME_COMMERCE) {
        db.create_object_store(STORE_NAME_COMMERCE)?;
    }
    if !store_names.contains(STORE_NAME_COMMERCE_KEYS) {
        db.create_object_store(STORE_NAME_COMMERCE_KEYS)?;
    }
    if !store_names.contains(STORE_NAME_COMMERCE_PROFILES) {
        db.create_object_store(STORE_NAME_COMMERCE_PROFILES)?;
    }
    Ok(())
}
