use nostro2::notes::NostrNote;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConsumerRegistry {
    consumers: HashMap<String, ConsumerRegistryEntry>,
}
impl Default for ConsumerRegistry {
    fn default() -> Self {
        Self {
            consumers: HashMap::new(),
        }
    }
}
impl ConsumerRegistry {
    pub fn is_registered(&self, consumer_id: &str) -> bool {
        self.consumers.contains_key(consumer_id)
    }
    pub fn insert_consumer(&mut self, consumer_id: String, entry: ConsumerRegistryEntry) {
        self.consumers.insert(consumer_id, entry);
    }
    pub fn update_blacklisted(&mut self, blacklist: &Vec<String>) {
        self.consumers.iter_mut().for_each(|(id, entry)| {
            entry.blacklisted = blacklist.contains(id);
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsumerRegistryEntry {
    pub profile: NostrNote,
    pub blacklisted: bool,
}

impl Default for ConsumerRegistryEntry {
    fn default() -> Self {
        Self {
            profile: NostrNote::default(),
            blacklisted: false,
        }
    }
}
