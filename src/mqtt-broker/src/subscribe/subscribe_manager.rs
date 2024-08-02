use super::sub_common::{decode_share_info, get_share_sub_leader, is_share_sub, path_regex_match};
use crate::handler::cache_manager::CacheManager;
use crate::subscribe::subscriber::Subscriber;
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info},
};
use dashmap::DashMap;
use protocol::mqtt::common::{Filter, MQTTProtocol, Subscribe, SubscribeProperties};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{sync::broadcast::Sender, time::sleep};

#[derive(Clone, Serialize, Deserialize)]
pub struct ShareSubShareSub {
    pub client_id: String,
    pub group_name: String,
    pub sub_name: String,
    pub protocol: MQTTProtocol,
    pub packet_identifier: u16,
    pub filter: Filter,
    pub subscription_identifier: Option<usize>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ShareLeaderSubscribeData {
    pub group_name: String,
    pub topic_id: String,
    pub topic_name: String,
    // (client_id_sub_path, subscriber)
    pub sub_list: DashMap<String, Subscriber>,
}

#[derive(Clone)]
pub struct SubscribeManager {
    client_poll: Arc<ClientPool>,
    metadata_cache: Arc<CacheManager>,

    // (client_id_topic_id, Subscriber)
    pub exclusive_subscribe: DashMap<String, Subscriber>,

    // (client_id_topic_id, Sender<bool>)
    pub exclusive_push_thread: DashMap<String, Sender<bool>>,

    // (group_name_topic_id, ShareLeaderSubscribeData)
    pub share_leader_subscribe: DashMap<String, ShareLeaderSubscribeData>,

    // (group_name_topic_id, Sender<bool>)
    pub share_leader_push_thread: DashMap<String, Sender<bool>>,

    // (client_id_group_name_sub_name,ShareSubShareSub)
    pub share_follower_subscribe: DashMap<String, ShareSubShareSub>,

    // (client_id_group_name_sub_name, Sender<bool>)
    pub share_follower_resub_thread: DashMap<String, Sender<bool>>,

