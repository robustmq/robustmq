// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;
use std::time::Duration;

use common_base::config::broker_mqtt::broker_mqtt_conf;
use dashmap::DashMap;
use grpc_clients::poll::ClientPool;
use log::{error, info};
use protocol::mqtt::common::{Filter, MQTTProtocol, Subscribe, SubscribeProperties};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;

use super::sub_common::{decode_share_info, get_share_sub_leader, is_share_sub, path_regex_match};
use crate::handler::cache::CacheManager;
use crate::subscribe::subscriber::Subscriber;

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
    pub sub_name: String,
    // (client_id_sub_path, subscriber)
    pub sub_list: DashMap<String, Subscriber>,
}

#[derive(Clone)]
pub struct SubscribeManager {
    client_poll: Arc<ClientPool>,
    metadata_cache: Arc<CacheManager>,

    // (client_id_sub_name_topic_id, Subscriber)
    pub exclusive_subscribe: DashMap<String, Subscriber>,

    // (client_id_sub_name_topic_id, Sender<bool>)
    pub exclusive_push_thread: DashMap<String, Sender<bool>>,

    // (group_name_sub_name_topic_id, ShareLeaderSubscribeData)
    pub share_leader_subscribe: DashMap<String, ShareLeaderSubscribeData>,

    // (group_name_sub_name_topic_id, Sender<bool>)
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
        SubscribeManager {
            client_poll,
            metadata_cache,
            exclusive_subscribe: DashMap::with_capacity(8),
            share_leader_subscribe: DashMap::with_capacity(8),
            share_follower_subscribe: DashMap::with_capacity(8),
            share_follower_identifier_id: DashMap::with_capacity(8),
            exclusive_push_thread: DashMap::with_capacity(8),
            share_leader_push_thread: DashMap::with_capacity(8),
            share_follower_resub_thread: DashMap::with_capacity(8),
        }
    }

    pub async fn start(&self) {
        info!("Subscribe manager thread started successfully.");
        loop {
            self.parse_subscribe_by_new_topic().await;
            sleep(Duration::from_secs(10)).await;
        }
    }

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

    pub fn stop_push_by_client_id(&self, client_id: &str) {
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

    pub fn remove_subscribe(&self, client_id: &str, filter_path: &[String]) {
        for (topic_name, _) in self.metadata_cache.topic_info.clone() {
            for path in filter_path {
                if !path_regex_match(topic_name.clone(), path.clone()) {
                    continue;
                }

                if is_share_sub(path.clone()) {
                    let (group_name, sub_name) = decode_share_info(path.clone());
                    // share leader
                    for (key, data) in self.share_leader_subscribe.clone() {
                        let mut flag = false;
                        for (sub_key, share_sub) in data.sub_list {
                            if share_sub.client_id == *client_id
                                && (share_sub.group_name.is_some()
                                    && share_sub.group_name.unwrap() == group_name)
                                && share_sub.sub_path == sub_name
                            {
                                let mut_data = self.share_leader_subscribe.get_mut(&key).unwrap();
                                mut_data.sub_list.remove(&sub_key);
                                flag = true;
                            }
                        }

                        if flag {
                            if let Some(sx) = self.share_leader_push_thread.get(&key) {
                                match sx.send(true) {
                                    Ok(_) => {}
                                    Err(e) => error!("{}", e),
                                }
                            }
                        }
                    }

                    // share follower
                    for (key, data) in self.share_follower_subscribe.clone() {
                        if data.client_id == *client_id && data.filter.path == *path {
                            self.share_follower_subscribe.remove(&key);
                            if let Some(sx) = self.share_follower_resub_thread.get(&key) {
                                match sx.send(true) {
                                    Ok(_) => {}
                                    Err(e) => error!("{}", e),
                                }
                            }
                        }
                    }
                } else {
                    for (key, subscriber) in self.exclusive_subscribe.clone() {
                        if subscriber.client_id == *client_id && subscriber.sub_path == *path {
                            if let Some(sx) = self.exclusive_push_thread.get(&key) {
                                match sx.send(true) {
                                    Ok(_) => {}
                                    Err(e) => error!("{}", e),
                                }
                                self.exclusive_subscribe.remove(&key);
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

        for filter in subscribe.filters.clone() {
            if is_share_sub(filter.path.clone()) {
                let conf = broker_mqtt_conf();
                let (group_name, sub_name) = decode_share_info(filter.path.clone());

                if path_regex_match(topic_name.clone(), sub_name.clone()) {
                    match get_share_sub_leader(self.client_poll.clone(), group_name.clone()).await {
                        Ok(reply) => {
                            if reply.broker_id == conf.broker_id {
                                self.parse_share_subscribe_leader(
                                    topic_name.clone(),
                                    topic_id.clone(),
                                    client_id.clone(),
                                    protocol.clone(),
                                    sub_identifier,
                                    filter,
                                    group_name.clone(),
                                    sub_name.clone(),
                                )
                                .await;
                            } else {
                                self.parse_share_subscribe_follower(
                                    topic_id.clone(),
                                    client_id.clone(),
                                    protocol.clone(),
                                    sub_identifier,
                                    filter,
                                    group_name.clone(),
                                    sub_name.clone(),
                                    subscribe.clone(),
                                )
                                .await;
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to get Leader for shared subscription, error message: {}",
                                e
                            );
                        }
                    }
                }
            } else {
                self.parse_exclusive_subscribe(
                    topic_name.clone(),
                    topic_id.clone(),
                    client_id.clone(),
                    protocol.clone(),
                    sub_identifier,
                    filter,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn parse_share_subscribe_leader(
        &self,
        topic_name: String,
        topic_id: String,
        client_id: String,
        protocol: MQTTProtocol,
        sub_identifier: Option<usize>,
        filter: Filter,
        group_name: String,
        sub_name: String,
    ) {
        let share_leader_key = self.share_leader_key(&group_name, &sub_name, &topic_id);
        let leader_sub_key = self.share_leader_sub_key(client_id.clone(), filter.path.clone());

        if let Some(share_sub) = self.share_leader_subscribe.get_mut(&share_leader_key) {
            let sub = Subscriber {
                protocol: protocol.clone(),
                client_id: client_id.clone(),
                topic_name: topic_name.clone(),
                group_name: Some(group_name.clone()),
                topic_id: topic_id.clone(),
                qos: filter.qos,
                nolocal: filter.nolocal,
                preserve_retain: filter.preserve_retain,
                retain_forward_rule: filter.retain_forward_rule.clone(),
                subscription_identifier: sub_identifier,
                sub_path: filter.path.clone(),
            };
            share_sub.sub_list.insert(leader_sub_key, sub);
        } else {
            let sub = Subscriber {
                protocol: protocol.clone(),
                client_id: client_id.clone(),
                topic_name: topic_name.clone(),
                group_name: Some(group_name.clone()),
                topic_id: topic_id.clone(),
                qos: filter.qos,
                nolocal: filter.nolocal,
                preserve_retain: filter.preserve_retain,
                retain_forward_rule: filter.retain_forward_rule.clone(),
                subscription_identifier: sub_identifier,
                sub_path: filter.path.clone(),
            };

            let sub_list = DashMap::with_capacity(8);
            sub_list.insert(leader_sub_key, sub);

            let data = ShareLeaderSubscribeData {
                group_name: group_name.clone(),
                topic_id: topic_id.clone(),
                topic_name: topic_name.clone(),
                sub_name: sub_name.clone(),
                sub_list,
            };

            self.share_leader_subscribe
                .insert(share_leader_key.clone(), data);
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn parse_share_subscribe_follower(
        &self,
        topic_id: String,
        client_id: String,
        protocol: MQTTProtocol,
        sub_identifier: Option<usize>,
        filter: Filter,
        group_name: String,
        sub_name: String,
        subscribe: Subscribe,
    ) {
        let share_sub = ShareSubShareSub {
            client_id: client_id.clone(),
            protocol: protocol.clone(),
            packet_identifier: subscribe.packet_identifier,
            filter: filter.clone(),
            group_name: group_name.clone(),
            sub_name: sub_name.clone(),
            subscription_identifier: sub_identifier,
        };

        let key = self.share_follower_key(client_id.clone(), group_name, topic_id.clone());
        self.share_follower_subscribe.insert(key, share_sub);
    }

    fn parse_exclusive_subscribe(
        &self,
        topic_name: String,
        topic_id: String,
        client_id: String,
        protocol: MQTTProtocol,
        sub_identifier: Option<usize>,
        filter: Filter,
    ) {
        if path_regex_match(topic_name.clone(), filter.path.clone()) {
            let key = self.exclusive_key(&client_id, &filter.path, &topic_id);
            let sub = Subscriber {
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

            self.exclusive_subscribe.insert(key, sub);
        }
    }

    fn exclusive_key(&self, client_id: &str, sub_name: &str, topic_id: &str) -> String {
        format!("{}_{}_{}", client_id, sub_name, topic_id)
    }

    fn share_leader_key(&self, group_name: &str, sub_name: &str, topic_id: &str) -> String {
        format!("{}_{}_{}", group_name, sub_name, topic_id)
    }

    fn share_leader_sub_key(&self, client_id: String, sub_path: String) -> String {
        format!("{}_{}", client_id, sub_path)
    }

    fn share_follower_key(
        &self,
        client_id: String,
        group_name: String,
        topic_id: String,
    ) -> String {
        format!("{}_{}_{}", client_id, group_name, topic_id)
    }
}
