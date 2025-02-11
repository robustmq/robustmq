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

use crate::subscribe::subscriber::Subscriber;
use dashmap::DashMap;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use protocol::mqtt::common::{Filter, MqttProtocol};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;

#[derive(Clone, Serialize, Deserialize)]
pub struct ShareSubShareSub {
    pub client_id: String,
    pub group_name: String,
    pub sub_name: String,
    pub protocol: MqttProtocol,
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
pub struct TopicSubscribeInfo {
    pub client_id: String,
    pub path: String,
}

#[derive(Clone, Default)]
pub struct SubscribeManager {
    //(client_id_path: MqttSubscribe)
    pub subscribe_list: DashMap<String, MqttSubscribe>,

    // (client_id_sub_name_topic_id, Subscriber)
    pub exclusive_push: DashMap<String, Subscriber>,

    // (client_id_sub_name_topic_id, Sender<bool>)
    pub exclusive_push_thread: DashMap<String, Sender<bool>>,

    // (group_name_sub_name_topic_id, ShareLeaderSubscribeData)
    pub share_leader_push: DashMap<String, ShareLeaderSubscribeData>,

    // (group_name_sub_name_topic_id, Sender<bool>)
    pub share_leader_push_thread: DashMap<String, Sender<bool>>,

    // (client_id_group_name_sub_name,ShareSubShareSub)
    pub share_follower_resub: DashMap<String, ShareSubShareSub>,

    // (client_id_group_name_sub_name, Sender<bool>)
    pub share_follower_resub_thread: DashMap<String, Sender<bool>>,

    // (identifier_id，client_id)
    pub share_follower_identifier_id: DashMap<usize, String>,

    //(topic, now)
    pub exclusive_subscribe: DashMap<String, String>,

    //(topic_id, Vec<TopicSubscribeInfo>)
    pub topic_subscribe_list: DashMap<String, Vec<TopicSubscribeInfo>>,
}

impl SubscribeManager {
    pub fn new() -> Self {
        SubscribeManager {
            subscribe_list: DashMap::with_capacity(8),
            exclusive_push: DashMap::with_capacity(8),
            share_leader_push: DashMap::with_capacity(8),
            share_follower_resub: DashMap::with_capacity(8),
            share_follower_identifier_id: DashMap::with_capacity(8),
            exclusive_push_thread: DashMap::with_capacity(8),
            share_leader_push_thread: DashMap::with_capacity(8),
            share_follower_resub_thread: DashMap::with_capacity(8),
            exclusive_subscribe: DashMap::with_capacity(8),
            topic_subscribe_list: DashMap::with_capacity(8),
        }
    }

    // subscribe info
    pub fn add_subscribe(&self, subscribe: MqttSubscribe) {
        let key = self.subscribe_key(&subscribe.client_id, &subscribe.path);
        self.subscribe_list.insert(key, subscribe);
    }

    pub fn get_subscribe(&self, client_id: &str, path: &str) -> Option<MqttSubscribe> {
        let key = self.subscribe_key(client_id, path);
        if let Some(da) = self.subscribe_list.get(&key) {
            return Some(da.clone());
        }
        None
    }

    pub fn remove_subscribe(&self, client_id: &str, path: &str) {
        let key = self.subscribe_key(client_id, path);
        self.subscribe_list.remove(&key);
    }

    // push by exclusive subscribe
    pub fn add_exclusive_push(&self, client_id: &str, path: &str, topic_id: &str, sub: Subscriber) {
        let key = self.exclusive_key(client_id, path, topic_id);
        self.exclusive_push.insert(key, sub);
    }

    pub fn remove_exclusive_push_by_client_id(&self, client_id: &str) {
        for (key, subscriber) in self.exclusive_push.clone() {
            if subscriber.client_id == *client_id {
                self.exclusive_push.remove(&key);
                self.remove_topic_subscribe_by_client_id(
                    &subscriber.topic_name,
                    &subscriber.client_id,
                );
            }
        }
    }

    // push by share subscribe
    pub fn add_share_subscribe_leader(&self, sub_name: &str, sub: Subscriber) {
        let group_name = sub.group_name.clone().unwrap();
        let share_leader_key = self.share_leader_key(&group_name, sub_name, &sub.topic_id);
        let leader_sub_key = self.share_leader_sub_key(&sub.client_id, sub_name);

        if let Some(share_sub) = self.share_leader_push.get_mut(&share_leader_key) {
            share_sub.sub_list.insert(leader_sub_key, sub);
        } else {
            let sub_list = DashMap::with_capacity(8);
            sub_list.insert(leader_sub_key, sub.clone());

            let data = ShareLeaderSubscribeData {
                group_name: group_name.to_owned(),
                topic_id: sub.topic_id.to_owned(),
                topic_name: sub.topic_name.to_owned(),
                sub_name: sub_name.to_owned(),
                sub_list,
            };

            self.share_leader_push
                .insert(share_leader_key.clone(), data);
        }
    }

