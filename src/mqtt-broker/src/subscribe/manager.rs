use crate::{
    core::share_sub::{decode_share_info, is_share_sub},
    handler::subscribe::path_regex_match,
    metadata::{cache::MetadataCacheManager, subscriber::Subscriber},
    server::MQTTProtocol,
};
use clients::{placement::mqtt::call::placement_get_share_sub, poll::ClientPool};
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError, log::info, tools::now_second,
};
use dashmap::DashMap;
use protocol::{
    mqtt::{Subscribe, SubscribeProperties},
    placement_center::generate::mqtt::GetShareSubRequest,
};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

#[derive(Clone)]
pub struct SubScribeManager<T> {
    // (topic_id, (client_id, sub))
    pub topic_subscribe: DashMap<String, DashMap<String, Subscriber>>,
    //(client_id, (topic_id, create_time))
    pub client_subscribe: DashMap<String, DashMap<String, u64>>,
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
            client_subscribe: DashMap::with_capacity(256),
        };
    }


    pub async fn parse_subscribe(
        &self,
        protocol: MQTTProtocol,
        client_id: String,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
        client_poll: Arc<ClientPool>,
    ) -> Result<(), RobustMQError> {
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

            if !self.client_subscribe.contains_key(&client_id) {
                self.client_subscribe
                    .insert(client_id.clone(), DashMap::with_capacity(256));
            }

            let tp_sub = self.topic_subscribe.get_mut(&topic_id).unwrap();
            let client_sub = self.client_subscribe.get_mut(&client_id).unwrap();
            for filter in subscribe.filters.clone() {

                if is_share_sub(filter.path.clone()) {
                    let (group_name, sub_name) = decode_share_info(filter.path.clone());
                    if path_regex_match(topic_name.clone(), sub_name.clone()) {
                        let conf = broker_mqtt_conf();
                        let req = GetShareSubRequest {
                            cluster_name: conf.cluster_name.clone(),
                            group_name,
                            sub_name: sub_name.clone(),
                        };
                        match placement_get_share_sub(
                            client_poll.clone(),
                            conf.placement.server.clone(),
                            req,
                        )
                        .await
                        {
                            Ok(reply) => {
                                info(format!(
                                    " Leader node for the shared subscription is [{}]",
                                    reply.broker_id
                                ));
                                if reply.broker_id != conf.broker_id {
                                    //todo
                                } else {
                                    let sub = Subscriber {
                                        protocol: protocol.clone(),
                                        client_id: client_id.clone(),
                                        packet_identifier: subscribe.packet_identifier,
                                        qos: filter.qos,
                                        nolocal: filter.nolocal,
                                        preserve_retain: filter.preserve_retain,
                                        subscription_identifier: sub_identifier,
                                        user_properties: Vec::new(),
                                        is_share_sub: true,
                                    };
                                    tp_sub.insert(client_id.clone(), sub);
                                    client_sub.insert(topic_id.clone(), now_second());
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                } else {
                    if path_regex_match(topic_name.clone(), filter.path.clone()) {
                        let sub = Subscriber {
                            protocol: protocol.clone(),
                            client_id: client_id.clone(),
                            packet_identifier: subscribe.packet_identifier,
                            qos: filter.qos,
                            nolocal: filter.nolocal,
                            preserve_retain: filter.preserve_retain,
                            subscription_identifier: sub_identifier,
                            user_properties: Vec::new(),
                            is_share_sub: false,
                        };
                        tp_sub.insert(client_id.clone(), sub);
                        client_sub.insert(topic_id.clone(), now_second());
                    }
                }
            }
        }
        return Ok(());
    }

    pub fn remove_topic(&self, topic_id: String) {
        self.topic_subscribe.remove(&topic_id);
    }

    pub fn remove_subscribe(&self, client_id: String, topic_ids: Vec<String>) {
        for topic_id in topic_ids {
            if let Some(sub_list) = self.topic_subscribe.get(&topic_id) {
                sub_list.remove(&client_id);
            }
        }
    }

    pub fn remove_connect_subscribe(&self, client_id: String) {
        for (topic_id, sub_list) in self.topic_subscribe.clone() {
            if sub_list.contains_key(&client_id) {
                let ts = self.topic_subscribe.get(&topic_id).unwrap();
                ts.remove(&client_id);
            }
        }
    }

    pub fn exclusive_sub(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::metadata::{cache::MetadataCacheManager, topic::Topic};
    use crate::subscribe::manager::SubScribeManager;
    use clients::poll::ClientPool;
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
        let client_poll = Arc::new(ClientPool::new(3));
        let client_id = "test-111".to_string();
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
                client_id.clone(),
                subscribe,
                None,
                client_poll,
            )
            .await;
        assert!(sub_manager.topic_subscribe.len() == 1);
        assert!(sub_manager.topic_subscribe.contains_key(&topic.topic_id));
        let vec_sub = sub_manager.topic_subscribe.get(&topic.topic_id).unwrap();
        assert!(vec_sub.contains_key(&client_id));
        let sub = vec_sub.get(&client_id).unwrap();
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
        let client_id = "test-111".to_string();
        let topic = Topic::new(&topic_name);
        metadata_cache.set_topic(&topic_name, &topic);

        let client_poll = Arc::new(ClientPool::new(3));

        let sub_manager = SubScribeManager::new(metadata_cache);
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
                client_id.clone(),
                subscribe.clone(),
                None,
                client_poll.clone(),
            )
            .await;
        assert!(sub_manager.topic_subscribe.len() == 1);
        assert!(sub_manager.topic_subscribe.contains_key(&topic.topic_id));

        sub_manager.remove_connect_subscribe(client_id.clone());
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
                client_id.clone(),
                subscribe,
                None,
                client_poll.clone(),
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
        sub_manager.remove_subscribe(client_id, topic_ids);
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
