use crate::{
    handler::subscribe::path_regex_match,
    metadata::{cache::MetadataCacheManager, subscriber::Subscriber},
    server::MQTTProtocol,
};
use dashmap::DashMap;
use protocol::mqtt::{Subscribe, SubscribeProperties};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

#[derive(Clone)]
pub struct SubScribeManager<T> {
    pub topic_subscribe: DashMap<String, DashMap<u64, Subscriber>>,
    pub metadata_cache: Arc<MetadataCacheManager<T>>,
}

impl<T> SubScribeManager<T>
where
    T: StorageAdapter,
{
    pub fn new(metadata_cache: Arc<MetadataCacheManager<T>>) -> Self {
        return SubScribeManager {
            metadata_cache,
            topic_subscribe: DashMap::with_capacity(256),
        };
    }

    pub async fn parse_subscribe(
        &self,
        protocol: MQTTProtocol,
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) {
        let sub_identifier = if let Some(properties) = subscribe_properties {
            properties.subscription_identifier
        } else {
            None
        };

        for (topic_id, topic_name) in self.metadata_cache.topic_id_name.clone() {
            if !self.topic_subscribe.contains_key(&topic_id) {
                self.topic_subscribe
                    .insert(topic_id.clone(), DashMap::with_capacity(256));
            }

            let tp_sub = self.topic_subscribe.get_mut(&topic_id).unwrap();
            for filter in subscribe.filters.clone() {
                if path_regex_match(topic_name.clone(), filter.path.clone()) {
                    let sub = Subscriber {
                        protocol: protocol.clone(),
                        connect_id,
                        packet_identifier: subscribe.packet_identifier,
                        qos: filter.qos,
                        nolocal: filter.nolocal,
                        preserve_retain: filter.preserve_retain,
                        subscription_identifier: sub_identifier,
                        user_properties: Vec::new(),
                    };
                    tp_sub.insert(connect_id, sub);
                }
            }
        }
    }

    pub fn remove_topic(&self, topic_id: String) {
        self.topic_subscribe.remove(&topic_id);
    }

    pub fn remove_subscribe(&self, connect_id: u64, topic_ids: Vec<String>) {
        for topic_id in topic_ids {
            if let Some(sub_list) = self.topic_subscribe.get(&topic_id) {
                sub_list.remove(&connect_id);
            }
        }
    }

    pub fn remove_connect_subscribe(&self, connect_id: u64) {
        for (topic_id, sub_list) in self.topic_subscribe.clone() {
            if sub_list.contains_key(&connect_id) {
                let ts = self.topic_subscribe.get(&topic_id).unwrap();
                ts.remove(&connect_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::metadata::{cache::MetadataCacheManager, topic::Topic};
    use crate::subscribe::manager::SubScribeManager;
    use protocol::mqtt::{Filter, Subscribe};
    use std::sync::Arc;
    use storage_adapter::memory::MemoryStorageAdapter;

    #[tokio::test]
    async fn parse_subscribe() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(MetadataCacheManager::new(
            storage_adapter.clone(),
            "test-cluster".to_string(),
        ));
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        metadata_cache.set_topic(&topic_name, &topic);
        let sub_manager = SubScribeManager::new(metadata_cache);
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
        let vec_sub = sub_manager.topic_subscribe.get(&topic.topic_id).unwrap();
        assert!(vec_sub.contains_key(&connect_id));
        let sub = vec_sub.get(&connect_id).unwrap();
        assert!(sub.qos == protocol::mqtt::QoS::AtLeastOnce);
    }

    #[tokio::test]
    async fn remove_subscribe() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(MetadataCacheManager::new(
            storage_adapter.clone(),
            "test-cluster".to_string(),
        ));
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        metadata_cache.set_topic(&topic_name, &topic);

        let sub_manager = SubScribeManager::new(metadata_cache);
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
