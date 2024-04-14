use crate::metadata::{cache::MetadataCache, subscriber::Subscriber};
use protocol::mqtt::Unsubscribe;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SubScribeManager {
    pub subscribe_list: HashMap<String, Vec<Subscriber>>,
    pub metadata_cache: Arc<RwLock<MetadataCache>>,
}

impl SubScribeManager {
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        return SubScribeManager {
            subscribe_list: HashMap::new(),
            metadata_cache,
        };
    }

    pub fn add_subscribe(&mut self, connect_id: u64, subscriber: Subscriber) {
        // self.subscribe_list.insert(connect_id, subscriber);
    }

    pub fn remove_subscribe(&mut self, connect_id: u64, un_subscribe: Option<Unsubscribe>) {
        // self.subscribe_list.remove(&connect_id);
    }
}
