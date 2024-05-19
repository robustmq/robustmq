use super::subscribe::{
    decode_share_info, is_contain_rewrite_flag, is_share_sub, path_regex_match,
};
use crate::core::metadata_cache::MetadataCacheManager;
use crate::metadata::subscriber::Subscriber;
use crate::server::MQTTProtocol;
use clients::{placement::mqtt::call::placement_get_share_sub, poll::ClientPool};
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
    pub sub_path: String,
    pub leader_id: u64,
    pub leader_addr: String,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
}

pub struct SubscribeManager {
    client_poll: Arc<ClientPool>,
    metadata_cache: Arc<MetadataCacheManager>,

    // (client_id,Vec<Subscriber>)
    pub exclusive_subscribe: DashMap<String, Vec<Subscriber>>,

    // (client_id,Vec<Subscriber>)
    pub share_leader_subscribe: DashMap<String, Vec<Subscriber>>,

    // (client_id,ShareSubShareSub)
    pub share_follower_subscribe: DashMap<String, ShareSubShareSub>,

    // (identifier_id，client_id)
    pub share_follower_identifier_id: DashMap<usize, String>,
}

impl SubscribeManager {
    pub fn new(metadata_cache: Arc<MetadataCacheManager>, client_poll: Arc<ClientPool>) -> Self {
        return SubscribeManager {
            client_poll,
            metadata_cache,
            exclusive_subscribe: DashMap::with_capacity(8),
            share_leader_subscribe: DashMap::with_capacity(8),
            share_follower_subscribe: DashMap::with_capacity(8),
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

    // Handle subscriptions to new topics
    pub async fn parse_subscribe_by_new_topic(&self) {
        for (topic_name, topic) in self.metadata_cache.topic_info.clone() {
            for (client_id, data) in self.metadata_cache.subscribe_filter.clone() {
                let subscribe = data.subscribe;
                let subscribe_properties = data.subscribe_properties;
                self.parse_subscribe(
                    topic_name.clone(),
                    topic.topic_id.clone(),
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
            self.parse_subscribe(
                topic_name,
                topic.topic_id,
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
                    self.exclusive_subscribe.remove(&client_id);
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

        if !self.exclusive_subscribe.contains_key(&client_id) {
            self.exclusive_subscribe
                .insert(client_id.clone(), Vec::new());
        }

        if !self.share_leader_subscribe.contains_key(&topic_id) {
            self.share_leader_subscribe
                .insert(topic_id.clone(), Vec::new());
        }

        let mut exclusive_sub = self.exclusive_subscribe.get_mut(&client_id).unwrap();
        let share_sub_leader = self.share_leader_subscribe.get_mut(&topic_id).unwrap();

        let conf = broker_mqtt_conf();

        for filter in subscribe.filters.clone() {
            let mut sub = Subscriber {
                protocol: protocol.clone(),
                client_id: client_id.clone(),
                topic_name: topic_name.clone(),
                topic_id: topic_id.clone(),
                qos: filter.qos,
                nolocal: filter.nolocal,
                preserve_retain: filter.preserve_retain,
                subscription_identifier: sub_identifier,
                is_contain_rewrite_flag: false,
                sub_path: filter.path.clone(),
            };
            if is_share_sub(filter.path.clone()) {
                let (group_name, sub_name) = decode_share_info(filter.path.clone());
                if path_regex_match(topic_name.clone(), sub_name.clone()) {
                    match self
                        .get_share_sub_leader(group_name.clone(), sub_name.clone())
                        .await
                    {
                        Ok(reply) => {
                            if reply.broker_id == conf.broker_id {
                                if let Some(properties) = subscribe_properties.clone() {
                                    if is_contain_rewrite_flag(properties.user_properties) {
                                        sub.is_contain_rewrite_flag = true;
                                    }
                                }
                                share_sub_leader.insert(client_id.clone(), sub);
                            } else {
                                let identifier_id = SUB_IDENTIFIER_ID_BUILD
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                let share_sub = ShareSubShareSub {
                                    identifier_id,
                                    client_id: client_id.clone(),
                                    sub_path: filter.path.clone(),
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
                }
            } else {
                if path_regex_match(topic_name.clone(), filter.path.clone()) {
                    exclusive_sub.push(sub);
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
    #[tokio::test]
    async fn topic_sub_push_thread_test() {}
}
