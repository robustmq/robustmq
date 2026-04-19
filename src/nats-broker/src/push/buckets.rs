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

use common_base::uuid::unique_id;
use dashmap::DashMap;
use metadata_struct::nats::subscriber::NatsSubscriber;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Bucket-based subscriber index for NATS push.
///
/// Each bucket holds subscribers for one dedicated push thread.
/// Indexed by `connect_id`, `connect_id#sid`, and `topic_name` for O(1) removal.
///
/// Buckets are pre-registered via `register_bucket` before any subscribers arrive.
/// New subscribers are placed into buckets via round-robin. Pre-registered buckets
/// are permanent — never deleted when empty.
#[derive(Clone, Default)]
pub struct NatsBucketsManager {
    // (bucket_id, (seq, NatsSubscriber))
    pub buckets_data_list: DashMap<String, DashMap<u64, NatsSubscriber>>,

    // connect_id → {seq}
    connect_id_sub: DashMap<u64, HashSet<u64>>,

    // connect_id#sid → {seq}  (one sid can match multiple topics via wildcards)
    connect_id_sid_sub: DashMap<String, HashSet<u64>>,

    // topic_name → {seq}
    topic_sub: DashMap<String, HashSet<u64>>,

    // seq → bucket_id  for O(1) removal without scanning all buckets
    seq_bucket: DashMap<u64, String>,

    // uniq_id → seq  for deduplication
    sub_seq: DashMap<String, u64>,

    // Ordered list of pre-registered bucket IDs for round-robin placement.
    // Populated by register_bucket; immutable after startup.
    // Also used to determine which buckets are permanent (never deleted when empty).
    bucket_order: Arc<std::sync::RwLock<Vec<String>>>,

    // Round-robin counter for distributing subscribers across pre-registered buckets.
    round_robin: Arc<AtomicU64>,

    seq_num: Arc<AtomicU64>,
}

