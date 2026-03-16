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
    core::sub_exclusive::is_exclusive_sub,
    subscribe::{buckets::BucketsManager, common::Subscriber, parse::ParseSubscribeData},
};
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::mqtt::subscribe::MqttSubscribe;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::error;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct TopicSubscribeInfo {
    pub client_id: String,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ShareSubscribeTopicInfo {
    pub topic: String,
}

#[derive(Clone, Default)]
pub struct SubscribeManager {
    // (tenant, (client_id#path, MqttSubscribe))
    pub subscribe_list: DashMap<String, DashMap<String, MqttSubscribe>>,

    // directly sub
    pub directly_push: BucketsManager,

    // share sub
    // (tenant, (group_name, BucketsManager))
    pub share_push: DashMap<String, DashMap<String, BucketsManager>>,
    // (tenant, (group_name, HashSet<ShareSubscribeTopicInfo>))
    pub share_group_topics: DashMap<String, DashMap<String, HashSet<ShareSubscribeTopicInfo>>>,

    // (tenant, (topic, HashSet<TopicSubscribeInfo>))
    pub topic_subscribes: DashMap<String, DashMap<String, HashSet<TopicSubscribeInfo>>>,

    // (tenant, (client_id, last_not_push_time))
    pub not_push_client: DashMap<String, DashMap<String, u64>>,

    pub update_cache_sender: Arc<RwLock<Option<Sender<ParseSubscribeData>>>>,
}

impl SubscribeManager {
    pub fn new() -> Self {
        SubscribeManager {
            subscribe_list: DashMap::with_capacity(128),
            topic_subscribes: DashMap::with_capacity(64),
            not_push_client: DashMap::with_capacity(32),
            directly_push: BucketsManager::new(None, 10000),
            share_push: DashMap::with_capacity(8),
            share_group_topics: DashMap::with_capacity(8),
            update_cache_sender: Arc::new(RwLock::new(None)),
        }
    }

    // subscribe_list
    pub fn add_subscribe(&self, subscribe: &MqttSubscribe) {
        let key = self.subscribe_key(&subscribe.client_id, &subscribe.path);
        self.subscribe_list
            .entry(subscribe.tenant.clone())
            .or_default()
            .insert(key, subscribe.clone());
    }

    pub fn get_subscribe(
        &self,
        tenant: &str,
        client_id: &str,
        path: &str,
    ) -> Option<MqttSubscribe> {
        self.subscribe_list.get(tenant).and_then(|m| {
            m.get(&self.subscribe_key(client_id, path))
                .map(|v| v.clone())
        })
    }

    pub fn subscribe_count(&self) -> usize {
        self.subscribe_list.iter().map(|e| e.value().len()).sum()
    }

    // directly && share
    pub fn add_directly_sub(&self, subscriber: &Subscriber) {
        self.add_topic_subscribe(
            &subscriber.tenant,
            &subscriber.topic_name,
            &subscriber.client_id,
            &subscriber.sub_path,
        );
        self.directly_push.add(subscriber);
    }

    pub fn add_share_sub(&self, subscriber: &Subscriber) {
        // topic_subscribes
        self.add_topic_subscribe(
            &subscriber.tenant,
            &subscriber.topic_name,
            &subscriber.client_id,
            &subscriber.sub_path,
        );

        // share_push
        self.share_push
            .entry(subscriber.tenant.clone())
            .or_default()
            .entry(subscriber.group_name.clone())
            .or_insert_with(|| BucketsManager::new(Some(subscriber.group_name.clone()), 10000))
            .add(subscriber);

        // share_group_topics
        self.share_group_topics
            .entry(subscriber.tenant.clone())
            .or_default()
            .entry(subscriber.group_name.clone())
            .or_default()
            .insert(ShareSubscribeTopicInfo {
                topic: subscriber.topic_name.clone(),
            });
    }

    // remove
    pub fn remove_by_client_id(&self, tenant: &str, client_id: &str) {
        if let Some(tenant_map) = self.subscribe_list.get(tenant) {
            tenant_map.retain(|_, subscribe| subscribe.client_id != *client_id);
        }
        self.subscribe_list.retain(|_, m| !m.is_empty());

        // Clean up topic_subscribes
        if let Some(tenant_topics) = self.topic_subscribes.get(tenant) {
            tenant_topics.retain(|_, list| {
                list.retain(|x| x.client_id != *client_id);
                !list.is_empty()
            });
        }

        if let Some(tenant_map) = self.not_push_client.get(tenant) {
            tenant_map.remove(client_id);
        }
        self.directly_push.remove_by_client_id(client_id);

        if let Some(tenant_share) = self.share_push.get(tenant) {
            for row in tenant_share.iter() {
                row.remove_by_client_id(client_id);
            }
        }
    }

    pub fn remove_by_sub(&self, tenant: &str, client_id: &str, sub_path: &str) {
        let key = self.subscribe_key(client_id, sub_path);
        if let Some(tenant_map) = self.subscribe_list.get(tenant) {
            tenant_map.remove(&key);
        }

        // Clean up topic_subscribes
        if let Some(tenant_topics) = self.topic_subscribes.get(tenant) {
            tenant_topics.retain(|_, list| {
                list.retain(|x| !(x.path == *sub_path && x.client_id == *client_id));
                !list.is_empty()
            });
        }

        self.directly_push.remove_by_sub(client_id, sub_path);

        if let Some(tenant_share) = self.share_push.get(tenant) {
            for row in tenant_share.iter() {
                row.remove_by_sub(client_id, sub_path);
            }
        }
    }

    pub fn remove_by_topic(&self, tenant: &str, topic_name: &str) {
        if let Some(tenant_topics) = self.topic_subscribes.get(tenant) {
            tenant_topics.remove(topic_name);
        }

        self.directly_push.remove_by_topic(topic_name);

        if let Some(tenant_share) = self.share_push.get(tenant) {
            for row in tenant_share.iter() {
                row.remove_by_topic(topic_name);
            }
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
    pub fn add_not_push_client(&self, tenant: &str, client_id: &str) {
        self.not_push_client
            .entry(tenant.to_string())
            .or_default()
            .insert(client_id.to_string(), now_second());
    }

    pub fn allow_push_client(&self, tenant: &str, client_id: &str) -> bool {
        if let Some(tenant_map) = self.not_push_client.get(tenant) {
            if let Some(time) = tenant_map.get(client_id) {
                if (now_second() - *time) >= 10 {
                    drop(time);
                    tenant_map.remove(client_id);
                    return true;
                } else {
                    return false;
                }
            }
        }
        true
    }

    // topic
    pub fn add_topic_subscribe(&self, tenant: &str, topic_name: &str, client_id: &str, path: &str) {
        self.topic_subscribes
            .entry(tenant.to_owned())
            .or_default()
            .entry(topic_name.to_owned())
            .or_default()
            .insert(TopicSubscribeInfo {
                client_id: client_id.to_owned(),
                path: path.to_owned(),
            });
    }

    pub fn is_exclusive_subscribe(&self, tenant: &str, topic_name: &str) -> bool {
        let Some(tenant_map) = self.topic_subscribes.get(tenant) else {
            return false;
        };
        tenant_map
            .get(topic_name)
            .map(|list| list.iter().any(|raw| is_exclusive_sub(&raw.path)))
            .unwrap_or(false)
    }

    pub fn is_exclusive_subscribe_by_other(
        &self,
        tenant: &str,
        topic_name: &str,
        client_id: &str,
    ) -> bool {
        let Some(tenant_map) = self.topic_subscribes.get(tenant) else {
            return false;
        };
        tenant_map
            .get(topic_name)
            .map(|list| {
                list.iter()
                    .any(|raw| is_exclusive_sub(&raw.path) && raw.client_id != *client_id)
            })
            .unwrap_or(false)
    }

    pub fn share_group_count(&self) -> usize {
        self.share_push.iter().map(|t| t.value().len()).sum()
    }

    pub fn share_sub_len(&self) -> u64 {
        let mut len = 0;
        for tenant_entry in self.share_push.iter() {
            for raw in tenant_entry.value().iter() {
                len += raw.value().sub_len();
            }
        }
        len
    }

    fn subscribe_key(&self, client_id: &str, path: &str) -> String {
        format!("{client_id}#{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use protocol::mqtt::common::{Filter, MqttProtocol, QoS, RetainHandling};

    fn create_subscribe(client_id: &str, path: &str) -> MqttSubscribe {
        MqttSubscribe {
            tenant: DEFAULT_TENANT.to_string(),
            client_id: client_id.to_string(),
            path: path.to_string(),
            broker_id: 1,
            protocol: MqttProtocol::Mqtt5,
            filter: Filter {
                path: path.to_string(),
                qos: QoS::AtLeastOnce,
                no_local: true,
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

        assert!(mgr.get_subscribe(DEFAULT_TENANT, "c1", "/t1").is_some());
        assert!(mgr.get_subscribe(DEFAULT_TENANT, "c1", "/t2").is_none());
    }

    #[test]
    fn test_remove_by_sub_logic() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "/t1");
        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "/t2");
        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c2", "/t1");

        assert_eq!(
            mgr.topic_subscribes
                .get(DEFAULT_TENANT)
                .unwrap()
                .get("topic1")
                .unwrap()
                .len(),
            3
        );

        mgr.remove_by_sub(DEFAULT_TENANT, "c1", "/t1");

        let tenant_map = mgr.topic_subscribes.get(DEFAULT_TENANT).unwrap();
        let list = tenant_map.get("topic1").unwrap();
        assert_eq!(list.len(), 2);
        assert!(!list.iter().any(|x| x.client_id == "c1" && x.path == "/t1"));
        assert!(list.iter().any(|x| x.client_id == "c1" && x.path == "/t2"));
    }

    #[test]
    fn test_topic_subscribe_deduplication() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "/t1");
        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "/t1");
        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "/t1");

        assert_eq!(
            mgr.topic_subscribes
                .get(DEFAULT_TENANT)
                .unwrap()
                .get("topic1")
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_remove_by_client_id() {
        let mgr = SubscribeManager::new();
        let sub1 = create_subscribe("c1", "/t1");
        let sub2 = create_subscribe("c2", "/t2");

        mgr.add_subscribe(&sub1);
        mgr.add_subscribe(&sub2);
        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "/t1");
        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c2", "/t2");

        mgr.remove_by_client_id(DEFAULT_TENANT, "c1");

        assert!(mgr.get_subscribe(DEFAULT_TENANT, "c1", "/t1").is_none());
        assert!(mgr.get_subscribe(DEFAULT_TENANT, "c2", "/t2").is_some());

        let tenant_map = mgr.topic_subscribes.get(DEFAULT_TENANT).unwrap();
        let list = tenant_map.get("topic1").unwrap();
        assert!(!list.iter().any(|x| x.client_id == "c1"));
    }

    #[test]
    fn test_is_exclusive_subscribe() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "/t1");
        assert!(!mgr.is_exclusive_subscribe(DEFAULT_TENANT, "topic1"));

        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic2", "c2", "$exclusive/t2");
        assert!(mgr.is_exclusive_subscribe(DEFAULT_TENANT, "topic2"));

        assert!(!mgr.is_exclusive_subscribe(DEFAULT_TENANT, "topic_not_exist"));
    }

    #[test]
    fn test_is_exclusive_subscribe_by_other() {
        let mgr = SubscribeManager::new();

        mgr.add_topic_subscribe(DEFAULT_TENANT, "topic1", "c1", "$exclusive/t1");

        // Same client should return false
        assert!(!mgr.is_exclusive_subscribe_by_other(DEFAULT_TENANT, "topic1", "c1"));

        // Different client should return true
        assert!(mgr.is_exclusive_subscribe_by_other(DEFAULT_TENANT, "topic1", "c2"));

        // Non-existent topic should return false
        assert!(!mgr.is_exclusive_subscribe_by_other(DEFAULT_TENANT, "topic_not_exist", "c1"));
    }
}