    // (identifier_id，client_id)
    pub share_follower_identifier_id: DashMap<usize, String>,
}

impl SubscribeManager {
    pub fn new(metadata_cache: Arc<CacheManager>, client_poll: Arc<ClientPool>) -> Self {
        return SubscribeManager {
            client_poll,
            metadata_cache,
            exclusive_subscribe: DashMap::with_capacity(8),
            share_leader_subscribe: DashMap::with_capacity(8),
            share_follower_subscribe: DashMap::with_capacity(8),
            share_follower_identifier_id: DashMap::with_capacity(8),
            exclusive_push_thread: DashMap::with_capacity(8),
            share_leader_push_thread: DashMap::with_capacity(8),
            share_follower_resub_thread: DashMap::with_capacity(8),
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
            for (client_id, sub_list) in self.metadata_cache.subscribe_filter.clone() {
                for (_, data) in sub_list {
                    let subscribe = Subscribe {
                        packet_identifier: 0,
                        filters: vec![data.filter],
                    };
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

    pub fn stop_push_by_client_id(&self, client_id: &String) {
        for (key, subscriber) in self.exclusive_subscribe.clone() {
            if subscriber.client_id == *client_id {
                self.exclusive_subscribe.remove(&key);
            }
        }

        for (key, share_sub) in self.share_leader_subscribe.clone() {
            for (sub_key, subscriber) in share_sub.sub_list {
                if subscriber.client_id == *client_id {
                    let mut_data = self.share_leader_subscribe.get_mut(&key).unwrap();
                    mut_data.sub_list.remove(&sub_key);
                }
            }
        }

        for (key, share_sub) in self.share_follower_subscribe.clone() {
            if share_sub.client_id == *client_id {
                self.share_follower_subscribe.remove(&key);
            }
        }
    }

    pub fn remove_subscribe(&self, client_id: &String, filter_path: &Vec<String>) {
        for (topic_name, _) in self.metadata_cache.topic_info.clone() {
            for path in filter_path.clone() {
                if !path_regex_match(topic_name.clone(), path.clone()) {
                    continue;
                }

                // exclusive
                for (key, subscriber) in self.exclusive_subscribe.clone() {
                    if subscriber.client_id == *client_id && subscriber.sub_path == path {
                        if let Some(sx) = self.exclusive_push_thread.get(&key) {
                            match sx.send(true) {
                                Ok(_) => {}
                                Err(e) => error(e.to_string()),
                            }
                            self.exclusive_subscribe.remove(&key);
                        }
                    }
                }

                // share leader
                for (key, data) in self.share_leader_subscribe.clone() {
                    let mut flag = false;
                    for (sub_key, share_sub) in data.sub_list {
                        if share_sub.client_id == *client_id && share_sub.sub_path == path {
                            let mut_data = self.share_leader_subscribe.get_mut(&key).unwrap();
                            mut_data.sub_list.remove(&sub_key);
                            flag = true;
                        }
                    }

                    if flag {
                        if let Some(sx) = self.share_leader_push_thread.get(&key) {
                            match sx.send(true) {
                                Ok(_) => {}
                                Err(e) => error(e.to_string()),
                            }
                        }
                    }
                }

                // share follower
                for (key, data) in self.share_follower_subscribe.clone() {
                    if data.client_id == *client_id && data.filter.path == path {
                        self.share_follower_subscribe.remove(&key);
                        if let Some(sx) = self.share_follower_resub_thread.get(&key) {
                            match sx.send(true) {
                                Ok(_) => {}
                                Err(e) => error(e.to_string()),
                            }
                        }
                    }
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

        let conf = broker_mqtt_conf();

        for filter in subscribe.filters.clone() {
            let mut sub = Subscriber {
                protocol: protocol.clone(),
                client_id: client_id.clone(),
                topic_name: topic_name.clone(),
                group_name: None,
                topic_id: topic_id.clone(),
                qos: filter.qos,
                nolocal: filter.nolocal,
                preserve_retain: filter.preserve_retain,
                retain_forward_rule: filter.retain_forward_rule.clone(),
                subscription_identifier: sub_identifier,
                sub_path: filter.path.clone(),
            };
            if is_share_sub(filter.path.clone()) {
                let (group_name, sub_name) = decode_share_info(filter.path.clone());
                if path_regex_match(topic_name.clone(), sub_name.clone()) {
                    match get_share_sub_leader(self.client_poll.clone(), group_name.clone()).await {
                        Ok(reply) => {
                            if reply.broker_id == conf.broker_id {
                                let leader_key =
                                    self.share_leader_key(group_name.clone(), topic_id.clone());

                                if !self.share_leader_subscribe.contains_key(&leader_key) {
                                    let data = ShareLeaderSubscribeData {
                                        group_name: group_name.clone(),
                                        topic_id: topic_id.clone(),
                                        topic_name: topic_name.clone(),
                                        sub_list: DashMap::with_capacity(8),
                                    };
                                    self.share_leader_subscribe.insert(leader_key.clone(), data);
                                }

                                let share_sub_leader =
                                    self.share_leader_subscribe.get_mut(&leader_key).unwrap();

                                sub.group_name = Some(group_name);
                                let leader_sub_key = self
                                    .share_leader_sub_key(client_id.clone(), filter.path.clone());
                                if !share_sub_leader.sub_list.contains_key(&leader_sub_key) {
                                    share_sub_leader.sub_list.insert(leader_sub_key, sub);
                                }
                            } else {
                                let share_sub = ShareSubShareSub {
                                    client_id: client_id.clone(),
                                    protocol: protocol.clone(),
                                    packet_identifier: subscribe.packet_identifier,
                                    filter: filter.clone(),
                                    group_name: group_name.clone(),
                                    sub_name: sub_name.clone(),
                                    subscription_identifier: if let Some(properties) =
                                        subscribe_properties.clone()
                                    {
                                        properties.subscription_identifier
                                    } else {
                                        None
                                    },
                                };
                                self.share_follower_subscribe.insert(
                                    self.share_follower_key(
                                        client_id.clone(),
                                        group_name,
                                        topic_id.clone(),
                                    ),
                                    share_sub,
                                );
                            }
                        }
                        Err(e) => {
                            error(e.to_string());
                        }
                    }
                }
            } else {
                if path_regex_match(topic_name.clone(), filter.path.clone()) {
                    self.exclusive_subscribe
                        .insert(self.exclusive_key(client_id.clone(), topic_id.clone()), sub);
                }
            }
        }
    }

    pub fn exclusive_push_thread_keys(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (key, _) in self.exclusive_push_thread.clone() {
            result.push(key);
        }
        return result;
    }

    pub fn share_leader_push_thread_keys(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (key, _) in self.share_leader_push_thread.clone() {
            result.push(key);
        }
        return result;
    }

    pub fn share_follower_resub_thread_keys(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (key, _) in self.share_follower_resub_thread.clone() {
            result.push(key);
        }
        return result;
    }

    pub fn exclusive_key(&self, client_id: String, topic_id: String) -> String {
        return format!("{}_{}", client_id, topic_id);
    }

    fn share_leader_key(&self, group_name: String, topic_id: String) -> String {
        return format!("{}_{}", group_name, topic_id);
    }

    fn share_leader_sub_key(&self, client_id: String, sub_path: String) -> String {
        return format!("{}_{}", client_id, sub_path);
    }

    fn share_follower_key(
        &self,
        client_id: String,
        group_name: String,
        topic_id: String,
    ) -> String {
        return format!("{}_{}_{}", client_id, group_name, topic_id);
    }
}

#[cfg(test)]
mod tests {}
