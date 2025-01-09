use nostro2::notes::NostrNote;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CourierRegistry {
    couriers: HashMap<String, CourierRegistryEntry>,
}
impl Default for CourierRegistry {
    fn default() -> Self {
        Self {
            couriers: HashMap::new(),
        }
    }
}
impl CourierRegistry {
    pub fn insert_courier(&mut self, courier_id: String, entry: CourierRegistryEntry) {
        self.couriers.insert(courier_id, entry);
    }
    pub fn find_courier(&self, courier_id: &str) -> Option<NostrNote> {
        self.couriers
            .get(courier_id)
            .map(|entry| entry.profile.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CourierRegistryEntry {
    pub profile: NostrNote,
}

impl Default for CourierRegistryEntry {
    fn default() -> Self {
        Self {
            profile: NostrNote::default(),
        }
    }
}
