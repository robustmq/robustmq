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
use common_base::tools::unique_id;
use dashmap::DashMap;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

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
pub struct BucketsManager {
    // (bucket_id, (seq,subscriber))
    pub buckets_data_list: DashMap<String, DashMap<u64, Subscriber>>,

    // (client_id, (seq))
    client_id_sub: DashMap<String, HashSet<u64>>,
    // (client_id_sub_path, (seq))
    client_id_sub_path_sub: DashMap<String, HashSet<u64>>,
    // (topic, (seq))
    topic_sub: DashMap<String, HashSet<u64>>,

    bucket_size: u64,
    seq_num: Arc<AtomicU64>,
}

impl Default for BucketsManager {
    fn default() -> Self {
        Self::new(10000)
    }
}

impl BucketsManager {
    pub fn new(bucket_len: u64) -> Self {
        BucketsManager {
            bucket_size: bucket_len,
            seq_num: Arc::new(AtomicU64::new(0)),
            client_id_sub: DashMap::with_capacity(128),
            client_id_sub_path_sub: DashMap::with_capacity(128),
            buckets_data_list: DashMap::with_capacity(8),
            topic_sub: DashMap::with_capacity(8),
        }
    }

    pub fn add(&self, subscriber: &Subscriber) {
        let seq = self
            .seq_num
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if let Some(mut data) = self.client_id_sub.get_mut(&subscriber.client_id) {
            data.insert(seq);
        } else {
            let mut set = HashSet::new();
            set.insert(seq);
            self.client_id_sub.insert(subscriber.client_id.clone(), set);
        }

        let key = self.client_sub_path_key(&subscriber.client_id, &subscriber.sub_path);
        if let Some(mut data) = self.client_id_sub_path_sub.get_mut(&key) {
            data.insert(seq);
        } else {
            let mut set = HashSet::new();
            set.insert(seq);
            self.client_id_sub_path_sub.insert(key, set);
        }

        if let Some(mut data) = self.topic_sub.get_mut(&subscriber.topic_name) {
            data.insert(seq);
        } else {
            let mut set = HashSet::new();
            set.insert(seq);
            self.topic_sub
                .insert(subscriber.topic_name.clone(), set);
        }

        self.add_data_list(seq, subscriber);
    }

    pub fn remove_by_client_id(&self, client_id: &str) {
        let seqs: Vec<u64> = self
            .client_id_sub
            .get(client_id)
            .map(|data| data.iter().copied().collect())
            .unwrap_or_default();

        for seq in seqs {
            self.remove_data_list_by_seq(&seq);
        }
    }

    pub fn remove_by_sub(&self, client_id: &str, sub_path: &str) {
        let key = self.client_sub_path_key(client_id, sub_path);
        let seqs: Vec<u64> = self
            .client_id_sub_path_sub
            .get(&key)
            .map(|data| data.iter().copied().collect())
            .unwrap_or_default();

        for seq in seqs {
            self.remove_data_list_by_seq(&seq);
        }
    }

    pub fn remove_by_topic(&self, topic: &str) {
        let seqs: Vec<u64> = self
            .topic_sub
            .get(topic)
            .map(|data| data.iter().copied().collect())
            .unwrap_or_default();

        for seq in seqs {
            self.remove_data_list_by_seq(&seq);
        }
    }

    fn add_data_list(&self, seq: u64, subscriber: &Subscriber) {
        let mut write_success = false;
        for row in self.buckets_data_list.iter() {
            if row.len() as u64 >= self.bucket_size {
                continue;
            }
            row.insert(seq, subscriber.clone());
            write_success = true;
            break;
        }

        if !write_success {
            let data = DashMap::with_capacity(2);
            data.insert(seq, subscriber.clone());
            self.buckets_data_list.insert(unique_id(), data);
        }
    }

    fn remove_data_list_by_seq(&self, seq: &u64) {
        let mut empty_bucket_id: Option<String> = None;

        for row in self.buckets_data_list.iter() {
            if let Some((_, subscriber)) = row.remove(seq) {
                if let Some(mut data) = self.client_id_sub.get_mut(&subscriber.client_id) {
                    data.remove(seq);
                    if data.is_empty() {
                        drop(data);
                        self.client_id_sub.remove(&subscriber.client_id);
                    }
                }

                let key = self.client_sub_path_key(&subscriber.client_id, &subscriber.sub_path);
                if let Some(mut data) = self.client_id_sub_path_sub.get_mut(&key) {
                    data.remove(seq);
                    if data.is_empty() {
                        drop(data);
                        self.client_id_sub_path_sub.remove(&key);
                    }
                }

                if row.is_empty() {
                    empty_bucket_id = Some(row.key().clone());
                }

                break;
            }
        }

        if let Some(bucket_id) = empty_bucket_id {
            self.buckets_data_list.remove(&bucket_id);
        }
    }