    pub fn remove_share_subscribe_leader_by_client_id(&self, client_id: &str) {
        for (key, share_sub) in self.share_leader_push.clone() {
            for (sub_key, subscriber) in share_sub.sub_list {
                if subscriber.client_id == *client_id {
                    let mut_data = self.share_leader_push.get_mut(&key).unwrap();
                    mut_data.sub_list.remove(&sub_key);
                    self.remove_topic_subscribe_by_client_id(
                        &subscriber.topic_name,
                        &subscriber.client_id,
                    );
                }
            }
        }
    }

    pub fn add_share_subscribe_follower(
        &self,
        client_id: &str,
        group_name: &str,
        topic_id: &str,
        share_sub: ShareSubShareSub,
    ) {
        let key = self.share_follower_key(client_id, group_name, topic_id);
        self.share_follower_resub.insert(key, share_sub);
    }

    pub fn remove_share_subscribe_follower_by_client_id(&self, client_id: &str) {
        for (key, share_sub) in self.share_follower_resub.clone() {
            if share_sub.client_id == *client_id {
                self.share_follower_resub.remove(&key);
            }
        }
    }

    // topic subscribe
    pub fn add_topic_subscribe(&self, topic_name: &str, client_id: &str, path: &str) {
        if let Some(mut list) = self.topic_subscribe_list.get_mut(topic_name) {
            list.push(TopicSubscribeInfo {
                client_id: client_id.to_owned(),
                path: path.to_owned(),
            });
        } else {
            self.topic_subscribe_list.insert(
                topic_name.to_owned(),
                vec![TopicSubscribeInfo {
                    client_id: client_id.to_owned(),
                    path: path.to_owned(),
                }],
            );
        }
    }

    pub fn contain_topic_subscribe(&self, topic_name: &str) -> bool {
        if let Some(list) = self.topic_subscribe_list.get(topic_name) {
            return !list.is_empty();
        }
        false
    }

    fn remove_topic_subscribe_by_client_id(&self, topic_name: &str, client_id: &str) {
        if let Some(mut list) = self.topic_subscribe_list.get_mut(topic_name) {
            list.retain(|x| x.client_id == *client_id);
        }
    }

    pub fn remove_topic_subscribe_by_path(&self, topic_name: &str, path: &str) {
        if let Some(mut list) = self.topic_subscribe_list.get_mut(topic_name) {
            list.retain(|x| x.path == *path);
        }
    }

    // exclusive subscribe
    pub fn is_exclusive_subscribe(&self, topic: &str) -> bool {
        self.exclusive_subscribe.contains_key(topic)
    }

    pub fn add_exclusive_subscribe(&self, topic: &str, client_id: &str) {
        self.exclusive_subscribe
            .insert(topic.to_owned(), client_id.to_owned());
    }

    pub fn remove_exclusive_subscribe_by_topic(&self, topic: &str) {
        self.exclusive_subscribe.remove(topic);
    }

    pub fn remove_exclusive_subscribe_by_client_id(&self, client_id: &str) {
        for (topic, cid) in self.exclusive_subscribe.clone() {
            if cid == *client_id {
                self.exclusive_subscribe.remove(&topic);
            }
        }
    }

    // key
    fn subscribe_key(&self, client_id: &str, path: &str) -> String {
        format!("{}_{}", client_id, path)
    }

    fn exclusive_key(&self, client_id: &str, sub_name: &str, topic_id: &str) -> String {
        format!("{}_{}_{}", client_id, sub_name, topic_id)
    }

    fn share_leader_key(&self, group_name: &str, sub_name: &str, topic_id: &str) -> String {
        format!("{}_{}_{}", group_name, sub_name, topic_id)
    }

    fn share_leader_sub_key(&self, client_id: &str, sub_path: &str) -> String {
        format!("{}_{}", client_id, sub_path)
    }

    fn share_follower_key(&self, client_id: &str, group_name: &str, topic_id: &str) -> String {
        format!("{}_{}_{}", client_id, group_name, topic_id)
    }
}
