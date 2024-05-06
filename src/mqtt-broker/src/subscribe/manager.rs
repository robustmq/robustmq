use crate::core::metadata_cache::MetadataCacheManager;
use crate::core::subscribe::path_regex_match;
use crate::metadata::subscriber::Subscriber;
use crate::server::MQTTProtocol;
use clients::placement::mqtt::call::placement_get_share_sub;
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::errors::RobustMQError;
use common_base::log::info;
use common_base::tools::now_second;
use dashmap::DashMap;
use protocol::mqtt::{Subscribe, SubscribeProperties};
use protocol::placement_center::generate::mqtt::GetShareSubRequest;
use std::{sync::Arc, time::Duration};
use tokio::{sync::broadcast::Sender, time::sleep};
use crate::subscribe::share_rewrite::{decode_share_info, is_share_sub};

pub struct SubscribeManager {
    metadata_cache: Arc<MetadataCacheManager>,
    client_poll: Arc<ClientPool>,

    // (topic_id,(client_id,Subscriber))
    pub exclusive_subscribe: DashMap<String, DashMap<String, Subscriber>>,

    // (topic_id,(client_id,Sender<bool>))
    pub share_sub_action_thread: DashMap<String, Sender<bool>>,

    // (topic_id,(client_id,Subscriber))
    pub share_subscribe: DashMap<String, DashMap<String, Subscriber>>,

    // (client_id, (topic_id, now_second()))
    pub client_subscribe: DashMap<String, DashMap<String, u64>>,
}

impl SubscribeManager {
    pub fn new(metadata_cache: Arc<MetadataCacheManager>, client_poll: Arc<ClientPool>) -> Self {
        return SubscribeManager {
            metadata_cache,
            client_poll,
            exclusive_subscribe: DashMap::with_capacity(8),
            share_sub_action_thread: DashMap::with_capacity(8),
            share_subscribe: DashMap::with_capacity(8),
            client_subscribe: DashMap::with_capacity(8),
        };
    }

