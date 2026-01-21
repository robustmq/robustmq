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

use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_second},
};
use dashmap::DashMap;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::{
    handler::cache::MQTTCacheManager,
    subscribe::{
        buckets::SubPushThreadData, directly_push::DirectlyPushManager, manager::SubscribeManager,
        share_push::SharePushManager,
    },
};

pub mod buckets;
pub mod common;
pub mod directly_push;
pub mod manager;
pub mod parse;
pub mod push;
pub mod push_model;
pub mod share_push;

pub struct PushManager {
    cache_manager: Arc<MQTTCacheManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connection_manager: Arc<ConnectionManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    subscribe_manager: Arc<SubscribeManager>,
    directly_buckets_push_thread: DashMap<String, SubPushThreadData>,
    share_buckets_push_thread: DashMap<String, SubPushThreadData>,
}

impl PushManager {
    pub fn new(
        cache_manager: Arc<MQTTCacheManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        connection_manager: Arc<ConnectionManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        subscribe_manager: Arc<SubscribeManager>,
    ) -> Self {
        PushManager {
            cache_manager,
            storage_driver_manager,
            connection_manager,
            rocksdb_engine_handler,
            subscribe_manager,
            directly_buckets_push_thread: DashMap::new(),
            share_buckets_push_thread: DashMap::new(),
        }
    }

    pub async fn start(&self, stop_sx: &broadcast::Sender<bool>) {
        info!("PushManager started");
        let ac_fn = async || -> ResultCommonError {
            // directly
            self.start_directly_push_thread();
            self.stop_directly_push_thread();

            // share
            self.start_share_push_thread();
            self.stop_share_push_thread();
            Ok(())
        };
        loop_select_ticket(ac_fn, 1000, stop_sx).await;
        info!("PushManager stopped");
    }

    pub fn start_directly_push_thread(&self) {
        // Collect empty buckets to remove
        let empty_buckets: Vec<String> = self
            .subscribe_manager
            .directly_push
            .buckets_data_list
            .iter()
            .filter(|row| row.value().is_empty())
            .map(|row| row.key().clone())
            .collect();

        // Stop threads for empty buckets first
        for bucket_id in &empty_buckets {
            if let Some((_, thread_data)) = self.directly_buckets_push_thread.remove(bucket_id) {
                debug!("Stopping thread for empty bucket: {}", bucket_id);
                if let Err(e) = thread_data.sender.send(true) {
                    warn!("Failed to send stop signal to bucket {}: {}", bucket_id, e);
                }
            }
        }

        // Then remove empty buckets data
        for bucket_id in empty_buckets {
            self.subscribe_manager
                .directly_push
                .buckets_data_list
                .remove(&bucket_id);
            debug!("Removed empty bucket: {}", bucket_id);
        }

        // Start threads for new buckets
        for row in self
            .subscribe_manager
            .directly_push
            .buckets_data_list
            .iter()
        {
            let bucket_id = row.key().clone();
            if !self.directly_buckets_push_thread.contains_key(&bucket_id) {
                info!("Starting push thread for bucket: {}", bucket_id);

                let (sub_thread_stop_sx, _) = broadcast::channel(1);
                let thread_data = SubPushThreadData {
                    push_error_record_num: 0,
                    push_success_record_num: 0,
                    last_push_time: 0,
                    last_run_time: 0,
                    create_time: now_second(),
                    sender: sub_thread_stop_sx.clone(),
                };

                let push_manager = DirectlyPushManager::new(
                    self.subscribe_manager.clone(),
                    self.cache_manager.clone(),
                    self.storage_driver_manager.clone(),
                    self.connection_manager.clone(),
                    self.rocksdb_engine_handler.clone(),
                    bucket_id.clone(),
                );

                let stop_sx = sub_thread_stop_sx.clone();
                tokio::spawn(Box::pin(async move {
                    push_manager.start(&stop_sx).await;
                }));

                self.directly_buckets_push_thread
                    .insert(bucket_id, thread_data);
            }
        }
    }

    pub fn stop_directly_push_thread(&self) {
        let threads_to_stop: Vec<String> = self
            .directly_buckets_push_thread
            .iter()
            .filter(|row| {
                !self
                    .subscribe_manager
                    .directly_push
                    .buckets_data_list
                    .contains_key(row.key())
            })
            .map(|row| row.key().clone())
            .collect();

        for bucket_id in threads_to_stop {
            if let Some((_, thread_data)) = self.directly_buckets_push_thread.remove(&bucket_id) {
                info!("Stopping push thread for bucket: {}", bucket_id);
                if let Err(e) = thread_data.sender.send(true) {
                    warn!("Failed to send stop signal to bucket {}: {}", bucket_id, e);
                }
            }
        }
    }

    pub fn start_share_push_thread(&self) {
        // Collect empty share groups to remove
        let empty_groups: Vec<String> = self
            .subscribe_manager
            .share_push
            .iter()
            .filter(|row| row.value().buckets_data_list.is_empty())
            .map(|row| row.key().clone())
            .collect();

        // Stop threads for empty share groups first
        for group_name in &empty_groups {
            if let Some((_, thread_data)) = self.share_buckets_push_thread.remove(group_name) {
                debug!("Stopping thread for empty share group: {}", group_name);
                if let Err(e) = thread_data.sender.send(true) {
                    warn!(
                        "Failed to send stop signal to share group {}: {}",
                        group_name, e
                    );
                }
            }
        }

        // Then remove empty share groups data
        for group_name in empty_groups {
            self.subscribe_manager.share_push.remove(&group_name);
            self.subscribe_manager
                .share_group_topics
                .remove(&group_name);
            debug!("Removed empty share group: {}", group_name);
        }

        // Start threads for new share groups
        for row in self.subscribe_manager.share_push.iter() {
            let group_name = row.key().clone();
            if !self.share_buckets_push_thread.contains_key(&group_name) {
                info!("Starting share push thread for group: {}", group_name);

                let (sub_thread_stop_sx, _) = broadcast::channel(1);
                let thread_data = SubPushThreadData {
                    push_error_record_num: 0,
                    push_success_record_num: 0,
                    last_push_time: 0,
                    last_run_time: 0,
                    create_time: now_second(),
                    sender: sub_thread_stop_sx.clone(),
                };

                let push_manager = SharePushManager::new(
                    self.subscribe_manager.clone(),
                    self.cache_manager.clone(),
                    self.storage_driver_manager.clone(),
                    self.connection_manager.clone(),
                    self.rocksdb_engine_handler.clone(),
                    group_name.clone(),
                );

                let stop_sx = sub_thread_stop_sx.clone();
                tokio::spawn(async move {
                    push_manager.start(&stop_sx).await;
                });

                self.share_buckets_push_thread
                    .insert(group_name, thread_data);
            }
        }
    }

    pub fn stop_share_push_thread(&self) {
        let threads_to_stop: Vec<String> = self
            .share_buckets_push_thread
            .iter()
            .filter(|row| !self.subscribe_manager.share_push.contains_key(row.key()))
            .map(|row| row.key().clone())
            .collect();

        for group_name in threads_to_stop {
            if let Some((_, thread_data)) = self.share_buckets_push_thread.remove(&group_name) {
                info!("Stopping share push thread for group: {}", group_name);
                if let Err(e) = thread_data.sender.send(true) {
                    warn!(
                        "Failed to send stop signal to share group {}: {}",
                        group_name, e
                    );
                }
            }
        }
    }
}
