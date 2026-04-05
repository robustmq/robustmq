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

use crate::subscribe::buckets::NatsBucketsManager;
use crate::subscribe::common::NatsSubscriber;
use crate::subscribe::parse::ParseSubscribeData;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::nats::subscribe::NatsSubscribe;
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::error;

pub mod buckets;
pub mod common;
pub mod fanout_push;
pub mod parse;
pub mod queue_push;

#[derive(Default)]
pub struct NatsSubscribeManager {
    pub subscribe_list: DashMap<String, NatsSubscribe>,
    pub fanout_push: NatsBucketsManager,
    pub queue_push: DashMap<String, NatsBucketsManager>,
    pub not_push_client: DashMap<String, u64>,

    parse_sender: Arc<RwLock<Option<Sender<ParseSubscribeData>>>>,
}

impl NatsSubscribeManager {
    pub fn new() -> Self {
        NatsSubscribeManager {
            subscribe_list: DashMap::with_capacity(256),
            fanout_push: NatsBucketsManager::new(None),
            queue_push: DashMap::with_capacity(16),
            not_push_client: DashMap::with_capacity(32),
            parse_sender: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_parse_sender(&self, sender: Sender<ParseSubscribeData>) {
        let mut w = self.parse_sender.write().await;
        *w = Some(sender);
    }

    pub async fn send_parse_event(&self, data: ParseSubscribeData) {
        let r = self.parse_sender.read().await;
        if let Some(sender) = r.as_ref() {
            if let Err(e) = sender.send(data).await {
                error!("Failed to send parse event: {}", e);
            }
        }
    }

    pub fn add_subscribe(&self, subscribe: NatsSubscribe) {
        let key = subscribe_key(subscribe.connect_id, &subscribe.sid);
        self.subscribe_list.insert(key, subscribe);
    }

    pub fn remove_subscribe(&self, connect_id: u64, sid: &str) {
        self.subscribe_list.remove(&subscribe_key(connect_id, sid));
    }

    pub fn get_subscribe(&self, connect_id: u64, sid: &str) -> Option<NatsSubscribe> {
        self.subscribe_list
            .get(&subscribe_key(connect_id, sid))
            .map(|e| e.value().clone())
    }

    pub fn list_subscribes_by_connection(&self, connect_id: u64) -> Vec<NatsSubscribe> {
        let prefix = format!("{}#", connect_id);
        self.subscribe_list
            .iter()
            .filter(|e| e.key().starts_with(&prefix))
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn subscribe_count(&self) -> usize {
        self.subscribe_list.len()
    }

    pub fn add_fanout_subscriber(&self, subscriber: NatsSubscriber) {
        self.fanout_push.add(&subscriber);
    }

    pub fn add_queue_subscriber(&self, subscriber: NatsSubscriber, queue_group: &str) {
        let queue_key = format!("{}#{}", subscriber.topic_name, queue_group);
        self.queue_push
            .entry(queue_key.clone())
            .or_insert_with(|| NatsBucketsManager::new(Some(queue_key.clone())))
            .add(&subscriber);
    }

    pub fn remove_by_connection(&self, connect_id: u64) {
        self.subscribe_list
            .retain(|_, s| s.connect_id != connect_id);

        self.fanout_push.remove_by_connect_id(connect_id);

        for entry in self.queue_push.iter() {
            entry.value().remove_by_connect_id(connect_id);
        }

        let prefix = format!("{}#", connect_id);
        self.not_push_client.retain(|k, _| !k.starts_with(&prefix));
    }

    pub fn remove_by_topic(&self, topic_name: &str) {
        self.fanout_push.remove_by_topic(topic_name);
        let prefix = format!("{}#", topic_name);
        self.queue_push.retain(|key, _| !key.starts_with(&prefix));
    }

    pub fn remove_push_by_sid(&self, connect_id: u64, sid: &str) {
        self.fanout_push.remove_by_sid(connect_id, sid);
        for entry in self.queue_push.iter() {
            entry.value().remove_by_sid(connect_id, sid);
        }
    }

    pub fn add_not_push_client(&self, connect_id: u64, topic_name: &str) {
        let key = format!("{}#{}", connect_id, topic_name);
        self.not_push_client.insert(key, now_second());
    }

    pub fn allow_push_client(&self, connect_id: u64, topic_name: &str) -> bool {
        let key = format!("{}#{}", connect_id, topic_name);
        if let Some(entry) = self.not_push_client.get(&key) {
            if now_second().saturating_sub(*entry) < 10 {
                return false;
            }
            drop(entry);
            self.not_push_client.remove(&key);
        }
        true
    }
}

pub fn subscribe_key(connect_id: u64, sid: &str) -> String {
    format!("{}#{}", connect_id, sid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscribe::common::fanout_group_name;
    use common_base::tools::now_second;
    use metadata_struct::nats::subscribe::NatsSubscribe;

    fn make_subscribe(connect_id: u64, sid: &str, subject: &str) -> NatsSubscribe {
        NatsSubscribe {
            tenant: "default".to_string(),
            connect_id,
            sid: sid.to_string(),
            subject: subject.to_string(),
            queue_group: String::new(),
            create_time: 0,
        }
    }

    fn make_subscriber(connect_id: u64, sid: &str, topic: &str) -> NatsSubscriber {
        NatsSubscriber {
            connect_id,
            sid: sid.to_string(),
            sub_subject: topic.to_string(),
            topic_name: topic.to_string(),
            queue_group: String::new(),
            group_name: fanout_group_name(connect_id, sid, topic),
            create_time: now_second(),
        }
    }

    #[test]
    fn test_add_and_remove_subscribe() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_subscribe(make_subscribe(1, "s1", "foo.bar"));
        assert_eq!(mgr.subscribe_count(), 1);
        assert!(mgr.get_subscribe(1, "s1").is_some());

        mgr.remove_subscribe(1, "s1");
        assert_eq!(mgr.subscribe_count(), 0);
    }

    #[test]
    fn test_list_subscribes_by_connection() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_subscribe(make_subscribe(1, "s1", "foo"));
        mgr.add_subscribe(make_subscribe(1, "s2", "bar"));
        mgr.add_subscribe(make_subscribe(2, "s1", "baz"));

        assert_eq!(mgr.list_subscribes_by_connection(1).len(), 2);
        assert_eq!(mgr.list_subscribes_by_connection(2).len(), 1);
    }

    #[test]
    fn test_directly_subscriber() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_fanout_subscriber(make_subscriber(1, "s1", "foo.bar"));
        assert_eq!(mgr.fanout_push.sub_len(), 1);
    }

    #[test]
    fn test_remove_by_connection() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_subscribe(make_subscribe(1, "s1", "foo"));
        mgr.add_fanout_subscriber(make_subscriber(1, "s1", "foo"));
        mgr.add_fanout_subscriber(make_subscriber(1, "s2", "bar"));
        mgr.add_subscribe(make_subscribe(2, "s1", "baz"));

        mgr.remove_by_connection(1);

        assert_eq!(mgr.subscribe_count(), 1);
        assert_eq!(mgr.fanout_push.sub_len(), 0);
    }

