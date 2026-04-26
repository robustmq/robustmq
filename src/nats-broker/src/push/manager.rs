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

use crate::push::buckets::NatsBucketsManager;
use crate::push::parse::ParseSubscribeData;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::nats::{subscribe::NatsSubscribe, subscriber::NatsSubscriber};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc::Sender, RwLock};
use tracing::error;

pub struct QueuePushThreadInfo {
    pub stop_tx: broadcast::Sender<bool>,
    pub total_pushed: Mutex<u64>,
    pub last_pull_time: Mutex<u64>,
}

impl QueuePushThreadInfo {
    pub fn new(stop_tx: broadcast::Sender<bool>) -> Self {
        QueuePushThreadInfo {
            stop_tx,
            total_pushed: Mutex::new(0),
            last_pull_time: Mutex::new(0),
        }
    }

    pub fn record_push(&self, count: u64) {
        *self.total_pushed.lock().unwrap() += count;
        *self.last_pull_time.lock().unwrap() = now_second();
    }
}

#[derive(Default)]
pub struct NatsSubscribeManager {
    pub subscribe_list: DashMap<String, NatsSubscribe>,

    /// NATS core fanout push buckets (subject-based, wildcards).
    pub nats_core_fanout_push: NatsBucketsManager,
    /// NATS core queue-group push buckets (key: `{tenant}#{queue_group}#{subject}`).
    pub nats_core_queue_push: DashMap<String, NatsBucketsManager>,
    /// Running queue-group push tasks; value is the per-task runtime info.
    pub nats_core_queue_push_thread: DashMap<String, Arc<QueuePushThreadInfo>>,

    /// MQ9 fanout push buckets (mail_address-based).
    pub mq9_fanout_push: NatsBucketsManager,
    /// MQ9 queue-group push buckets (key: `{tenant}#{queue_group}#{subject}`).
    pub mq9_queue_push: DashMap<String, NatsBucketsManager>,

    pub not_push_client: DashMap<u64, u64>,
    parse_sender: Arc<RwLock<Option<Sender<ParseSubscribeData>>>>,
}

impl NatsSubscribeManager {
    pub fn new() -> Self {
        NatsSubscribeManager {
            subscribe_list: DashMap::with_capacity(256),
            nats_core_fanout_push: NatsBucketsManager::new(),
            nats_core_queue_push: DashMap::with_capacity(16),
            nats_core_queue_push_thread: DashMap::with_capacity(16),
            mq9_fanout_push: NatsBucketsManager::new(),
            mq9_queue_push: DashMap::with_capacity(16),
            not_push_client: DashMap::with_capacity(32),
            parse_sender: Arc::new(RwLock::new(None)),
        }
    }

    // parse event
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

    // subscribe
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

    // nats core add fanout/queue
    pub fn add_nats_core_fanout_subscriber(&self, subscriber: NatsSubscriber) {
        self.nats_core_fanout_push.add(&subscriber);
    }

    pub fn add_nats_core_queue_subscriber(&self, subscriber: &NatsSubscriber) {
        let queue_key = format!(
            "{}#{}#{}",
            subscriber.tenant,
            subscriber.queue_group.as_deref().unwrap_or(""),
            subscriber.subject
        );
        self.nats_core_queue_push
            .entry(queue_key)
            .or_default()
            .add(subscriber);
    }

    // mq9 add fanout/queue
    pub fn add_mq9_fanout_subscriber(&self, subscriber: NatsSubscriber) {
        self.mq9_fanout_push.add(&subscriber);
    }

    pub fn add_mq9_queue_subscriber(&self, subscriber: &NatsSubscriber) {
        let queue_key = format!(
            "{}#{}#{}",
            subscriber.tenant,
            subscriber.queue_group.as_deref().unwrap_or(""),
            subscriber.subject
        );
        self.mq9_queue_push
            .entry(queue_key)
            .or_default()
            .add(subscriber);
    }

    // remove
    pub fn remove_fanout_by_connection(&self, connect_id: u64) {
        // remove subscribe
        self.subscribe_list
            .retain(|_, s| s.connect_id != connect_id);

        // remove nats core fanout
        self.nats_core_fanout_push.remove_by_connect_id(connect_id);

        // remove mq9 fanout
        self.mq9_fanout_push.remove_by_connect_id(connect_id);

        // remove not push client
        self.not_push_client.remove(&connect_id);
    }

    pub fn remove_fanout_by_subject(&self, subject: &str) {
        // remove nats core fanout
        self.nats_core_fanout_push.remove_by_topic(subject);

        // remove mq9 fanout
        self.mq9_fanout_push.remove_by_topic(subject);
    }

    pub fn remove_push_by_sub(
        &self,
        broker_id: u64,
        connect_id: u64,
        sid: &str,
    ) -> Vec<NatsSubscriber> {
        self.nats_core_fanout_push
            .remove_by_sid(broker_id, connect_id, sid);
        self.mq9_fanout_push
            .remove_by_sid(broker_id, connect_id, sid);
        self.remove_queue_subscribers_by_sid(broker_id, connect_id, sid)
    }

