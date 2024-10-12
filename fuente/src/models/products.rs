use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    f64,
    hash::{DefaultHasher, Hash, Hasher},
};
use wasm_bindgen::JsValue;

use crate::browser::indexed_db::IdbStoreManager;

use super::{
    nostr_kinds::NOSTR_KIND_COMMERCE_PRODUCTS, upgrade_shared_db, DB_NAME_SHARED,
    DB_VERSION_SHARED, STORE_NAME_PRODUCT_LISTS,
};

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct ProductSide {
    id: String,
    order: usize,
    name: String,
    price: String,
}
impl ToString for ProductSide {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for ProductSide {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s)?)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct ProductItem {
    id: String,
    name: String,
    price: String,
    order: usize,
    category: String,
    description: String,
    sides: Vec<ProductSide>,
}
impl ProductItem {
    pub fn new(
        order: usize,
        name: String,
        price: String,
        description: String,
        category: String,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        format!("{}{}", name, category).hash(&mut hasher);
        ProductItem {
            id: hasher.finish().to_string(),
            name,
            price,
            order,
            category,
            description,
            sides: vec![],
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn price(&self) -> String {
        self.price.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn order(&self) -> usize {
        self.order
    }
    pub fn description(&self) -> String {
        self.description.clone()
    }
    pub fn add_side(&mut self, side: ProductSide) {
        self.sides.push(side);
    }
}
impl TryFrom<String> for ProductItem {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s)?)
    }
}
impl ToString for ProductItem {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct ProductCategory {
    id: String,
    name: String,
    order: usize,
    products: Vec<ProductItem>,
}
impl ToString for ProductCategory {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl ProductCategory {
    pub fn new(order: usize, name: String) -> Self {
        let mut hasher = DefaultHasher::new();
        format!("{}", name).hash(&mut hasher);
        ProductCategory {
            id: hasher.finish().to_string(),
            name,
            order,
            products: vec![],
        }
    }
    pub fn add_product(&mut self, product: ProductItem) {
        if let Some(i) = self.products.iter().position(|p| p.id == product.id) {
            let mut new_product = self.products[i].clone();
            new_product.name = product.name;
            new_product.price = product.price;
            self.products[i] = new_product;
        } else {
            self.products.push(product);
        }
    }
    pub fn remove_product(&mut self, product_id: String) {
        self.products.retain(|p| p.id != product_id);
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn order(&self) -> usize {
        self.order
    }
    pub fn products(&self) -> Vec<ProductItem> {
        let mut products = self.products.clone();
        products.sort_by(|a, b| a.order.cmp(&b.order));
        products
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductMenu {
    categories: Vec<ProductCategory>,
}
impl ProductMenu {
    pub fn categories(&self) -> Vec<ProductCategory> {
        let mut menu = self.categories.clone();
        menu.sort_by(|a, b| a.order.cmp(&b.order));
        menu
    }
    pub fn add_category(&mut self, category: ProductCategory) {
        self.categories.push(category);
    }
    pub fn add_product(&mut self, category_id: String, product: ProductItem) {
        if let Some(i) = self.categories.iter().position(|c| c.id == category_id) {
            self.categories[i].add_product(product);
        }
    }
    pub fn update_category_name(&mut self, category: ProductCategory) {
        if let Some(i) = self.categories.iter().position(|c| c.id == category.id) {
            let mut updated = self.categories[i].clone();
            updated.name = category.name;
            self.categories[i] = updated;
        } else {
            self.categories.push(category);
        }
    }
    pub fn new() -> Self {
        Self { categories: vec![] }
    }
}
impl TryFrom<String> for ProductMenu {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s)?)
    }
}
impl ToString for ProductMenu {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<SignedNote> for ProductMenu {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_COMMERCE_PRODUCTS {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let product_menu: ProductMenu = note.get_content().try_into()?;
        Ok(product_menu)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductMenuIdb {
    id: String,
    menu: ProductMenu,
    note: SignedNote,
}
impl ProductMenuIdb {
    pub fn new(menu: ProductMenu, user_keys: &UserKeys) -> Self {
        let content = menu.to_string();
        let unsigned_note = Note::new(
            &user_keys.get_public_key(),
            NOSTR_KIND_COMMERCE_PRODUCTS,
            &content,
        );
        let note = user_keys.sign_nostr_event(unsigned_note);
        Self {
            id: note.get_pubkey().to_string(),
            menu,
            note,
        }
    }
    pub async fn save(self) -> Result<(), JsValue> {
        self.save_to_store()?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub async fn delete(&self) -> Result<(), JsValue> {
        self.delete_from_store()?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub async fn find(id: &str) -> Result<Self, JsValue> {
        Self::retrieve::<Self>(id)?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub async fn find_all() -> Result<Vec<Self>, JsValue> {
        Self::retrieve_all_from_store::<Self>()?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub fn menu(&self) -> ProductMenu {
        self.menu.clone()
    }
    pub fn note(&self) -> SignedNote {
        self.note.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
}
impl TryFrom<SignedNote> for ProductMenuIdb {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_COMMERCE_PRODUCTS {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let content = note.get_content();
        let menu: ProductMenu = serde_json::from_str(&content)?;
        Ok(Self {
            id: note.get_pubkey().to_string(),
            menu,
            note,
        })
    }
}
impl TryFrom<JsValue> for ProductMenuIdb {
    type Error = JsValue;
    fn try_from(js_value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(js_value)?)
    }
}
impl TryInto<JsValue> for ProductMenuIdb {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}
impl IdbStoreManager for ProductMenuIdb {
    fn db_name() -> &'static str {
        DB_NAME_SHARED
    }
    fn db_version() -> u32 {
        DB_VERSION_SHARED
    }
    fn store_name() -> &'static str {
        STORE_NAME_PRODUCT_LISTS
    }
    fn document_key(&self) -> JsValue {
        JsValue::from_str(&self.id)
    }
    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue> {
        upgrade_shared_db(event)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ProductOrder {
    products: Vec<ProductItem>,
}
impl ToString for ProductOrder {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for ProductOrder {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s)?)
    }
}
impl ProductOrder {
    pub fn new(products: Vec<ProductItem>) -> Self {
        Self { products }
    }
    pub fn products(&self) -> Vec<ProductItem> {
        self.products.clone()
    }
    pub fn counted_products(&self) -> Vec<(ProductItem, u32)> {
        let mut counted_products = HashMap::new();
        self.products.iter().for_each(|product| {
            let count = counted_products.entry(product.clone()).or_insert(0);
            *count += 1;
        });
        let mut products: Vec<(ProductItem, u32)> = counted_products.into_iter().collect();
        products.sort_by(|a, b| a.0.order.cmp(&b.0.order));
        products
    }
    pub fn total(&self) -> f64 {
        self.products
            .iter()
            .map(|p| p.price.parse::<f64>().unwrap())
            .sum()
    }
    pub fn is_empty(&self) -> bool {
        self.products.is_empty()
    }
    pub fn add(&mut self, product: ProductItem) {
        self.products.push(product);
    }
    pub fn remove_one(&mut self, product_id: String) {
        if let Some(i) = self.products.iter().position(|p| p.id == product_id) {
            self.products.remove(i);
        }
    }
}
