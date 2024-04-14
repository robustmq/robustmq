use super::cache::MetadataCache;
use protocol::mqtt::{Filter, Subscribe, SubscribeProperties};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub connect_id: u64,
    pub packet_identifier: u16,
    pub filters: Vec<Filter>,
    pub subscription_identifier: Option<usize>,
    pub user_properties: Vec<(String, String)>,
}

impl Subscriber {
    pub fn build_subscriber(
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) -> Subscriber {
        let mut subscriber = Subscriber::default();
        subscriber.connect_id = connect_id;
        subscriber.packet_identifier = subscribe.packet_identifier;
        subscriber.filters = subscribe.filters;
        if let Some(properties) = subscribe_properties {
            subscriber.subscription_identifier = properties.subscription_identifier;
            subscriber.user_properties = properties.user_properties;
        }
        return subscriber;
    }

    pub async fn build_topic_list(&self, metadata_cache: Arc<RwLock<MetadataCache>>) {
        let cache = metadata_cache.read().await;
        let topic_info = cache.topic_info.clone();
        for (topic_id, topic) in topic_info {
            
        }
    }
}
