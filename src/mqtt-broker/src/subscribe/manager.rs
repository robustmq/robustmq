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

use crate::{
    handler::sub_exclusive::is_exclusive_sub,
    subscribe::{buckets::BucketsManager, common::Subscriber, parse::ParseSubscribeData},
};
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::error;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct TopicSubscribeInfo {
    pub client_id: String,
    pub path: String,
}

#[derive(Clone, Default)]
pub struct SubscribeManager {
    //(client_id_path: MqttSubscribe)
    pub subscribe_list: DashMap<String, MqttSubscribe>,

    // directly sub
    pub directly_push: BucketsManager,

    // share sub
    // (group_name, BucketsManager)
    pub share_push: DashMap<String, BucketsManager>,
    // (group_name, Vec<TopicName>)
    pub share_group_topics: DashMap<String, HashSet<String>>,

    pub topic_subscribes: DashMap<String, HashSet<TopicSubscribeInfo>>,

    //(client_id, TemporaryNotPushClient)
    pub not_push_client: DashMap<String, u64>,

    pub update_cache_sender: Arc<RwLock<Option<Sender<ParseSubscribeData>>>>,
}

impl SubscribeManager {
    pub fn new() -> Self {
        SubscribeManager {
            subscribe_list: DashMap::with_capacity(128),
            topic_subscribes: DashMap::with_capacity(64),
            not_push_client: DashMap::with_capacity(32),
            directly_push: BucketsManager::new(10000),
            share_push: DashMap::with_capacity(8),
            share_group_topics: DashMap::with_capacity(8),
            update_cache_sender: Arc::new(RwLock::new(None)),
        }
    }

    // subscribe_list
    pub fn add_subscribe(&self, subscribe: &MqttSubscribe) {
        let key = self.subscribe_key(&subscribe.client_id, &subscribe.path);
        self.subscribe_list.insert(key, subscribe.clone());
    }

    pub fn get_subscribe(&self, client_id: &str, path: &str) -> Option<MqttSubscribe> {
        self.subscribe_list
            .get(&self.subscribe_key(client_id, path))
            .map(|da| da.clone())
    }

    // directly && share
    pub fn add_directly_sub(&self, topic: &str, subscriber: &Subscriber) {
        self.add_topic_subscribe(topic, &subscriber.client_id, &subscriber.sub_path);
        self.directly_push.add(subscriber);
    }

    pub fn add_share_sub(&self, topic: &str, subscriber: &Subscriber) {
        // topic_subscribes
        self.add_topic_subscribe(topic, &subscriber.client_id, &subscriber.sub_path);

        // share_push
        if let Some(bucket) = self.share_push.get(&subscriber.group_name) {
            bucket.add(subscriber);
        } else {
            let bucket = BucketsManager::new(10000);
            bucket.add(subscriber);
            self.share_push
                .insert(subscriber.group_name.to_string(), bucket);
        }

        // share_group_topics
        if let Some(mut list) = self.share_group_topics.get_mut(&subscriber.group_name) {
            list.insert(topic.to_string());
        } else {
            let mut set = HashSet::new();
            set.insert(topic.to_string());
            self.share_group_topics
                .insert(subscriber.group_name.to_string(), set);
        }
    }

    // remove
    pub fn remove_by_client_id(&self, client_id: &str) {
        self.subscribe_list
            .retain(|_, subscribe| subscribe.client_id != *client_id);

        // Clean up topic_subscribes and remove empty entries
        self.topic_subscribes.retain(|_, list| {
            list.retain(|x| x.client_id != *client_id);
            !list.is_empty()
        });

        self.not_push_client.remove(client_id);
        self.directly_push.remove_by_client_id(client_id);

        for row in self.share_push.iter() {
            row.remove_by_client_id(client_id);
        }
    }

    pub fn remove_by_sub(&self, client_id: &str, sub_path: &str) {
        let key = self.subscribe_key(client_id, sub_path);
        self.subscribe_list.remove(&key);

        // Clean up topic_subscribes and remove empty entries
        self.topic_subscribes.retain(|_, list| {
            list.retain(|x| !(x.path == *sub_path && x.client_id == *client_id));
            !list.is_empty()
        });

        self.directly_push.remove_by_sub(client_id, sub_path);

        for row in self.share_push.iter() {
            row.remove_by_sub(client_id, sub_path);
        }
    }

    pub fn remove_by_topic(&self, topic_name: &str) {
        self.topic_subscribes.remove(topic_name);

        self.directly_push.remove_by_topic(topic_name);

        for row in self.share_push.iter() {
            row.remove_by_topic(topic_name);
        }
    }

    // add parse data
    pub async fn set_cache_sender(&self, sender: Sender<ParseSubscribeData>) {
        let mut write = self.update_cache_sender.write().await;
        *write = Some(sender);
    }

    pub async fn add_wait_parse_data(&self, data: ParseSubscribeData) {
        let read = self.update_cache_sender.read().await;
        if let Some(sender) = read.clone() {
            if let Err(e) = sender.send(data).await {
                error!("{}", e);
            }
        }
    }

    // not push client
    pub fn add_not_push_client(&self, client_id: &str) {
        self.not_push_client
            .insert(client_id.to_string(), now_second());
    }

    pub fn update_not_push_client(&self, client_id: &str) {
        self.not_push_client
            .insert(client_id.to_string(), now_second());
    }

