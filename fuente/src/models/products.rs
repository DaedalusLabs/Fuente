use nostro2::notes::NostrNote;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    f64,
    hash::{DefaultHasher, Hash, Hasher},
};
use web_sys::wasm_bindgen::JsValue;

use nostr_minions::browser_api::IdbStoreManager;
#[cfg(target_arch = "wasm32")]
use nostr_minions::key_manager::UserIdentity;

use super::{
    nostr_kinds::NOSTR_KIND_COMMERCE_PRODUCTS, DB_NAME_FUENTE, DB_VERSION_FUENTE,
    STORE_NAME_PRODUCT_LISTS,
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
    #[serde(default)]
    sku: Option<String>,
    price: String,
    #[serde(default)]
    discount: Option<String>,
    order: usize,
    category: String,
    #[serde(default)]
    details: String,
    description: String,
    #[serde(default)]
    image_url: Option<String>,
    #[serde(default)]
    thumbnail_url: Option<String>,
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
        let id = hasher.finish().to_string();
        let sku = Some(format!("SKU-{}", id[..6].to_uppercase()));
        Self {
            id,
            name,
            sku,
            price,
            order,
            discount: None,
            category,
            description,
            details: String::new(),
            image_url: None,
            thumbnail_url: None,
            sides: vec![],
        }
    }
    // Add new getter methods
    pub fn category_id(&self) -> String {
        // Add this getter method
        self.category.clone()
    }
    pub fn details(&self) -> String {
        self.details.clone()
    }
    pub fn sku(&self) -> String {
        self.sku.clone().unwrap_or_default()
    }
    pub fn image_url(&self) -> String {
        self.image_url
            .clone()
            .unwrap_or_else(|| "/public/assets/img/logo.png".to_string())
    }
    pub fn thumbnail_url(&self) -> String {
        // added debug loggins
        let url = self
            .thumbnail_url
            .clone()
            .unwrap_or_else(|| "/public/assets/img/logo.png".to_string());
        url
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn price(&self) -> String {
        let base = self.price.parse::<f64>().unwrap_or(0.0);

        if let Some(discount) = &self.discount {
            let disc = discount.parse::<f64>().unwrap_or(0.0);
            return format!("{:.2}", base - disc);
        }

        // Return original base price if no discount
        format!("{:.2}", base)
    }
    pub fn discount(&self) -> Option<String> {
        self.discount.clone()
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
    pub fn set_image_url(&mut self, url: String) {
        self.image_url = Some(url);
    }
    pub fn set_thumbnail_url(&mut self, url: String) {
        self.thumbnail_url = Some(url);
    }
    pub fn set_sku(&mut self, sku: String) {
        self.sku = Some(sku);
    }
    pub fn set_details(&mut self, details: String) {
        self.details = details;
    }
    pub fn set_discount(&mut self, discount: Option<String>) {
        self.discount = discount;
    }
    pub fn set_price(&mut self, price: String) {
        self.price = price;
    }
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ProductOrder {
    products: Vec<ProductItem>,
}
impl Default for ProductOrder {
    fn default() -> Self {
        Self { products: vec![] }
    }
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
            .map(|p| p.price().parse::<f64>().unwrap_or(0.0))
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
    pub fn remove_all(&mut self, product_id: String) {
        self.products.retain(|p| p.id != product_id);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductMenu {
    categories: Vec<ProductCategory>,
}
impl Default for ProductMenu {
    fn default() -> Self {
        Self { categories: vec![] }
    }
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
    pub fn remove_product(&mut self, category_id: &str, product_id: &str) {
        if let Some(category) = self.categories.iter_mut().find(|c| c.id == category_id) {
            category.remove_product(product_id.to_string());
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
impl TryFrom<NostrNote> for ProductMenu {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_COMMERCE_PRODUCTS {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let product_menu: ProductMenu = note.content.try_into()?;
        Ok(product_menu)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductMenuIdb {
    pubkey: String,
    menu: ProductMenu,
    note: NostrNote,
}
impl ProductMenuIdb {
    #[cfg(target_arch = "wasm32")]
    pub async fn new(menu: ProductMenu, user_keys: &UserIdentity) -> Self {
        let pubkey = user_keys.get_pubkey().await.expect("no pubkey");
        let content = menu.to_string();
        let new_note = NostrNote {
            pubkey: pubkey.clone(),
            kind: NOSTR_KIND_COMMERCE_PRODUCTS,
            content,
            ..Default::default()
        };
        let new_note = user_keys
            .sign_nostr_note(new_note)
            .await
            .expect("Could not sign note");
        Self {
            pubkey: new_note.pubkey.clone(),
            menu,
            note: new_note,
        }
    }
    pub fn menu(&self) -> ProductMenu {
        self.menu.clone()
    }
    pub fn note(&self) -> NostrNote {
        self.note.clone()
    }
    pub fn id(&self) -> String {
        self.pubkey.clone()
    }
}
impl TryFrom<NostrNote> for ProductMenuIdb {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_COMMERCE_PRODUCTS {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let menu: ProductMenu = serde_json::from_str(&note.content)?;
        Ok(Self {
            pubkey: note.pubkey.clone(),
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
impl Into<JsValue> for ProductMenuIdb {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl IdbStoreManager for ProductMenuIdb {
    fn config() -> nostr_minions::browser_api::IdbStoreConfig {
        nostr_minions::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_PRODUCT_LISTS,
            document_key: "pubkey",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.pubkey)
    }
}
