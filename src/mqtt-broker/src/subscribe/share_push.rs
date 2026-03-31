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

use crate::core::sub_option::message_is_same_client;
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
use crate::subscribe::common::{message_is_exceeds_max_message_size, message_is_expire};
use crate::subscribe::common::{record_sub_send_metrics, stale_subscriber_error};
use crate::subscribe::push::{
    push_data, BATCH_SIZE, HIGH_LOAD_SLEEP_MS, IDLE_SLEEP_MS, LOW_LOAD_SLEEP_MS, LOW_LOAD_THRESHOLD,
};
use crate::{core::cache::MQTTCacheManager, storage::message::MessageStorage};
use crate::{
    core::error::MqttBrokerError,
    core::tool::ResultMqttBrokerError,
    subscribe::{common::client_unavailable_error, manager::SubscribeManager},
};
use dashmap::DashMap;
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::{sync::Arc, time::Duration};
use storage_adapter::driver::StorageDriverManager;
use tokio::{select, sync::broadcast, sync::broadcast::Sender, time::sleep};
use tracing::{debug, error, info};

pub struct SharePushManager {
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<MQTTCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    group_offsets: DashMap<String, HashMap<String, u64>>,
    message_storage: MessageStorage,
    tenant: String,
    group_name: String,
    seq: AtomicU64,
}

