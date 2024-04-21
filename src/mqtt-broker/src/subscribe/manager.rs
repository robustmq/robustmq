use crate::{
    handler::subscribe::path_regex_match,
    metadata::{cache::MetadataCache, subscriber::Subscriber},
    server::MQTTProtocol,
};
use protocol::mqtt::{Subscribe, SubscribeProperties};
use std::{collections::HashMap, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SubScribeManager<T> {
    pub topic_subscribe: HashMap<String, HashMap<u64, Subscriber>>,
    pub metadata_cache: Arc<RwLock<MetadataCache<T>>>,
}

impl<T> SubScribeManager<T>
where
    T: StorageAdapter,
{
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache<T>>>) -> Self {
        return SubScribeManager {
            metadata_cache,
            topic_subscribe: HashMap::new(),
        };
    }

    pub async fn parse_subscribe(
        &mut self,
        protocol: MQTTProtocol,
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
                        protocol: protocol.clone(),
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

    pub fn remove_subscribe(&mut self, connect_id: u64, topic_ids: Vec<String>) {
        for topic_id in topic_ids {
            if let Some(mut sub_list) = self.topic_subscribe.remove(&topic_id) {
                sub_list.remove(&connect_id);
                self.topic_subscribe.insert(topic_id, sub_list);
            }
        }
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

#[cfg(test)]
mod tests {
    use crate::metadata::{cache::MetadataCache, topic::Topic};
    use crate::subscribe::manager::SubScribeManager;
    use protocol::mqtt::{Filter, Subscribe};
    use std::sync::Arc;
    use storage_adapter::memory::MemoryStorageAdapter;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn parse_subscribe() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(RwLock::new(MetadataCache::new(storage_adapter.clone())));
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        let mut cache = metadata_cache.write().await;
        cache.set_topic(&topic_name, &topic);
        drop(cache);

        let mut sub_manager = SubScribeManager::new(metadata_cache);
        let connect_id = 1;
        let packet_identifier = 2;
        let mut filters = Vec::new();
        let filter = Filter {
            path: "/test/topic".to_string(),
            qos: protocol::mqtt::QoS::AtLeastOnce,
            nolocal: true,
            preserve_retain: true,
            retain_forward_rule: protocol::mqtt::RetainForwardRule::Never,
        };
        filters.push(filter);
        let subscribe = Subscribe {
            packet_identifier,
            filters,
        };
        sub_manager
            .parse_subscribe(
                crate::server::MQTTProtocol::MQTT5,
                connect_id,
                subscribe,
                None,
            )
            .await;
        assert!(sub_manager.topic_subscribe.len() == 1);
        assert!(sub_manager.topic_subscribe.contains_key(&topic.topic_id));
        let mut vec_sub = sub_manager.topic_subscribe.remove(&topic.topic_id).unwrap();
        assert!(vec_sub.contains_key(&connect_id));
        let sub = vec_sub.remove(&connect_id).unwrap();
        assert!(sub.qos == protocol::mqtt::QoS::AtLeastOnce);
    }

    #[tokio::test]
    async fn remove_subscribe() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(RwLock::new(MetadataCache::new(storage_adapter.clone())));
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        let mut cache = metadata_cache.write().await;
        cache.set_topic(&topic_name, &topic);
        drop(cache);

        let mut sub_manager = SubScribeManager::new(metadata_cache);
        let connect_id = 1;
        let packet_identifier = 2;
        let mut filters = Vec::new();
        let filter = Filter {
            path: "/test/topic".to_string(),
            qos: protocol::mqtt::QoS::AtLeastOnce,
            nolocal: true,
            preserve_retain: true,
            retain_forward_rule: protocol::mqtt::RetainForwardRule::Never,
        };
        filters.push(filter);
        let subscribe = Subscribe {
            packet_identifier,
            filters,
        };
        sub_manager
            .parse_subscribe(
                crate::server::MQTTProtocol::MQTT5,
                connect_id,
                subscribe.clone(),
                None,
            )
            .await;
        assert!(sub_manager.topic_subscribe.len() == 1);
        assert!(sub_manager.topic_subscribe.contains_key(&topic.topic_id));

        sub_manager.remove_connect_subscribe(connect_id);
        assert!(sub_manager.topic_subscribe.len() == 1);
        assert!(
            sub_manager
                .topic_subscribe
                .get(&topic.topic_id)
                .unwrap()
                .len()
                == 0
        );

        sub_manager
            .parse_subscribe(
                crate::server::MQTTProtocol::MQTT5,
                connect_id,
                subscribe,
                None,
            )
            .await;
        assert!(
            sub_manager
                .topic_subscribe
                .get(&topic.topic_id)
                .unwrap()
                .len()
                == 1
        );
        let topic_ids = vec![topic.topic_id.clone()];
        sub_manager.remove_subscribe(connect_id, topic_ids);
        assert!(
            sub_manager
                .topic_subscribe
                .get(&topic.topic_id)
                .unwrap()
                .len()
                == 0
        );
    }
}
