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
use common_base::tools::now_second;
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
    pub sub_path: String,
    pub protocol: MqttProtocol,
    pub topic_name: String,
    pub packet_identifier: u16,
    pub filter: Filter,
    pub subscription_identifier: Option<usize>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ShareLeaderSubscribeData {
    pub path: String,

    // group
    pub group_name: String,
    pub sub_path: String,

    // topic
    pub topic_name: String,

    // (client_id, subscriber)
    pub sub_list: DashMap<String, Subscriber>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TopicSubscribeInfo {
    pub client_id: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct TemporaryNotPushClient {
    pub client_id: String,
    pub last_check_time: u64,
}

#[derive(Clone, Serialize)]
pub struct SubPushThreadData {
    pub push_success_record_num: u64,
    pub push_error_record_num: u64,
    pub last_push_time: u64,
    pub last_run_time: u64,
    pub create_time: u64,
    #[serde(skip_serializing, skip_deserializing)]
    pub sender: Sender<bool>,
}

#[derive(Clone)]
pub struct SubscribeManager {
    //(client_id_path: MqttSubscribe)
    subscribe_list: DashMap<String, MqttSubscribe>,

    // (client_id_sub_path_topic_name, Subscriber)
    exclusive_push: DashMap<String, Subscriber>,

    // (client_id_sub_path_topic_name, SubPushThreadData)
    exclusive_push_thread: DashMap<String, SubPushThreadData>,

    // (group_name_topic_name, ShareLeaderSubscribeData)
    share_leader_push: DashMap<String, ShareLeaderSubscribeData>,

    // (group_name_topic_name, SubPushThreadData)
    share_leader_push_thread: DashMap<String, SubPushThreadData>,

    // (client_id_group_name_topic_name,ShareSubShareSub)
    share_follower_resub: DashMap<String, ShareSubShareSub>,

    // (client_id_group_name_topic_name, SubPushThreadData)
    share_follower_resub_thread: DashMap<String, SubPushThreadData>,

    //(topic_name, Vec<TopicSubscribeInfo>)
    topic_subscribes: DashMap<String, Vec<TopicSubscribeInfo>>,

    //(client_id,(path,Vec<String>))
    subscribe_topics: DashMap<String, DashMap<String, Vec<String>>>,

    //(client_id, TemporaryNotPushClient)
    not_push_client: DashMap<String, TemporaryNotPushClient>,
}

impl Default for SubscribeManager {
    fn default() -> Self {
        Self::new()
    }
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
            topic_subscribes: DashMap::with_capacity(8),
            not_push_client: DashMap::with_capacity(8),
            subscribe_topics: DashMap::with_capacity(8),
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

    pub fn list_subscribe(&self) -> DashMap<String, MqttSubscribe> {
        self.subscribe_list.clone()
    }

    pub fn subscribe_list_len(&self) -> u64 {
        self.subscribe_list.len() as u64
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

    // exclusive push data
    pub fn add_exclusive_push(
        &self,
        client_id: &str,
        path: &str,
        topic_name: &str,
        sub: Subscriber,
    ) {
        let key = self.exclusive_key(client_id, path, topic_name);
        self.exclusive_push.insert(key, sub);
        self.add_subscribe_topics(client_id, path, topic_name);
    }

    pub fn exclusive_push_list(&self) -> DashMap<String, Subscriber> {
        self.exclusive_push.clone()
    }

    pub fn exclusive_push_len(&self) -> u64 {
        self.exclusive_push.len() as u64
    }

    pub fn get_exclusive_push(&self, exclusive_key: &str) -> Option<Subscriber> {
        if let Some(data) = self.exclusive_push.get(exclusive_key) {
            return Some(data.clone());
        }

        None
    }

    pub fn contain_exclusive_push(&self, exclusive_key: &str) -> bool {
        self.exclusive_push.contains_key(exclusive_key)
    }

    pub fn remove_contain_exclusive_push(&self, exclusive_key: &str) {
        self.exclusive_push.remove(exclusive_key);
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

    // exclusive push thread
    pub fn exclusive_push_thread_list(&self) -> DashMap<String, SubPushThreadData> {
        self.exclusive_push_thread.clone()
    }

    pub fn remove_exclusive_push_thread(&self, exclusive_key: &str) {
        self.exclusive_push_thread.remove(exclusive_key);
    }

    pub fn exclusive_push_thread_len(&self) -> u64 {
        self.exclusive_push_thread.len() as u64
    }

    pub fn get_exclusive_push_thread(&self, exclusive_key: &str) -> Option<SubPushThreadData> {
        if let Some(data) = self.exclusive_push_thread.get(exclusive_key) {
            return Some(data.clone());
        }
        None
    }

    pub fn contain_exclusive_push_thread(&self, exclusive_key: &str) -> bool {
        self.exclusive_push_thread.contains_key(exclusive_key)
    }

    pub fn add_exclusive_push_thread(&self, exclusive_key: String, sub: SubPushThreadData) {
        self.exclusive_push_thread.insert(exclusive_key, sub);
    }

    pub fn update_exclusive_push_thread_info(
        &self,
        key: &str,
        success_record_num: u64,
        error_record_num: u64,
    ) {
        if let Some(mut thread) = self.exclusive_push_thread.get_mut(key) {
            thread.last_run_time = now_second();
            if (success_record_num + error_record_num) > 0 {
                thread.push_success_record_num += success_record_num;
                thread.push_error_record_num += error_record_num;
                thread.last_push_time = now_second();
            }
        }
    }

    // Leader push by share subscribe
    pub fn share_leader_push_list(&self) -> DashMap<String, ShareLeaderSubscribeData> {
        self.share_leader_push.clone()
    }

    pub fn share_leader_push_len(&self) -> u64 {
        self.share_leader_push.len() as u64
    }

    pub fn contain_share_leader_push(&self, share_leader_key: &str) -> bool {
        self.share_leader_push.contains_key(share_leader_key)
    }

    pub fn remove_share_leader_push(&self, share_leader_key: &str) {
        self.share_leader_push.remove(share_leader_key);
    }

    pub fn get_share_leader_push(
        &self,
        share_leader_key: &str,
    ) -> Option<ShareLeaderSubscribeData> {
        if let Some(data) = self.share_leader_push.get(share_leader_key) {
            return Some(data.clone());
        }
        None
    }

    pub fn add_share_subscribe_leader(&self, sub_path: &str, sub: Subscriber) {
        let group_name = sub.group_name.clone().unwrap();
        let share_leader_key = self.share_leader_key(&group_name, sub_path, &sub.topic_name);

        // add topic-sub, sub-topic
        self.add_subscribe_topics(&sub.client_id, &sub.sub_path, &sub.topic_name);
        self.add_topic_subscribe(&sub.topic_name, &sub.client_id, &sub.sub_path);

        // add leader push data
        if let Some(share_sub) = self.share_leader_push.get(&share_leader_key) {
            share_sub
                .sub_list
                .insert(sub.client_id.clone(), sub.clone());
        } else {
            let sub_list = DashMap::with_capacity(2);
            sub_list.insert(sub.client_id.clone(), sub.clone());
            self.share_leader_push.insert(
                share_leader_key.clone(),
                ShareLeaderSubscribeData {
                    group_name: group_name.to_owned(),
                    topic_name: sub.topic_name.to_owned(),
                    sub_path: sub_path.to_owned(),
                    path: sub.sub_path,
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

    // share_leader_push_thread
    pub fn share_leader_push_thread_list(&self) -> DashMap<String, SubPushThreadData> {
        self.share_leader_push_thread.clone()
    }

    pub fn share_leader_push_thread_len(&self) -> u64 {
        self.share_leader_push_thread.len() as u64
    }

    pub fn add_share_leader_push_thread(&self, share_leader_key: String, data: SubPushThreadData) {
        self.share_leader_push_thread.insert(share_leader_key, data);
    }

    pub fn get_share_leader_push_thread(
        &self,
        share_leader_key: &str,
    ) -> Option<SubPushThreadData> {
        if let Some(data) = self.share_leader_push_thread.get(share_leader_key) {
            return Some(data.clone());
        }
        None
    }

    pub fn remove_share_leader_push_thread(&self, share_leader_key: &str) {
        self.share_leader_push_thread.remove(share_leader_key);
    }

    pub fn contain_share_leader_push_thread(&self, share_leader_key: &str) -> bool {
        self.share_leader_push_thread.contains_key(share_leader_key)
    }

    pub fn update_subscribe_leader_push_thread_info(
        &self,
        key: &str,
        success_record_num: u64,
        error_record_num: u64,
    ) {
        if let Some(mut thread) = self.share_leader_push_thread.get_mut(key) {
            thread.last_run_time = now_second();

            if (success_record_num + error_record_num) > 0 {
                thread.push_success_record_num += success_record_num;
                thread.push_error_record_num += error_record_num;
                thread.last_push_time = now_second();
            }
        }
    }

    // share_follower_resub
    pub fn share_follower_resub_list(&self) -> DashMap<String, ShareSubShareSub> {
        self.share_follower_resub.clone()
    }

    pub fn share_follower_resub_len(&self) -> u64 {
        self.share_follower_resub.len() as u64
    }

    pub fn remove_share_follower_resub(&self, follower_resub_key: &str) {
        self.share_follower_resub.remove(follower_resub_key);
    }

    pub fn contain_share_follower_resub(&self, follower_resub_key: &str) -> bool {
        self.share_follower_resub.contains_key(follower_resub_key)
    }

    pub fn add_share_subscribe_follower(
        &self,
        client_id: &str,
        group_name: &str,
        topic_name: &str,
        share_sub: ShareSubShareSub,
    ) {
        let key = self.share_follower_key(client_id, group_name, topic_name);
        self.share_follower_resub.insert(key, share_sub);
    }

    fn remove_share_subscribe_follower_by_client_id(&self, client_id: &str) {
        for (key, share_sub) in self.share_follower_resub.clone() {
            if share_sub.client_id == *client_id {
                self.share_follower_resub.remove(&key);
            }
        }
    }

    // share_follower_resub_thread
    pub fn share_follower_resub_thread_list(&self) -> DashMap<String, SubPushThreadData> {
        self.share_follower_resub_thread.clone()
    }

    pub fn share_follower_resub_thread_len(&self) -> u64 {
        self.share_follower_resub_thread.len() as u64
    }

    pub fn contain_share_follower_resub_thread(&self, follower_resub_key: &str) -> bool {
        self.share_follower_resub_thread
            .contains_key(follower_resub_key)
    }

    pub fn remove_share_follower_resub_thread(&self, follower_resub_key: &str) {
        self.share_follower_resub_thread.remove(follower_resub_key);
    }

    pub fn get_share_follower_resub_thread(
        &self,
        follower_resub_key: &str,
    ) -> Option<SubPushThreadData> {
        if let Some(data) = self.share_follower_resub_thread.get(follower_resub_key) {
            return Some(data.clone());
        }
        None
    }

    pub fn add_share_follower_resub_thread(
        &self,
        follower_resub_key: String,
        data: SubPushThreadData,
    ) {
        self.share_follower_resub_thread
            .insert(follower_resub_key, data);
    }

    pub fn update_subscribe_follower_push_thread_info(
        &self,
        key: &str,
        success_record_num: u64,
        error_record_num: u64,
    ) {
        if let Some(mut thread) = self.share_follower_resub_thread.get_mut(key) {
            thread.last_run_time = now_second();
            if (success_record_num + error_record_num) > 0 {
                thread.push_success_record_num += success_record_num;
                thread.push_error_record_num += error_record_num;
                thread.last_push_time = now_second();
            }
        }
    }

    // Not push client
    pub fn add_not_push_client(&self, client_id: &str) {
        self.not_push_client.insert(
            client_id.to_string(),
            TemporaryNotPushClient {
                client_id: client_id.to_string(),
                last_check_time: now_second(),
            },
        );
    }

    pub fn remove_not_push_client(&self, client_id: &str) {
        self.not_push_client.remove(client_id);
    }

    pub fn update_not_push_client(&self, client_id: &str) {
        if let Some(mut raw) = self.not_push_client.get_mut(client_id) {
            raw.last_check_time = now_second();
        }
    }

    pub fn contain_not_push_client(&self, client_id: &str) -> bool {
        self.not_push_client.contains_key(client_id)
    }

    // topic subscribe
    pub fn topic_subscribe_list(&self) -> DashMap<String, Vec<TopicSubscribeInfo>> {
        self.topic_subscribes.clone()
    }

    pub fn add_topic_subscribe(&self, topic_name: &str, client_id: &str, path: &str) {
        if let Some(mut list) = self.topic_subscribes.get_mut(topic_name) {
            list.push(TopicSubscribeInfo {
                client_id: client_id.to_owned(),
                path: path.to_owned(),
            });
        } else {
            self.topic_subscribes.insert(
                topic_name.to_owned(),
                vec![TopicSubscribeInfo {
                    client_id: client_id.to_owned(),
                    path: path.to_owned(),
                }],
            );
        }
    }

    pub fn get_topic_subscribe_list(&self, topic_name: &str) -> Vec<TopicSubscribeInfo> {
        if let Some(list) = self.topic_subscribes.get(topic_name) {
            list.clone()
        } else {
            Vec::new()
        }
    }

    pub fn contain_topic_subscribe(&self, topic_name: &str) -> bool {
        if let Some(list) = self.topic_subscribes.get(topic_name) {
            return !list.is_empty();
        }
        false
    }

    pub fn is_exclusive_subscribe(&self, topic_name: &str) -> bool {
        if let Some(list) = self.topic_subscribes.get(topic_name) {
            for raw in list.iter() {
                if raw.path.starts_with("$exclusive") {
                    return true;
                }
            }
        }
        false
    }

    fn remove_topic_subscribe_by_client_id(&self, topic_name: &str, client_id: &str) {
        if let Some(mut list) = self.topic_subscribes.get_mut(topic_name) {
            list.retain(|x| x.client_id != *client_id);
        }
    }

    pub fn remove_topic_subscribe_by_path(&self, topic_name: &str, client_id: &str, path: &str) {
        if let Some(mut list) = self.topic_subscribes.get_mut(topic_name) {
            list.retain(|x| x.path != *path && x.client_id != *client_id);
        }
    }

    // subscribe_topics
    pub fn get_subscribe_topics_by_client_id_path(
        &self,
        client_id: &str,
        path: &str,
    ) -> Vec<String> {
        if let Some(list) = self.subscribe_topics.get(client_id) {
            if let Some(res) = list.get(path) {
                return res.clone();
            }
        }
        Vec::new()
    }

    pub fn add_subscribe_topics(&self, client_id: &str, path: &str, topic_name: &str) {
        if let Some(list) = self.subscribe_topics.get_mut(client_id) {
            if let Some(mut data) = list.get_mut(path) {
                if !data.contains(&topic_name.to_string()) {
                    data.push(topic_name.to_string());
                }
            } else {
                list.insert(path.to_string(), vec![topic_name.to_string()]);
            }
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(path.to_string(), vec![topic_name.to_string()]);
            self.subscribe_topics.insert(client_id.to_string(), data);
        }
    }

    pub fn remove_subscribe_topics_by_client_id(&self, client_id: &str) {
        self.subscribe_topics.remove(client_id);
    }

    pub fn remove_subscribe_topics_by_client_id_path(&self, client_id: &str, path: &str) {
        if let Some(list) = self.subscribe_topics.get_mut(client_id) {
            list.remove(path);
        }
    }

    // remove
    pub fn remove_client_id(&self, client_id: &str) {
        self.remove_exclusive_push_by_client_id(client_id);
        self.remove_share_subscribe_leader_by_client_id(client_id);
        self.remove_share_subscribe_follower_by_client_id(client_id);
        self.remove_subscriber_by_client_id(client_id);
        self.remove_not_push_client(client_id);
    }

    pub fn remove_topic(&self, _topic_name: &str) {
        // todo
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
    pub fn subscribe_key(&self, client_id: &str, path: &str) -> String {
        format!("{client_id}_{path}")
    }

    pub fn exclusive_key(&self, client_id: &str, sub_path: &str, topic_name: &str) -> String {
        format!("{client_id}_{sub_path}_{topic_name}")
    }

    pub fn share_leader_key(&self, group_name: &str, sub_path: &str, topic_name: &str) -> String {
        format!("{group_name}_{sub_path}_{topic_name}")
    }
    pub fn share_follower_key(
        &self,
        client_id: &str,
        group_name: &str,
        topic_name: &str,
    ) -> String {
        format!("{client_id}_{group_name}_{topic_name}")
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
        assert_eq!(subscribe_manager.topic_subscribes.len(), 1);
        assert!(subscribe_manager.contain_topic_subscribe(topic_name));
        assert!(!subscribe_manager.is_exclusive_subscribe(topic_name));
        subscribe_manager.remove_topic_subscribe_by_client_id(topic_name, &client_id);
        assert!(!subscribe_manager.contain_topic_subscribe(topic_name));
        assert!(!subscribe_manager.is_exclusive_subscribe(topic_name));

        let path = "$exclusive/path";
        subscribe_manager.add_topic_subscribe(topic_name, &client_id, path);
        assert_eq!(subscribe_manager.topic_subscribes.len(), 1);
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
            sub_path: "s1".to_string(),
            subscription_identifier: None,
            topic_name: "tname".to_string(),
        };
        let subscribe_manager = Arc::new(SubscribeManager::new());
        subscribe_manager.add_share_subscribe_follower(
            &share_sub.client_id,
            &share_sub.group_name,
            &share_sub.topic_name,
            share_sub.clone(),
        );
        subscribe_manager.remove_share_subscribe_follower_by_client_id(&share_sub.client_id);
        assert_eq!(subscribe_manager.share_follower_resub.len(), 0);
    }
}
