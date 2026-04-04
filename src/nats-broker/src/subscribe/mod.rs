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

pub mod directly_push;
pub mod parse;
pub mod queue_push;

use crate::subscribe::parse::ParseSubscribeData;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::nats::subscribe::NatsSubscribe;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::error;

#[derive(Debug, Clone)]
pub struct NatsSubscriber {
    pub connect_id: u64,
    pub sid: String,
    pub sub_subject: String,
    pub topic_name: String,
    // format: nats_directly_{connect_id}_{sid}_{topic_name}
    pub group_name: String,
    pub create_time: u64,
}

#[derive(Debug)]
pub struct NatsQueueGroupSubscribers {
    // seq → NatsSubscriber
    pub subscribers: DashMap<u64, NatsSubscriber>,
    pub round_robin: Arc<AtomicU64>,
}

impl Default for NatsQueueGroupSubscribers {
    fn default() -> Self {
        NatsQueueGroupSubscribers {
            subscribers: DashMap::with_capacity(8),
            round_robin: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[derive(Default)]
pub struct NatsSubscribeManager {
    // key: {connect_id}#{sid}
    pub subscribe_list: DashMap<String, NatsSubscribe>,

    // topic_name → (seq → NatsSubscriber)
    pub directly_push: DashMap<String, DashMap<u64, NatsSubscriber>>,

    // {topic_name}#{queue_group} → NatsQueueGroupSubscribers
    pub queue_push: DashMap<String, NatsQueueGroupSubscribers>,

    // {connect_id}#{topic_name} → unix-second
    pub not_push_client: DashMap<String, u64>,

    connect_id_seqs: DashMap<u64, HashSet<u64>>,
    sub_key_seq: DashMap<String, u64>,
    seq_topic: DashMap<u64, String>,
    seq_queue_key: DashMap<u64, String>,
    seq_num: Arc<AtomicU64>,
    parse_sender: Arc<RwLock<Option<Sender<ParseSubscribeData>>>>,
}

impl NatsSubscribeManager {
    pub fn new() -> Self {
        NatsSubscribeManager {
            subscribe_list: DashMap::with_capacity(256),
            directly_push: DashMap::with_capacity(64),
            queue_push: DashMap::with_capacity(16),
            connect_id_seqs: DashMap::with_capacity(128),
            sub_key_seq: DashMap::with_capacity(256),
            seq_topic: DashMap::with_capacity(256),
            seq_queue_key: DashMap::with_capacity(256),
            not_push_client: DashMap::with_capacity(32),
            seq_num: Arc::new(AtomicU64::new(0)),
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
        let key = self.subscribe_key(subscribe.connect_id, &subscribe.sid);
        self.subscribe_list.insert(key, subscribe);
    }

    pub fn remove_subscribe(&self, connect_id: u64, sid: &str) {
        let key = self.subscribe_key(connect_id, sid);
        self.subscribe_list.remove(&key);
        self.remove_push_entry(connect_id, sid);
    }

    pub fn get_subscribe(&self, connect_id: u64, sid: &str) -> Option<NatsSubscribe> {
        self.subscribe_list
            .get(&self.subscribe_key(connect_id, sid))
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

    pub fn add_directly_subscriber(&self, subscriber: NatsSubscriber) {
        let seq = self.next_seq();
        let connect_id = subscriber.connect_id;
        let sid = subscriber.sid.clone();
        let topic_name = subscriber.topic_name.clone();

        self.directly_push
            .entry(topic_name.clone())
            .or_default()
            .insert(seq, subscriber);

        self.record_seq(connect_id, &sid, seq, &topic_name, "");
    }

    pub fn remove_directly_subscriber(&self, topic_name: &str, seq: u64) {
        if let Some(bucket) = self.directly_push.get(topic_name) {
            bucket.remove(&seq);
        }
    }

    pub fn add_queue_subscriber(&self, subscriber: NatsSubscriber, queue_group: &str) {
        let seq = self.next_seq();
        let connect_id = subscriber.connect_id;
        let sid = subscriber.sid.clone();
        let topic_name = subscriber.topic_name.clone();
        let queue_key = format!("{}#{}", topic_name, queue_group);

        self.queue_push
            .entry(queue_key.clone())
            .or_default()
            .subscribers
            .insert(seq, subscriber);

        self.record_seq(connect_id, &sid, seq, &topic_name, &queue_key);
    }

    pub fn remove_queue_subscriber(&self, queue_key: &str, seq: u64) {
        if let Some(group) = self.queue_push.get(queue_key) {
            group.subscribers.remove(&seq);
        }
    }

    pub fn remove_by_connection(&self, connect_id: u64) {
        self.subscribe_list
            .retain(|_, s| s.connect_id != connect_id);

        if let Some((_, seqs)) = self.connect_id_seqs.remove(&connect_id) {
            for seq in seqs {
                self.remove_push_by_seq(seq);
            }
        }

        let prefix = format!("{}#", connect_id);
        self.not_push_client.retain(|k, _| !k.starts_with(&prefix));
    }

    pub fn add_not_push_client(&self, connect_id: u64, topic_name: &str) {
        let key = format!("{}#{}", connect_id, topic_name);
        self.not_push_client.insert(key, now_second());
    }

    pub fn allow_push_client(&self, connect_id: u64, topic_name: &str) -> bool {
        let key = format!("{}#{}", connect_id, topic_name);
        if let Some(entry) = self.not_push_client.get(&key) {
            let elapsed = now_second().saturating_sub(*entry);
            if elapsed < 10 {
                return false;
            }
            drop(entry);
            self.not_push_client.remove(&key);
        }
        true
    }

    fn subscribe_key(&self, connect_id: u64, sid: &str) -> String {
        format!("{}#{}", connect_id, sid)
    }

    fn next_seq(&self) -> u64 {
        self.seq_num.fetch_add(1, Ordering::Relaxed)
    }

    fn record_seq(&self, connect_id: u64, sid: &str, seq: u64, topic_name: &str, queue_key: &str) {
        self.connect_id_seqs
            .entry(connect_id)
            .or_default()
            .insert(seq);

        let sub_key = self.subscribe_key(connect_id, sid);
        self.sub_key_seq.insert(sub_key, seq);
        self.seq_topic.insert(seq, topic_name.to_string());
        self.seq_queue_key.insert(seq, queue_key.to_string());
    }

    fn remove_push_entry(&self, connect_id: u64, sid: &str) {
        let sub_key = self.subscribe_key(connect_id, sid);
        if let Some((_, seq)) = self.sub_key_seq.remove(&sub_key) {
            if let Some(mut seqs) = self.connect_id_seqs.get_mut(&connect_id) {
                seqs.remove(&seq);
            }
            self.remove_push_by_seq(seq);
        }
    }

    fn remove_push_by_seq(&self, seq: u64) {
        let queue_key = self.seq_queue_key.remove(&seq).map(|(_, v)| v);
        let topic_name = self.seq_topic.remove(&seq).map(|(_, v)| v);

        match (topic_name, queue_key) {
            (Some(_), Some(qk)) if !qk.is_empty() => {
                self.remove_queue_subscriber(&qk, seq);
            }
            (Some(topic), _) => {
                self.remove_directly_subscriber(&topic, seq);
            }
            _ => {}
        }
    }
}

pub fn directly_group_name(connect_id: u64, sid: &str, topic_name: &str) -> String {
    format!("nats_directly_{}_{}_{}", connect_id, sid, topic_name)
}

#[cfg(test)]
mod tests {
    use super::*;
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
            group_name: directly_group_name(connect_id, sid, topic),
            create_time: now_second(),
        }
    }

    #[test]
    fn test_add_and_remove_subscribe() {
        let mgr = NatsSubscribeManager::new();
        let sub = make_subscribe(1, "s1", "foo.bar");
        mgr.add_subscribe(sub.clone());
        assert_eq!(mgr.subscribe_count(), 1);
        assert!(mgr.get_subscribe(1, "s1").is_some());

        mgr.remove_subscribe(1, "s1");
        assert_eq!(mgr.subscribe_count(), 0);
        assert!(mgr.get_subscribe(1, "s1").is_none());
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
        mgr.add_directly_subscriber(make_subscriber(1, "s1", "foo.bar"));

        assert!(mgr.directly_push.contains_key("foo.bar"));
        assert_eq!(mgr.directly_push.get("foo.bar").unwrap().len(), 1);
    }

    #[test]
    fn test_remove_by_connection() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_subscribe(make_subscribe(1, "s1", "foo"));
        mgr.add_subscribe(make_subscribe(1, "s2", "bar"));
        mgr.add_subscribe(make_subscribe(2, "s1", "baz"));
        mgr.add_directly_subscriber(make_subscriber(1, "s1", "foo"));
        mgr.add_directly_subscriber(make_subscriber(1, "s2", "bar"));

        mgr.remove_by_connection(1);

        assert_eq!(mgr.subscribe_count(), 1);
        assert!(mgr
            .directly_push
            .get("foo")
            .map(|b| b.is_empty())
            .unwrap_or(true));
        assert!(mgr
            .directly_push
            .get("bar")
            .map(|b| b.is_empty())
            .unwrap_or(true));
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
        mgr.add_queue_subscriber(make_subscriber(1, "s1", "orders"), "workers");
        mgr.add_queue_subscriber(make_subscriber(2, "s2", "orders"), "workers");

        let key = "orders#workers";
        assert!(mgr.queue_push.contains_key(key));
        assert_eq!(mgr.queue_push.get(key).unwrap().subscribers.len(), 2);
    }
}