impl NatsBucketsManager {
    pub fn new() -> Self {
        NatsBucketsManager {
            seq_num: Arc::new(AtomicU64::new(0)),
            round_robin: Arc::new(AtomicU64::new(0)),
            connect_id_sub: DashMap::with_capacity(128),
            connect_id_sid_sub: DashMap::with_capacity(128),
            buckets_data_list: DashMap::with_capacity(8),
            topic_sub: DashMap::with_capacity(8),
            seq_bucket: DashMap::with_capacity(256),
            sub_seq: DashMap::with_capacity(256),
            bucket_order: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    /// Pre-register a bucket with a known ID before push threads start.
    /// Registered buckets are never deleted when empty.
    pub fn register_bucket(&self, bucket_id: String) {
        self.buckets_data_list
            .entry(bucket_id.clone())
            .or_insert_with(|| DashMap::with_capacity(2));
        self.bucket_order.write().unwrap().push(bucket_id);
    }

    pub fn add(&self, subscriber: &NatsSubscriber) {
        if self.sub_seq.contains_key(&subscriber.uniq_id) {
            return;
        }

        let seq = self.seq_num.fetch_add(1, Ordering::Relaxed);

        self.connect_id_sub
            .entry(subscriber.connect_id)
            .or_default()
            .insert(seq);

        let sid_key = sid_key(subscriber.connect_id, &subscriber.sid);
        self.connect_id_sid_sub
            .entry(sid_key)
            .or_default()
            .insert(seq);

        self.topic_sub
            .entry(subscriber.subject.clone())
            .or_default()
            .insert(seq);

        self.sub_seq.insert(subscriber.uniq_id.clone(), seq);

        let bucket_id = self.pick_bucket();
        self.seq_bucket.insert(seq, bucket_id.clone());

        self.buckets_data_list
            .entry(bucket_id)
            .or_insert_with(|| DashMap::with_capacity(2))
            .insert(seq, subscriber.clone());
    }

    pub fn remove_by_connect_id(&self, connect_id: u64) {
        let seqs: Vec<u64> = self
            .connect_id_sub
            .get(&connect_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        for seq in seqs {
            self.remove_by_seq(seq);
        }
    }

    pub fn remove_by_sid(&self, connect_id: u64, sid: &str) {
        let key = sid_key(connect_id, sid);
        let seqs: Vec<u64> = self
            .connect_id_sid_sub
            .get(&key)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        for seq in seqs {
            self.remove_by_seq(seq);
        }
    }

    pub fn remove_by_topic(&self, topic_name: &str) {
        let seqs: Vec<u64> = self
            .topic_sub
            .get(topic_name)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        for seq in seqs {
            self.remove_by_seq(seq);
        }
    }

    pub fn sub_len(&self) -> u64 {
        self.buckets_data_list
            .iter()
            .map(|b| b.value().len() as u64)
            .sum()
    }

    fn pick_bucket(&self) -> String {
        let order = self.bucket_order.read().unwrap();
        if !order.is_empty() {
            let idx = self.round_robin.fetch_add(1, Ordering::Relaxed) as usize % order.len();
            return order[idx].clone();
        }
        unique_id()
    }

    fn remove_by_seq(&self, seq: u64) {
        let bucket_id = match self.seq_bucket.remove(&seq) {
            Some((_, bid)) => bid,
            None => return,
        };

        let subscriber = if let Some(bucket) = self.buckets_data_list.get(&bucket_id) {
            match bucket.remove(&seq) {
                Some((_, sub)) => sub,
                None => return,
            }
        } else {
            return;
        };

        // Clean connect_id index
        if let Some(mut seqs) = self.connect_id_sub.get_mut(&subscriber.connect_id) {
            seqs.remove(&seq);
            if seqs.is_empty() {
                drop(seqs);
                self.connect_id_sub.remove(&subscriber.connect_id);
            }
        }

        // Clean connect_id#sid index
        let sk = sid_key(subscriber.connect_id, &subscriber.sid);
        if let Some(mut seqs) = self.connect_id_sid_sub.get_mut(&sk) {
            seqs.remove(&seq);
            if seqs.is_empty() {
                drop(seqs);
                self.connect_id_sid_sub.remove(&sk);
            }
        }

        // Clean topic index
        if let Some(mut seqs) = self.topic_sub.get_mut(&subscriber.subject) {
            seqs.remove(&seq);
            if seqs.is_empty() {
                drop(seqs);
                self.topic_sub.remove(&subscriber.subject);
            }
        }

        self.sub_seq.remove(&subscriber.uniq_id);

        let is_registered = self.bucket_order.read().unwrap().contains(&bucket_id);
        if !is_registered
            && self
                .buckets_data_list
                .get(&bucket_id)
                .map(|b| b.is_empty())
                .unwrap_or(false)
        {
            self.buckets_data_list.remove(&bucket_id);
        }
    }
}

fn sid_key(connect_id: u64, sid: &str) -> String {
    format!("{}#{}", connect_id, sid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::now_second;

    fn make_sub(connect_id: u64, sid: &str, topic: &str) -> NatsSubscriber {
        NatsSubscriber {
            uniq_id: unique_id(),
            tenant: "default".to_string(),
            connect_id,
            broker_id: 1,
            sid: sid.to_string(),
            sub_subject: topic.to_string(),
            subject: topic.to_string(),
            queue_group: String::new(),
            create_time: now_second(),
        }
    }

    #[test]
    fn test_add_dedup_and_len() {
        let mgr = NatsBucketsManager::new();
        let sub = make_sub(1, "s1", "foo");
        mgr.add(&sub);
        mgr.add(&sub); // duplicate — ignored
        mgr.add(&make_sub(2, "s2", "bar"));
        assert_eq!(mgr.sub_len(), 2);
    }

    #[test]
    fn test_round_robin_and_bucket_lifecycle() {
        let mgr = NatsBucketsManager::new();
        mgr.register_bucket("b0".to_string());
        mgr.register_bucket("b1".to_string());
        mgr.register_bucket("b2".to_string());

        // Round-robin distributes evenly across 3 registered buckets.
        for i in 0..9u64 {
            mgr.add(&make_sub(i, "s1", &format!("t{}", i)));
        }
        assert_eq!(mgr.buckets_data_list.get("b0").unwrap().len(), 3);
        assert_eq!(mgr.buckets_data_list.get("b1").unwrap().len(), 3);
        assert_eq!(mgr.buckets_data_list.get("b2").unwrap().len(), 3);

        // Registered buckets persist even when emptied; dynamic buckets are removed.
        mgr.remove_by_connect_id(0); // in b0
        assert_eq!(mgr.sub_len(), 8);
        assert_eq!(mgr.buckets_data_list.len(), 3); // b0 kept

        // Dynamic bucket (no register_bucket call) is cleaned up when empty.
        let mgr2 = NatsBucketsManager::new();
        mgr2.add(&make_sub(1, "s1", "foo"));
        assert_eq!(mgr2.buckets_data_list.len(), 1);
        mgr2.remove_by_connect_id(1);
        assert_eq!(mgr2.buckets_data_list.len(), 0);
    }

    #[test]
    fn test_remove_by_connect_id() {
        let mgr = NatsBucketsManager::new();
        mgr.add(&make_sub(1, "s1", "foo"));
        mgr.add(&make_sub(1, "s2", "bar"));
        mgr.add(&make_sub(2, "s1", "baz"));

        mgr.remove_by_connect_id(1);

        assert_eq!(mgr.sub_len(), 1);
        assert!(mgr.connect_id_sub.get(&1).is_none());
        assert!(mgr.connect_id_sub.get(&2).is_some());
    }

    #[test]
    fn test_remove_by_sid() {
        let mgr = NatsBucketsManager::new();
        mgr.add(&make_sub(1, "s1", "foo"));
        mgr.add(&make_sub(1, "s1", "bar"));
        mgr.add(&make_sub(1, "s2", "baz"));

        mgr.remove_by_sid(1, "s1");
        assert_eq!(mgr.sub_len(), 1);
    }

    #[test]
    fn test_remove_by_topic() {
        let mgr = NatsBucketsManager::new();
        mgr.add(&make_sub(1, "s1", "foo"));
        mgr.add(&make_sub(2, "s2", "foo"));
        mgr.add(&make_sub(3, "s3", "bar"));

        mgr.remove_by_topic("foo");
        assert_eq!(mgr.sub_len(), 1);
    }
}
