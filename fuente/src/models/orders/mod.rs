mod db;
mod request;
mod state;
mod update;
pub use db::*;
pub use request::*;
pub use state::*;
pub use update::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::init_consumer_db;
    use nostr_minions::browser_api::IdbStoreManager;

    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn _commerce_profile_idb() -> Result<(), web_sys::wasm_bindgen::JsValue> {
        init_consumer_db()?;
        let order_idb = OrderStateIdb::default();
        order_idb.clone().save_to_store().await?;

        let db_entries = OrderStateIdb::retrieve_all_from_store().await?;
        assert_eq!(db_entries.len(), 1);

        let order_idb_2 = OrderStateIdb::default();
        order_idb_2.clone().save_to_store().await?;

        let db_entries = OrderStateIdb::retrieve_all_from_store().await?;
        assert_eq!(db_entries.len(), 2);
        order_idb.delete_from_store().await?;
        order_idb_2.delete_from_store().await?;
        assert_eq!(OrderStateIdb::retrieve_all_from_store().await?.len(), 0);

        Ok(())
    }
}
