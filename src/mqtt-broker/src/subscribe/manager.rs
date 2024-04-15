use crate::{
    handler::subscribe::path_regex_match,
    metadata::{cache::MetadataCache, subscriber::Subscriber, topic},
};
use protocol::mqtt::Unsubscribe;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};

#[derive(Clone)]
pub struct SubScribeManager {
    pub topic_subscribe: HashMap<String, Vec<Subscriber>>,
    pub topic_subscribe_num: HashMap<String, u64>,
    pub metadata_cache: Arc<RwLock<MetadataCache>>,
}

impl SubScribeManager {
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        return SubScribeManager {
            metadata_cache,
            topic_subscribe: HashMap::new(),
            topic_subscribe_num: HashMap::new(),
        };
    }

    pub fn refresh_subscription_relationships() {}

    pub async fn add_subscribe(&mut self, connect_id: u64, subscriber: Subscriber) {
        let cache = self.metadata_cache.read().await;
        let topic_info = cache.topic_id_name.clone();

    }

    pub async fn remove_topic(&mut self, topic_id: String) {
        self.topic_subscribe.remove(&topic_id);
        self.topic_subscribe_num.remove(&topic_id);
    }

    pub fn remove_subscribe(&mut self, connect_id: u64, un_subscribe: Option<Unsubscribe>) {
        // self.subscribe_list.remove(&connect_id);
    }
}