    fn client_sub_path_key(&self, client_id: &str, sub_path: &str) -> String {
        format!("{client_id}_{sub_path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::mqtt::common::{MqttProtocol, QoS, RetainHandling};

    fn create_sub(client_id: &str, sub_path: &str) -> Subscriber {
        Subscriber {
            client_id: client_id.to_string(),
            sub_path: sub_path.to_string(),
            rewrite_sub_path: None,
            topic_name: "topic".to_string(),
            group_name: format!("group_{}", client_id),
            protocol: MqttProtocol::Mqtt5,
            qos: QoS::AtLeastOnce,
            no_local: false,
            preserve_retain: false,
            retain_forward_rule: RetainHandling::OnNewSubscribe,
            subscription_identifier: None,
            create_time: 0,
        }
    }

    #[test]
    fn test_add() {
        let mgr = BucketsManager::new(10);

        mgr.add(&create_sub("c1", "/t1"));
        mgr.add(&create_sub("c2", "/t2"));

        assert_eq!(mgr.seq_num.load(std::sync::atomic::Ordering::Relaxed), 2);
        assert_eq!(mgr.buckets_data_list.len(), 1);

        let bucket = mgr.buckets_data_list.iter().next().unwrap();
        assert_eq!(bucket.len(), 2);
        assert!(bucket.get(&0).is_some());
        assert!(bucket.get(&1).is_some());

        assert!(mgr.client_id_sub.get("c1").unwrap().contains(&0));
        assert!(mgr.client_id_sub.get("c2").unwrap().contains(&1));

        let key = mgr.client_sub_path_key("c1", "/t1");
        assert!(mgr.client_id_sub_path_sub.get(&key).unwrap().contains(&0));
    }

    #[test]
    fn test_bucket_overflow() {
        let mgr = BucketsManager::new(2);

        for i in 0..5 {
            mgr.add(&create_sub(&format!("c{}", i), "/t"));
        }

        assert!(mgr.buckets_data_list.len() >= 2);

        let total: usize = mgr.buckets_data_list.iter().map(|b| b.len()).sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn test_remove_by_client_id() {
        let mgr = BucketsManager::new(10);

        mgr.add(&create_sub("c1", "/t1"));
        mgr.add(&create_sub("c1", "/t2"));
        mgr.add(&create_sub("c2", "/t1"));

        assert_eq!(mgr.client_id_sub.get("c1").unwrap().len(), 2);

        mgr.remove_by_client_id("c1");

        assert!(mgr.client_id_sub.get("c1").is_none());
        assert!(mgr
            .client_id_sub_path_sub
            .get(&mgr.client_sub_path_key("c1", "/t1"))
            .is_none());
        assert!(mgr.client_id_sub.get("c2").is_some());

        let total: usize = mgr.buckets_data_list.iter().map(|b| b.len()).sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn test_remove_by_sub() {
        let mgr = BucketsManager::new(10);

        mgr.add(&create_sub("c1", "/t1"));
        mgr.add(&create_sub("c1", "/t2"));

        mgr.remove_by_sub("c1", "/t1");

        assert_eq!(mgr.client_id_sub.get("c1").unwrap().len(), 1);
        assert!(mgr
            .client_id_sub_path_sub
            .get(&mgr.client_sub_path_key("c1", "/t1"))
            .is_none());
        assert!(mgr
            .client_id_sub_path_sub
            .get(&mgr.client_sub_path_key("c1", "/t2"))
            .is_some());
    }

    #[test]
    fn test_cleanup_empty_bucket() {
        let mgr = BucketsManager::new(10);

        mgr.add(&create_sub("c1", "/t"));

        assert_eq!(mgr.buckets_data_list.len(), 1);

        mgr.remove_by_client_id("c1");

        assert_eq!(mgr.buckets_data_list.len(), 0);
        assert_eq!(mgr.client_id_sub.len(), 0);
        assert_eq!(mgr.client_id_sub_path_sub.len(), 0);
    }
}