    fn remove_queue_subscribers_by_sid(
        &self,
        broker_id: u64,
        connect_id: u64,
        sid: &str,
    ) -> Vec<NatsSubscriber> {
        let mut removed = Vec::new();

        for entry in self.nats_core_queue_push.iter() {
            removed.extend(entry.value().remove_by_sid(broker_id, connect_id, sid));
        }
        self.nats_core_queue_push.retain(|_, mgr| mgr.sub_len() > 0);

        for entry in self.mq9_queue_push.iter() {
            removed.extend(entry.value().remove_by_sid(broker_id, connect_id, sid));
        }
        self.mq9_queue_push.retain(|_, mgr| mgr.sub_len() > 0);
        removed
    }

    // not push client
    pub fn add_not_push_client(&self, connect_id: u64) {
        self.not_push_client.insert(connect_id, now_second());
    }

    pub fn allow_push_client(&self, connect_id: u64) -> bool {
        if let Some(entry) = self.not_push_client.get(&connect_id) {
            if now_second().saturating_sub(*entry) < 10 {
                return false;
            }
            drop(entry);
            self.not_push_client.remove(&connect_id);
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
    use common_base::{tools::now_second, uuid::unique_id};
    use metadata_struct::nats::subscribe::NatsSubscribe;

    fn make_subscribe(connect_id: u64, sid: &str, subject: &str) -> NatsSubscribe {
        NatsSubscribe {
            broker_id: 1,
            tenant: "default".to_string(),
            connect_id,
            sid: sid.to_string(),
            subject: subject.to_string(),
            queue_group: None,
            create_time: 0,
        }
    }

    fn make_subscriber(connect_id: u64, sid: &str, subject: &str) -> NatsSubscriber {
        NatsSubscriber {
            uniq_id: unique_id(),
            broker_id: 1,
            tenant: "default".to_string(),
            connect_id,
            sid: sid.to_string(),
            sub_subject: subject.to_string(),
            subject: subject.to_string(),
            queue_group: None,
            create_time: now_second(),
        }
    }

    fn make_queue_subscriber(
        connect_id: u64,
        sid: &str,
        subject: &str,
        group: &str,
    ) -> NatsSubscriber {
        let mut sub = make_subscriber(connect_id, sid, subject);
        sub.queue_group = Some(group.to_string());
        sub
    }

    #[test]
    fn test_subscribe_crud() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_subscribe(make_subscribe(1, "s1", "foo"));
        mgr.add_subscribe(make_subscribe(1, "s2", "bar"));
        mgr.add_subscribe(make_subscribe(2, "s1", "baz"));

        assert_eq!(mgr.list_subscribes_by_connection(1).len(), 2);
        assert!(mgr.get_subscribe(1, "s1").is_some());

        mgr.remove_subscribe(1, "s1");
        assert!(mgr.get_subscribe(1, "s1").is_none());
        assert_eq!(mgr.subscribe_count(), 2);
    }

    #[test]
    fn test_fanout_remove_by_connection_and_sid() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_nats_core_fanout_subscriber(make_subscriber(1, "s1", "foo"));
        mgr.add_mq9_fanout_subscriber(make_subscriber(1, "s1", "mailbox"));
        mgr.add_nats_core_fanout_subscriber(make_subscriber(1, "s2", "bar"));

        // remove s1 only
        mgr.remove_push_by_sub(1, 1, "s1");
        assert_eq!(mgr.nats_core_fanout_push.sub_len(), 1);
        assert_eq!(mgr.mq9_fanout_push.sub_len(), 0);

        // remove remaining by connection
        mgr.remove_fanout_by_connection(1);
        assert_eq!(mgr.nats_core_fanout_push.sub_len(), 0);
    }

    #[test]
    fn test_queue_subscriber_grouped_by_key() {
        let mgr = NatsSubscribeManager::new();
        mgr.add_nats_core_queue_subscriber(&make_queue_subscriber(1, "s1", "orders", "workers"));
        mgr.add_nats_core_queue_subscriber(&make_queue_subscriber(2, "s2", "orders", "workers"));
        mgr.add_mq9_queue_subscriber(&make_queue_subscriber(3, "s3", "events", "processors"));

        let key = "default#workers#orders";
        assert_eq!(mgr.nats_core_queue_push.get(key).unwrap().sub_len(), 2);
        assert!(mgr.mq9_queue_push.contains_key("default#processors#events"));
    }

    #[test]
    fn test_not_push_client() {
        let mgr = NatsSubscribeManager::new();
        assert!(mgr.allow_push_client(1));
        mgr.add_not_push_client(1);
        assert!(!mgr.allow_push_client(1));
    }
}