    // topic
    pub fn add_topic_subscribe(&self, topic_name: &str, client_id: &str, path: &str) {
        self.topic_subscribes
            .entry(topic_name.to_owned())
            .or_default()
            .insert(TopicSubscribeInfo {
                client_id: client_id.to_owned(),
                path: path.to_owned(),
            });
    }

    pub fn is_exclusive_subscribe(&self, topic_name: &str) -> bool {
        self.topic_subscribes
            .get(topic_name)
            .map(|list| list.iter().any(|raw| is_exclusive_sub(&raw.path)))
            .unwrap_or(false)
    }

    pub fn is_exclusive_subscribe_by_other(&self, topic_name: &str, client_id: &str) -> bool {
        self.topic_subscribes
            .get(topic_name)
            .map(|list| {
                list.iter()
                    .any(|raw| is_exclusive_sub(&raw.path) && raw.client_id != *client_id)
            })
            .unwrap_or(false)
    }

    fn subscribe_key(&self, client_id: &str, path: &str) -> String {
        format!("{client_id}#{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::mqtt::common::{Filter, MqttProtocol, QoS, RetainHandling};

    fn create_subscribe(client_id: &str, path: &str) -> MqttSubscribe {
        MqttSubscribe {
            client_id: client_id.to_string(),
            path: path.to_string(),
            cluster_name: "test_cluster".to_string(),
            broker_id: 1,
            protocol: MqttProtocol::Mqtt5,
            filter: Filter {
                path: path.to_string(),
                qos: QoS::AtLeastOnce,
                nolocal: true,
                preserve_retain: true,
                retain_handling: RetainHandling::Never,
            },
            pkid: 1,
            subscribe_properties: None,
            create_time: 0,
        }
    }

    #[test]
    fn test_subscribe_key_no_conflict() {
        let mgr = SubscribeManager::new();
        assert_ne!(
            mgr.subscribe_key("user_123", "topic"),
            mgr.subscribe_key("user", "123_topic")
        );
    }

    #[test]
    fn test_add_and_get_subscribe() {
        let mgr = SubscribeManager::new();
        let sub = create_subscribe("c1", "/t1");

        mgr.add_subscribe(&sub);

        assert!(mgr.get_subscribe("c1", "/t1").is_some());
        assert!(mgr.get_subscribe("c1", "/t2").is_none());
    }

    #[test]
    fn test_remove_by_sub_logic() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe("topic1", "c1", "/t1");
        mgr.add_topic_subscribe("topic1", "c1", "/t2");
        mgr.add_topic_subscribe("topic1", "c2", "/t1");

        assert_eq!(mgr.topic_subscribes.get("topic1").unwrap().len(), 3);

        mgr.remove_by_sub("c1", "/t1");

        let list = mgr.topic_subscribes.get("topic1").unwrap();
        assert_eq!(list.len(), 2);
        assert!(!list.iter().any(|x| x.client_id == "c1" && x.path == "/t1"));
        assert!(list.iter().any(|x| x.client_id == "c1" && x.path == "/t2"));
    }

    #[test]
    fn test_topic_subscribe_deduplication() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe("topic1", "c1", "/t1");
        mgr.add_topic_subscribe("topic1", "c1", "/t1");
        mgr.add_topic_subscribe("topic1", "c1", "/t1");

        assert_eq!(mgr.topic_subscribes.get("topic1").unwrap().len(), 1);
    }

    #[test]
    fn test_remove_by_client_id() {
        let mgr = SubscribeManager::new();
        let sub1 = create_subscribe("c1", "/t1");
        let sub2 = create_subscribe("c2", "/t2");

        mgr.add_subscribe(&sub1);
        mgr.add_subscribe(&sub2);
        mgr.add_topic_subscribe("topic1", "c1", "/t1");
        mgr.add_topic_subscribe("topic1", "c2", "/t2");

        mgr.remove_by_client_id("c1");

        assert!(mgr.get_subscribe("c1", "/t1").is_none());
        assert!(mgr.get_subscribe("c2", "/t2").is_some());

        let list = mgr.topic_subscribes.get("topic1").unwrap();
        assert!(!list.iter().any(|x| x.client_id == "c1"));
    }

    #[test]
    fn test_not_push_client() {
        let mgr = SubscribeManager::new();

        mgr.add_not_push_client("c1");
        assert!(mgr.not_push_client.get("c1").is_some());

        let initial_time = *mgr.not_push_client.get("c1").unwrap();

        mgr.update_not_push_client("c1");
        let updated_time = *mgr.not_push_client.get("c1").unwrap();

        assert!(updated_time >= initial_time);
    }

    #[test]
    fn test_is_exclusive_subscribe() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe("topic1", "c1", "/t1");
        assert!(!mgr.is_exclusive_subscribe("topic1"));

        mgr.add_topic_subscribe("topic2", "c2", "$exclusive/t2");
        assert!(mgr.is_exclusive_subscribe("topic2"));

        assert!(!mgr.is_exclusive_subscribe("topic_not_exist"));
    }

    #[test]
    fn test_is_exclusive_subscribe_by_other() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe("topic1", "c1", "$exclusive/t1");

        // Same client should return false
        assert!(!mgr.is_exclusive_subscribe_by_other("topic1", "c1"));

        // Different client should return true
        assert!(mgr.is_exclusive_subscribe_by_other("topic1", "c2"));

        // Non-existent topic should return false
        assert!(!mgr.is_exclusive_subscribe_by_other("topic_not_exist", "c1"));
    }
}
