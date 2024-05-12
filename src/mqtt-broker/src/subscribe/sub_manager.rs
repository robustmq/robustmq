use super::subscribe::{decode_share_info, is_share_sub, path_regex_match};
use crate::core::metadata_cache::MetadataCacheManager;
use crate::metadata::subscriber::Subscriber;
use crate::server::MQTTProtocol;
use clients::{placement::mqtt::call::placement_get_share_sub, poll::ClientPool};
use common_base::tools::now_second;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    errors::RobustMQError,
    log::{error, info},
};
use dashmap::DashMap;
use protocol::{
    mqtt::{Subscribe, SubscribeProperties},
    placement_center::generate::mqtt::{GetShareSubReply, GetShareSubRequest},
};
use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use tokio::time::sleep;

static SUB_IDENTIFIER_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct ShareSubShareSub {
    pub identifier_id: u64,
    pub client_id: String,
    pub protocol: MQTTProtocol,
    pub group_name: String,
    pub sub_name: String,
    pub leader_id: u64,
    pub leader_addr: String,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
}

pub struct SubscribeManager {
    client_poll: Arc<ClientPool>,
    metadata_cache: Arc<MetadataCacheManager>,

    // (topic_id,(client_id,Subscriber))
    pub exclusive_subscribe: DashMap<String, DashMap<String, Subscriber>>,

    // (topic_id,(client_id,Subscriber))
    pub share_leader_subscribe: DashMap<String, DashMap<String, Subscriber>>,

    // (client_id,ShareSubShareSub)
    pub share_follower_subscribe: DashMap<String, ShareSubShareSub>,

    // (identifier_id，client_id)
    pub share_follower_identifier_id: DashMap<usize, String>,

    // (client_id, (topic_id, now_second()))
    pub client_subscribe: DashMap<String, DashMap<String, u64>>,
}

impl SubscribeManager {
    pub fn new(metadata_cache: Arc<MetadataCacheManager>, client_poll: Arc<ClientPool>) -> Self {
        return SubscribeManager {
            client_poll,
            metadata_cache,
            exclusive_subscribe: DashMap::with_capacity(8),
            share_leader_subscribe: DashMap::with_capacity(8),
            share_follower_subscribe: DashMap::with_capacity(8),
            client_subscribe: DashMap::with_capacity(8),
            share_follower_identifier_id: DashMap::with_capacity(8),
        };
    }

    pub async fn start(&self) {
        info("Subscribe manager thread started successfully.".to_string());
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
                .await;
            }
        }
    }

    pub async fn add_subscribe(
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
            )
            .await;
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
                    if let Some(sub_list) = self.share_leader_subscribe.get_mut(&topic_id) {
                        sub_list.remove(&client_id);
                    }
                } else {
                    if let Some(sub_list) = self.exclusive_subscribe.get_mut(&topic_id) {
                        sub_list.remove(&client_id);
                    }
                }

                if let Some(client_list) = self.client_subscribe.get_mut(&client_id) {
                    client_list.remove(&topic_id);
                }
            }
        }
    }

    async fn parse_subscribe(
        &self,
        topic_name: String,
        topic_id: String,
        client_id: String,
        protocol: MQTTProtocol,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) {
        let sub_identifier = if let Some(properties) = subscribe_properties.clone() {
            properties.subscription_identifier
        } else {
            None
        };
        let exclusive_sub = self.exclusive_subscribe.get_mut(&topic_id).unwrap();
        let share_sub_leader = self.share_leader_subscribe.get_mut(&topic_id).unwrap();

        let client_sub = self.client_subscribe.get_mut(&client_id).unwrap();

        let conf = broker_mqtt_conf();
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
                    sub.group_name = Some(group_name.clone());
                    match self
                        .get_share_sub_leader(group_name.clone(), sub_name.clone())
                        .await
                    {
                        Ok(reply) => {
                            if reply.broker_id == conf.broker_id {
                                share_sub_leader.insert(client_id.clone(), sub);
                            } else {
                                let identifier_id = SUB_IDENTIFIER_ID_BUILD
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                let share_sub = ShareSubShareSub {
                                    identifier_id,
                                    client_id: client_id.clone(),
                                    group_name: group_name.clone(),
                                    sub_name: sub_name.clone(),
                                    protocol: protocol.clone(),
                                    leader_id: reply.broker_id,
                                    leader_addr: reply.broker_ip,
                                    subscribe: subscribe.clone(),
                                    subscribe_properties: subscribe_properties.clone(),
                                };
                                self.share_follower_subscribe
                                    .insert(client_id.clone(), share_sub);
                                self.share_follower_identifier_id
                                    .insert(identifier_id as usize, client_id.clone());
                            }
                        }
                        Err(e) => {
                            error(e.to_string());
                        }
                    }

                    client_sub.insert(topic_id.clone(), now_second());
                }
            } else {
                if path_regex_match(topic_name.clone(), filter.path.clone()) {
                    exclusive_sub.insert(client_id.clone(), sub);
                    client_sub.insert(topic_id.clone(), now_second());
                }
            }
        }
    }

    pub async fn get_share_sub_leader(
        &self,
        group_name: String,
        sub_name: String,
    ) -> Result<GetShareSubReply, RobustMQError> {
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
                return Ok(reply);
            }
            Err(e) => {
                return Err(e);
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
