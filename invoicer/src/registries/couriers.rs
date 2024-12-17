use nostro2::notes::NostrNote;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CourierRegistry {
    consumers: HashMap<String, CourierRegistryEntry>,
}
impl Default for CourierRegistry {
    fn default() -> Self {
        Self {
            consumers: HashMap::new(),
        }
    }
}
impl CourierRegistry {
    pub fn is_registered(&self, consumer_id: &str) -> bool {
        self.consumers.contains_key(consumer_id)
    }
    pub fn insert_courier(&mut self, consumer_id: String, entry: CourierRegistryEntry) {
        self.consumers.insert(consumer_id, entry);
    }
    pub fn update_blacklisted(&mut self, blacklist: &Vec<String>) {
        self.consumers.iter_mut().for_each(|(id, entry)| {
            entry.blacklisted = blacklist.contains(id);
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CourierRegistryEntry {
    pub profile: NostrNote,
    pub blacklisted: bool,
}

impl Default for CourierRegistryEntry {
    fn default() -> Self {
        Self {
            profile: NostrNote::default(),
            blacklisted: false,
        }
    }
}