    #[test]
    fn test_remove_push_by_sid() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_fanout_subscriber(make_subscriber(1, "s1", "foo"));
        mgr.add_fanout_subscriber(make_subscriber(1, "s1", "bar")); // wildcard match, same sid
        mgr.add_fanout_subscriber(make_subscriber(1, "s2", "baz"));

        mgr.remove_push_by_sid(1, "s1");

        assert_eq!(mgr.fanout_push.sub_len(), 1); // only s2/baz remains
    }

    #[test]
    fn test_not_push_client() {
        let mgr = NatsSubscribeManager::new();
        assert!(mgr.allow_push_client(1, "foo"));
        mgr.add_not_push_client(1, "foo");
        assert!(!mgr.allow_push_client(1, "foo"));
    }

    #[test]
    fn test_queue_subscriber() {
        let mgr = NatsSubscribeManager::new();
        let mut sub1 = make_subscriber(1, "s1", "orders");
        sub1.queue_group = "workers".to_string();
        let mut sub2 = make_subscriber(2, "s2", "orders");
        sub2.queue_group = "workers".to_string();

        mgr.add_queue_subscriber(sub1, "workers");
        mgr.add_queue_subscriber(sub2, "workers");

        let key = "orders#workers";
        assert!(mgr.queue_push.contains_key(key));
        assert_eq!(mgr.queue_push.get(key).unwrap().sub_len(), 2);
    }
}