    pub async fn start(&self) {
        info("Subscription push thread is started successfully.".to_string());
        loop {
            self.parse_subscribe_by_new_topic().await;
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn parse_subscribe_by_new_topic(&self) {
        for (topic_name, topic) in self.metadata_cache.topic_info.clone() {
            let topic_id = topic.topic_id;
            if self.exclusive_subscribe.contains_key(&topic_id) {
                continue;
            }

            if !self.exclusive_subscribe.contains_key(&topic_id) {
                self.exclusive_subscribe
                    .insert(topic_id.clone(), DashMap::with_capacity(256));
            }

            for (client_id, data) in self.metadata_cache.subscribe_filter.clone() {
                if !self.client_subscribe.contains_key(&client_id) {
                    self.client_subscribe
                        .insert(client_id.clone(), DashMap::with_capacity(256));
                }
                let subscribe = data.subscribe;
                let subscribe_properties = data.subscribe_properties;
                self.parse_subscribe(
                    topic_name.clone(),
                    topic_id.clone(),
                    client_id.clone(),
                    data.protocol.clone(),
                    subscribe,
                    subscribe_properties,
                )
            }
        }
    }

    pub async fn is_share_sub_leader(
        &self,
        group_name: String,
        sub_name: String,
    ) -> Result<bool, RobustMQError> {
        let conf = broker_mqtt_conf();
        let req = GetShareSubRequest {
            cluster_name: conf.cluster_name.clone(),
            group_name,
            sub_name: sub_name.clone(),
        };
        match placement_get_share_sub(self.client_poll.clone(), conf.placement.server.clone(), req)
            .await
        {
            Ok(reply) => {
                info(format!(
                    " Leader node for the shared subscription is [{}]",
                    reply.broker_id
                ));
                return Ok(reply.broker_id == conf.broker_id);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn add_subscribe(
        &self,
        client_id: String,
        protocol: MQTTProtocol,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) {
        for (topic_name, topic) in self.metadata_cache.topic_info.clone() {
            let topic_id = topic.topic_id;
            if !self.exclusive_subscribe.contains_key(&topic_id) {
                continue;
            }
            self.parse_subscribe(
                topic_name,
                topic_id,
                client_id.clone(),
                protocol.clone(),
                subscribe.clone(),
                subscribe_properties.clone(),
            );
        }
    }

    pub fn remove_subscribe(&self, client_id: String, filter_path: Vec<String>) {
        for (topic_name, topic) in self.metadata_cache.topic_info.clone() {
            let topic_id = topic.topic_id;
            for path in filter_path.clone() {
                if !path_regex_match(topic_name.clone(), path.clone()) {
                    continue;
                }

                if is_share_sub(path) {
                    if let Some(sub_list) = self.exclusive_subscribe.get_mut(&topic_id) {
                        sub_list.remove(&client_id);
                    }
                } else {
                    if let Some(sub_list) = self.share_subscribe.get_mut(&topic_id) {
                        sub_list.remove(&client_id);
                    }
                }

                if let Some(client_list) = self.client_subscribe.get_mut(&client_id) {
                    client_list.remove(&topic_id);
                }
            }
        }
    }

    fn parse_subscribe(
        &self,
        topic_name: String,
        topic_id: String,
        client_id: String,
        protocol: MQTTProtocol,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) {
        let sub_identifier = if let Some(properties) = subscribe_properties {
            properties.subscription_identifier
        } else {
            None
        };
        let exclusive_sub = self.exclusive_subscribe.get_mut(&topic_id).unwrap();
        let share_sub = self.exclusive_subscribe.get_mut(&topic_id).unwrap();
        let client_sub = self.client_subscribe.get_mut(&client_id).unwrap();

        for filter in subscribe.filters.clone() {
            let mut sub = Subscriber {
                protocol: protocol.clone(),
                client_id: client_id.clone(),
                packet_identifier: subscribe.packet_identifier,
                qos: filter.qos,
                nolocal: filter.nolocal,
                preserve_retain: filter.preserve_retain,
                subscription_identifier: sub_identifier,
                user_properties: Vec::new(),
                is_share_sub: false,
                group_name: None,
            };
            if is_share_sub(filter.path.clone()) {
                let (group_name, sub_name) = decode_share_info(filter.path.clone());
                if path_regex_match(topic_name.clone(), sub_name.clone()) {
                    sub.is_share_sub = true;
                    sub.group_name = Some(group_name);
                    exclusive_sub.insert(client_id.clone(), sub);
                    client_sub.insert(topic_id.clone(), now_second());
                }
            } else {
                if path_regex_match(topic_name.clone(), filter.path.clone()) {
                    share_sub.insert(client_id.clone(), sub);
                    client_sub.insert(topic_id.clone(), now_second());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use crate::metadata::message::Message;
    // use crate::{metadata::topic::Topic, storage::message::MessageStorage};
    // use bytes::Bytes;
    // use clients::poll::ClientPool;
    // use protocol::mqtt::{Filter, MQTTPacket, Subscribe};
    // use std::sync::Arc;
    // use storage_adapter::memory::MemoryStorageAdapter;
    // use storage_adapter::record::Record;
    // use tokio::sync::broadcast;

    #[tokio::test]
    async fn topic_sub_push_thread_test() {
        // let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        // let metadata_cache = Arc::new(MetadataCacheManager::new("test-cluster".to_string()));

        // let client_poll = Arc::new(ClientPool::new(3));

        // // Create topic
        // let topic_name = "/test/topic".to_string();
        // let topic = Topic::new(&topic_name);
        // metadata_cache.set_topic(&topic_name, &topic);

        // // Subscription topic
        // let client_id = "test-ttt".to_string();
        // let packet_identifier = 2;
        // let mut filters = Vec::new();
        // let filter = Filter {
        //     path: "/test/topic".to_string(),
        //     qos: protocol::mqtt::QoS::AtLeastOnce,
        //     nolocal: true,
        //     preserve_retain: true,
        //     retain_forward_rule: protocol::mqtt::RetainForwardRule::Never,
        // };
        // filters.push(filter);
        // let subscribe = Subscribe {
        //     packet_identifier,
        //     filters,
        // };
        // sub_manager
        //     .parse_subscribe(
        //         crate::server::MQTTProtocol::MQTT5,
        //         client_id,
        //         subscribe,
        //         None,
        //         client_poll.clone(),
        //     )
        //     .await;

        // // Start push thread
        // let message_storage = MessageStorage::new(storage_adapter.clone());
        // let (response_queue_sx4, mut response_queue_rx4) = broadcast::channel(1000);
        // let (response_queue_sx5, mut response_queue_rx5) = broadcast::channel(1000);
        // let ms = message_storage.clone();
        // let topic_id: String = topic.topic_id.clone();
        // tokio::spawn(async move {
        //     topic_sub_push_thread(
        //         metadata_cache,
        //         ms,
        //         topic_id,
        //         response_queue_sx4,
        //         response_queue_sx5,
        //     )
        //     .await;
        // });

        // // Send data
        // let mut msg = Message::default();
        // msg.payload = Bytes::from("testtest".to_string());

        // let record = Record::build_b(serde_json::to_vec(&msg).unwrap());
        // message_storage
        //     .append_topic_message(topic.topic_id.clone(), vec![record])
        //     .await
        //     .unwrap();

        // // Receive subscription data
        // loop {
        //     match response_queue_rx5.recv().await {
        //         Ok(packet) => {
        //             if let MQTTPacket::Publish(publish, _) = packet.packet {
        //                 assert_eq!(publish.topic, topic.topic_id);
        //                 assert_eq!(publish.payload, msg.payload);
        //             } else {
        //                 println!("Package does not exist");
        //                 assert!(false);
        //             }
        //             break;
        //         }
        //         Err(e) => {
        //             println!("{}", e)
        //         }
        //     }
        // }
    }
}
