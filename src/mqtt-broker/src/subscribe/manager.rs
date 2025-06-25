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

use crate::subscribe::common::Subscriber;
use dashmap::DashMap;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use protocol::mqtt::common::{Filter, MqttProtocol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast::Sender;

#[derive(Clone, Serialize, Deserialize)]
pub struct ShareSubShareSub {
    pub client_id: String,
    pub group_name: String,
    pub sub_name: String,
    pub protocol: MqttProtocol,
    pub topic_name: String,
    pub topic_id: String,
    pub packet_identifier: u16,
    pub filter: Filter,
    pub subscription_identifier: Option<usize>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ShareLeaderSubscribeData {
    // key
    pub group_name: String,
    pub sub_name: String,
    pub topic_id: String,

    // extend
    pub topic_name: String,

    // (client_id, subscriber)
    pub sub_list: DashMap<String, Subscriber>,
}

#[derive(Clone, Debug)]
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

    // (group_name_topic_id, ShareLeaderSubscribeData)
    pub share_leader_push: DashMap<String, ShareLeaderSubscribeData>,

    // (group_name_topic_id, Sender<bool>)
    pub share_leader_push_thread: DashMap<String, Sender<bool>>,

    // (client_id_group_name_sub_name,ShareSubShareSub)
    pub share_follower_resub: DashMap<String, ShareSubShareSub>,

    // (client_id_group_name_sub_name, Sender<bool>)
    pub share_follower_resub_thread: DashMap<String, Sender<bool>>,

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
            exclusive_push_thread: DashMap::with_capacity(8),
            share_leader_push_thread: DashMap::with_capacity(8),
            share_follower_resub_thread: DashMap::with_capacity(8),
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

    pub fn list_subscribe(&self) -> Vec<String> {
        let mut list = Vec::new();
        for (key, subscribe) in self.subscribe_list.clone() {
            list.push(key);
        }
        list
    }

    pub fn remove_subscribe(&self, client_id: &str, path: &str) {
        let key = self.subscribe_key(client_id, path);
        self.subscribe_list.remove(&key);
    }

    pub fn remove_subscriber_by_client_id(&self, client_id: &str) {
        for (key, subscribe) in self.subscribe_list.clone() {
            if subscribe.client_id == *client_id {
                self.subscribe_list.remove(&key);
            }
        }
    }

    // push by exclusive subscribe
    pub fn add_exclusive_push(&self, client_id: &str, path: &str, topic_id: &str, sub: Subscriber) {
        let key = self.exclusive_key(client_id, path, topic_id);
        self.exclusive_push.insert(key, sub);
    }

    fn remove_exclusive_push_by_client_id(&self, client_id: &str) {
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

    // Leader push by share subscribe
    pub fn add_share_subscribe_leader(&self, sub_name: &str, sub: Subscriber) {
        let group_name = sub.group_name.clone().unwrap();
        let share_leader_key = self.share_leader_key(&group_name, sub_name, &sub.topic_id);

        if let Some(share_sub) = self.share_leader_push.get(&share_leader_key) {
            share_sub.sub_list.insert(sub.client_id.clone(), sub);
        } else {
            let sub_list = DashMap::with_capacity(2);
            sub_list.insert(sub.client_id.clone(), sub.clone());
            self.share_leader_push.insert(
                share_leader_key.clone(),
                ShareLeaderSubscribeData {
                    group_name: group_name.to_owned(),
                    topic_id: sub.topic_id.to_owned(),
                    topic_name: sub.topic_name.to_owned(),
                    sub_name: sub_name.to_owned(),
                    sub_list,
                },
            );
        }
    }

    fn remove_share_subscribe_leader_by_client_id(&self, client_id: &str) {
        for share_sub in self.share_leader_push.iter() {
            if let Some((_, subscriber)) = share_sub.sub_list.remove(client_id) {
                self.remove_topic_subscribe_by_client_id(
                    &subscriber.topic_name,
                    &subscriber.client_id,
                );
            }
        }

        for (key, raw) in self.share_leader_push.clone() {
            if raw.sub_list.is_empty() {
                self.share_leader_push.remove(&key);
            }
        }
    }

    // Follower resub by share subscribe
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

    fn remove_share_subscribe_follower_by_client_id(&self, client_id: &str) {
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

    pub fn is_exclusive_subscribe(&self, topic_name: &str) -> bool {
        if let Some(list) = self.topic_subscribe_list.get(topic_name) {
            for raw in list.iter() {
                if raw.path.starts_with("$exclusive") {
                    return true;
                }
            }
        }
        false
    }

    fn remove_topic_subscribe_by_client_id(&self, topic_name: &str, client_id: &str) {
        if let Some(mut list) = self.topic_subscribe_list.get_mut(topic_name) {
            list.retain(|x| x.client_id != *client_id);
        }
    }

    pub fn remove_topic_subscribe_by_path(&self, topic_name: &str, client_id: &str, path: &str) {
        if let Some(mut list) = self.topic_subscribe_list.get_mut(topic_name) {
            list.retain(|x| x.path != *path && x.client_id != *client_id);
        }
    }

    pub fn remove_client_id(&self, client_id: &str) {
        self.remove_exclusive_push_by_client_id(client_id);
        self.remove_share_subscribe_leader_by_client_id(client_id);
        self.remove_share_subscribe_follower_by_client_id(client_id);
        self.remove_subscriber_by_client_id(client_id);
    }

    // info
    pub fn snapshot_info(&self) -> HashMap<String, Vec<String>> {
        let exclusive_push_key: Vec<String> = self
            .exclusive_push
            .iter()
            .map(|raw| raw.key().clone())
            .collect();

        let exclusive_push_thread_key: Vec<String> = self
            .exclusive_push_thread
            .iter()
            .map(|raw| raw.key().clone())
            .collect();

        let share_leader_push_key: Vec<String> = self
            .share_leader_push
            .iter()
            .map(|raw| raw.key().clone())
            .collect();

        let share_leader_push_thread_key: Vec<String> = self
            .share_leader_push_thread
            .iter()
            .map(|raw| raw.key().clone())
            .collect();

        let share_follower_resub_key: Vec<String> = self
            .share_follower_resub
            .iter()
            .map(|raw| raw.key().clone())
            .collect();

        let share_follower_resub_thread_key: Vec<String> = self
            .share_follower_resub_thread
            .iter()
            .map(|raw| raw.key().clone())
            .collect();

        let mut results = HashMap::new();
        results.insert("exclusive_push_key".to_string(), exclusive_push_key);
        results.insert(
            "exclusive_push_thread_key".to_string(),
            exclusive_push_thread_key,
        );
        results.insert("share_leader_push_key".to_string(), share_leader_push_key);
        results.insert(
            "share_leader_push_thread_key".to_string(),
            share_leader_push_thread_key,
        );
        results.insert(
            "share_follower_resub_key".to_string(),
            share_follower_resub_key,
        );
        results.insert(
            "share_follower_resub_thread_key".to_string(),
            share_follower_resub_thread_key,
        );

        results
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
    fn share_follower_key(&self, client_id: &str, group_name: &str, topic_id: &str) -> String {
        format!("{}_{}_{}", client_id, group_name, topic_id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::tools::{now_second, unique_id};
    use protocol::mqtt::common::{Filter, MqttProtocol, QoS, RetainHandling};

    use crate::subscribe::{
        common::Subscriber,
        manager::{ShareSubShareSub, SubscribeManager},
    };

    #[test]
    fn topic_subscribe_test() {
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let topic_name = "t1";
        let client_id = unique_id();
        let path = "/topic/path";

        subscribe_manager.add_topic_subscribe(topic_name, &client_id, path);
        assert_eq!(subscribe_manager.topic_subscribe_list.len(), 1);
        assert!(subscribe_manager.contain_topic_subscribe(topic_name));
        assert!(!subscribe_manager.is_exclusive_subscribe(topic_name));
        subscribe_manager.remove_topic_subscribe_by_client_id(topic_name, &client_id);
        assert!(!subscribe_manager.contain_topic_subscribe(topic_name));
        assert!(!subscribe_manager.is_exclusive_subscribe(topic_name));

        let path = "$exclusive/path";
        subscribe_manager.add_topic_subscribe(topic_name, &client_id, path);
        assert_eq!(subscribe_manager.topic_subscribe_list.len(), 1);
        assert!(subscribe_manager.contain_topic_subscribe(topic_name));
        assert!(subscribe_manager.is_exclusive_subscribe(topic_name));

        subscribe_manager.remove_topic_subscribe_by_path(topic_name, &client_id, path);
        assert!(!subscribe_manager.contain_topic_subscribe(topic_name));
        assert!(!subscribe_manager.is_exclusive_subscribe(topic_name));
    }

    #[test]
    fn share_subscribe_leader_test() {
        let subscribe_manager = Arc::new(SubscribeManager::new());

        let sub = Subscriber {
            protocol: MqttProtocol::Mqtt5,
            client_id: "client_id_1".to_string(),
            topic_name: "t_name_1".to_string(),
            group_name: Some("g1".to_string()),
            topic_id: "t_id_1".to_string(),
            qos: QoS::AtLeastOnce,
            nolocal: true,
            preserve_retain: true,
            retain_forward_rule: RetainHandling::Never,
            subscription_identifier: None,
            sub_path: "/var/111".to_string(),
            rewrite_sub_path: None,
            create_time: now_second(),
        };
        subscribe_manager.add_share_subscribe_leader("queue_", sub.clone());
        assert_eq!(subscribe_manager.share_leader_push.len(), 1);

        subscribe_manager.remove_share_subscribe_leader_by_client_id(&sub.client_id);
        println!("{:?}", subscribe_manager.share_leader_push);
        assert_eq!(subscribe_manager.share_leader_push.len(), 0);
    }

    #[test]
    fn share_subscribe_followe_test() {
        let share_sub = ShareSubShareSub {
            protocol: MqttProtocol::Mqtt5,
            client_id: "client_id_1".to_string(),
            packet_identifier: 1,
            filter: Filter::default(),
            group_name: "g1".to_string(),
            sub_name: "s1".to_string(),
            subscription_identifier: None,
            topic_id: "t1".to_string(),
            topic_name: "tname".to_string(),
        };
        let subscribe_manager = Arc::new(SubscribeManager::new());
        subscribe_manager.add_share_subscribe_follower(
            &share_sub.client_id,
            &share_sub.group_name,
            &share_sub.topic_id,
            share_sub.clone(),
        );
        subscribe_manager.remove_share_subscribe_follower_by_client_id(&share_sub.client_id);
        assert_eq!(subscribe_manager.share_follower_resub.len(), 0);
    }
}
