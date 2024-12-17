use nostro2::notes::NostrNote;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CommerceRegistry {
    commerce: HashMap<String, CommerceRegistryEntry>,
}
impl Default for CommerceRegistry {
    fn default() -> Self {
        Self {
            commerce: HashMap::new(),
        }
    }
}
impl CommerceRegistry {
    pub fn get_commerce(&self, commerce_id: &str) -> Option<&CommerceRegistryEntry> {
        self.commerce.get(commerce_id)
    }
    pub fn update_record(&mut self, commerce_id: String, new_entry: CommerceRegistryEntry) {
        if let Some(old_entry) = self.commerce.get_mut(&commerce_id) {
            if let Some(profile) = new_entry.profile {
                old_entry.profile = Some(profile);
            }
            if let Some(menu) = new_entry.menu {
                old_entry.menu = Some(menu);
            }
        } else {
            self.commerce.insert(commerce_id, new_entry);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommerceRegistryEntry {
    pub profile: Option<NostrNote>,
    pub menu: Option<NostrNote>,
}
impl Default for CommerceRegistryEntry {
    fn default() -> Self {
        Self {
            profile: None,
            menu: None,
        }
    }
}
