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
use std::sync::{atomic::AtomicU32, Arc};
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

#[derive(Clone, Default)]
pub struct BucketsManager {
    // (bucket_id, (seq,subscriber))
    pub buckets_data_list: DashMap<String, DashMap<u32, Subscriber>>,

    // (client_id, (seq))
    client_id_sub: DashMap<String, Vec<u32>>,
    // (client_id_sub_path, (seq))
    client_id_sub_path_sub: DashMap<String, Vec<u32>>,

    bucket_size: u32,
    seq_num: Arc<AtomicU32>,
}

impl BucketsManager {
    pub fn new(bucket_len: u32) -> Self {
        BucketsManager {
            bucket_size: bucket_len,
            seq_num: Arc::new(AtomicU32::new(0)),
            client_id_sub: DashMap::new(),
            client_id_sub_path_sub: DashMap::new(),
            buckets_data_list: DashMap::with_capacity(2),
        }
    }

    pub async fn add(&self, subscriber: &Subscriber) {
        let seq = self
            .seq_num
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // add client_id sub
        if let Some(mut data) = self.client_id_sub.get_mut(&subscriber.client_id) {
            if !data.contains(&seq) {
                data.push(seq);
            }
        } else {
            self.client_id_sub
                .insert(subscriber.client_id.clone(), vec![seq]);
        }

        // add client_id_sub_path_sub
        let key = self.client_sub_path_key(&subscriber.client_id, &subscriber.sub_path);
        if let Some(mut data) = self.client_id_sub_path_sub.get_mut(&key) {
            if !data.contains(&seq) {
                data.push(seq);
            }
        } else {
            self.client_id_sub_path_sub.insert(key, vec![seq]);
        }

        // add data list
        self.add_data_list(seq, subscriber).await;
    }

    pub async fn remove_by_client_id(&self, client_id: &str) {
        if let Some(data) = self.client_id_sub.get(client_id) {
            for seq in data.iter() {
                self.remove_data_list_by_seq(seq).await;
            }
        }
    }

    pub async fn remove_by_sub(&self, client_id: &str, sub_path: &str) {
        let key = self.client_sub_path_key(client_id, sub_path);
        if let Some(data) = self.client_id_sub_path_sub.get(&key) {
            for seq in data.iter() {
                self.remove_data_list_by_seq(seq).await;
            }
        }
    }

    // data list
    async fn add_data_list(&self, seq: u32, subscriber: &Subscriber) {
        let mut write_success = false;
        for row in self.buckets_data_list.iter() {
            if row.len() as u32 >= self.bucket_size {
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

    async fn remove_data_list_by_seq(&self, seq: &u32) {
        for row in self.buckets_data_list.iter() {
            if let Some((_, subscriber)) = row.remove(seq) {
                if let Some(mut data) = self.client_id_sub.get_mut(&subscriber.client_id) {
                    data.retain(|s| s != seq);
                    if data.is_empty() {
                        drop(data);
                        self.client_id_sub.remove(&subscriber.client_id);
                    }
                }

                let key = self.client_sub_path_key(&subscriber.client_id, &subscriber.sub_path);
                if let Some(mut data) = self.client_id_sub_path_sub.get_mut(&key) {
                    data.retain(|s| s != seq);
                    if data.is_empty() {
                        drop(data);
                        self.client_id_sub_path_sub.remove(&key);
                    }
                }

                break;
            }
        }
    }

    fn client_sub_path_key(&self, client_id: &str, sub_path: &str) -> String {
        format!("{client_id}_{sub_path}")
    }
}
