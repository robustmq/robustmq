use crate::{
    handler::subscribe::path_regex_match,
    metadata::{cache::MetadataCache, subscriber::Subscriber},
};
use protocol::mqtt::{Subscribe, SubscribeProperties, Unsubscribe, UnsubscribeProperties};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SubScribeManager {
    pub topic_subscribe: HashMap<String, HashMap<u64, Subscriber>>,
    pub metadata_cache: Arc<RwLock<MetadataCache>>,
}

impl SubScribeManager {
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        return SubScribeManager {
            metadata_cache,
            topic_subscribe: HashMap::new(),
        };
    }

    pub async fn parse_subscribe(
        &mut self,
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) {
        let cache = self.metadata_cache.read().await;
        let topic_info = cache.topic_id_name.clone();
        drop(cache);

        let sub_id = if let Some(properties) = subscribe_properties {
            properties.subscription_identifier
        } else {
            None
        };

        for (topic_id, topic_name) in topic_info.clone() {
            let mut tp_sub = if let Some(sub_list) = self.topic_subscribe.remove(&topic_id) {
                sub_list
            } else {
                HashMap::new()
            };
            for filter in subscribe.filters.clone() {
                if path_regex_match(topic_name.clone(), filter.path.clone()) {
                    let sub = Subscriber {
                        connect_id,
                        packet_identifier: subscribe.packet_identifier,
                        qos: filter.qos,
                        nolocal: filter.nolocal,
                        preserve_retain: filter.preserve_retain,
                        subscription_identifier: sub_id,
                        user_properties: Vec::new(),
                    };
                    tp_sub.insert(connect_id, sub);
                }
            }
            self.topic_subscribe.insert(topic_id, tp_sub);
        }
    }

    pub fn remove_topic(&mut self, topic_id: String) {
        self.topic_subscribe.remove(&topic_id);
    }

    pub fn remove_subscribe(
        &mut self,
        connect_id: u64,
        un_subscribe: Unsubscribe,
        un_subscribe_properties: Option<UnsubscribeProperties>,
    ) {
    }

    pub fn remove_connect_subscribe(&mut self, connect_id: u64) {
        for (topic_id, mut sub_list) in self.topic_subscribe.clone() {
            if sub_list.contains_key(&connect_id) {
                sub_list.remove(&connect_id);
                self.topic_subscribe.insert(topic_id, sub_list);
            }
        }
    }
}