impl SharePushManager {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<MQTTCacheManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        connection_manager: Arc<ConnectionManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        tenant: String,
        group_name: String,
    ) -> Self {
        SharePushManager {
            subscribe_manager,
            message_storage: MessageStorage::new(storage_driver_manager),
            cache_manager,
            rocksdb_engine_handler,
            connection_manager,
            group_offsets: DashMap::with_capacity(8),
            tenant,
            group_name,
            seq: AtomicU64::new(0),
        }
    }

    pub async fn start(&self, stop_sx: &Sender<bool>) {
        info!("SharePushManager[{}] started", self.group_name);
        let mut stop_rx = stop_sx.subscribe();
        loop {
            select! {
                val = stop_rx.recv() =>{
                    match val {
                        Ok(true) => {
                            info!("SharePushManager[{}] stopped", self.group_name);
                            break;
                        }
                        Ok(false) => {}
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("SharePushManager[{}] stop channel closed, exiting.", self.group_name);
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            debug!(
                                "SharePushManager[{}] stop channel lagged, skipped {} messages.",
                                self.group_name, skipped
                            );
                        }
                    }
                }
                res = self.send_messages(stop_sx) =>{
                    match res {
                        Ok(processed_count) => {
                            if processed_count == 0 {
                                sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                            } else if processed_count < LOW_LOAD_THRESHOLD {
                                sleep(Duration::from_millis(LOW_LOAD_SLEEP_MS)).await;
                            } else {
                                sleep(Duration::from_millis(HIGH_LOAD_SLEEP_MS)).await;
                            }
                        }
                        Err(e) => {
                            if stale_subscriber_error(&e) {
                                info!("SharePushManager[{}] stopping: topic no longer exists ({})", self.group_name, e);
                                break;
                            }
                            error!("SharePushManager[{}] send messages failed: {}", self.group_name, e);
                            sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn send_messages(&self, stop_sx: &Sender<bool>) -> Result<u64, MqttBrokerError> {
        let topic_list = self
            .subscribe_manager
            .share_group_topics
            .get(&self.tenant)
            .and_then(|t| t.get(&self.group_name).map(|v| v.clone()));

        let topic_list = match topic_list {
            Some(list) if !list.is_empty() => list,
            _ => return Ok(0),
        };

        let buckets = self
            .subscribe_manager
            .share_push
            .get(&self.tenant)
            .and_then(|t| t.get(&self.group_name).map(|v| v.clone()));

        let buckets = match buckets {
            Some(b) => b,
            None => return Ok(0),
        };

        let seqs = buckets.get_sub_client_seqs(&self.group_name);
        if seqs.is_empty() {
            return Ok(0);
        }

        let mut processed_count = 0;
        for topic in topic_list.iter() {
            let data_list = self
                .next_message(&self.group_name, &self.tenant, &topic.topic)
                .await?;

            if data_list.is_empty() {
                continue;
            }

            for record in data_list {
                // If the message has expired, it will not be delivered.
                if message_is_expire(&record) {
                    if let Some(mut offsets) = self.group_offsets.get_mut(&self.group_name) {
                        offsets.insert(record.metadata.shard.to_string(), record.metadata.offset);
                    }
                    continue;
                }

                let max_attempts = seqs.len();
                let mut attempts = 0;
                let mut break_flag = false;
                loop {
                    if attempts >= max_attempts {
                        debug!(
                            "Failed to push message after {} attempts for group {}, skipping record",
                            attempts, self.group_name
                        );
                        break_flag = true;
                        break;
                    }
                    attempts += 1;

                    let row_seq = self.seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let index = row_seq % (seqs.len() as u64);
                    if let Some(subscriber) =
                        buckets.get_subscribe_by_key_seq(&self.group_name, index)
                    {
                        // check allow send
                        if !self
                            .subscribe_manager
                            .allow_push_client(&subscriber.tenant, &subscriber.client_id)
                        {
                            continue;
                        }

                        // If the message exceeds the size limit specified for the entire client ID, try the next client.
                        match message_is_exceeds_max_message_size(
                            &self.cache_manager,
                            &subscriber.client_id,
                            &record,
                        )
                        .await
                        {
                            Ok(allow) => {
                                if !allow {
                                    continue;
                                }
                            }
                            Err(e) => {
                                if !client_unavailable_error(&e) {
                                    self.subscribe_manager.add_not_push_client(
                                        &subscriber.tenant,
                                        &subscriber.client_id,
                                    );
                                }
                                continue;
                            }
                        };

                        // If the message cannot be sent to the entire client group, try the next client.
                        if !message_is_same_client(&subscriber, &record) {
                            continue;
                        }

                        // Push data
                        let success = match push_data(
                            &self.connection_manager,
                            &self.cache_manager,
                            &self.rocksdb_engine_handler,
                            &subscriber,
                            &record,
                            stop_sx,
                        )
                        .await
                        {
                            Ok(pushed) => {
                                if pushed {
                                    processed_count += 1;
                                }
                                pushed
                            }
                            Err(e) => {
                                if !client_unavailable_error(&e) {
                                    self.subscribe_manager.add_not_push_client(
                                        &subscriber.tenant,
                                        &subscriber.client_id,
                                    );
                                }

                                continue;
                            }
                        };

                        record_sub_send_metrics(
                            &subscriber.tenant,
                            &subscriber.client_id,
                            &subscriber.sub_path,
                            &subscriber.topic_name,
                            0,
                            success,
                        );

                        if let Some(mut offsets) =
                            self.group_offsets.get_mut(&subscriber.group_name)
                        {
                            offsets.insert(
                                record.metadata.shard.to_string(),
                                record.metadata.offset + 1,
                            );
                        }

                        break;
                    }
                }

                if break_flag {
                    break;
                }
            }

            if let Some(offsets) = self.group_offsets.get(&self.group_name).map(|r| r.clone()) {
                self.commit_offset(&self.tenant, &self.group_name, &offsets)
                    .await?;
            }
        }

        Ok(processed_count)
    }

    async fn next_message(
        &self,
        group: &str,
        tenant: &str,
        topic_name: &str,
    ) -> Result<Vec<StorageRecord>, MqttBrokerError> {
        let offsets = if let Some(offsets) = self.group_offsets.get(group) {
            offsets.clone()
        } else {
            let offsets = self.message_storage.get_group_offset(tenant, group).await?;
            self.group_offsets
                .insert(group.to_string(), offsets.clone());
            offsets
        };

        Ok(self
            .message_storage
            .read_topic_message(tenant, topic_name, &offsets, BATCH_SIZE)
            .await?)
    }

    async fn commit_offset(
        &self,
        tenant: &str,
        group: &str,
        offsets: &HashMap<String, u64>,
    ) -> ResultMqttBrokerError {
        self.message_storage
            .commit_group_offset(tenant, group, offsets)
            .await?;
        Ok(())
    }
}
