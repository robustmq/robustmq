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
use crate::subscribe::common::{message_is_exceeds_max_message_size, message_is_expire};
use crate::subscribe::common::{record_sub_send_metrics, stale_subscriber_error};
use crate::subscribe::push::{
    push_data, BATCH_SIZE, HIGH_LOAD_SLEEP_MS, IDLE_SLEEP_MS, LOW_LOAD_SLEEP_MS, LOW_LOAD_THRESHOLD,
};
use crate::{core::cache::MQTTCacheManager, storage::message::MessageStorage};
use crate::{
    core::error::MqttBrokerError,
    core::tool::ResultMqttBrokerError,
    subscribe::{
        common::{client_unavailable_error, Subscriber},
        manager::SubscribeManager,
        push_model::{get_push_model, PushModel},
    },
};
use dashmap::DashMap;
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::{sync::Arc, time::Duration};
use storage_adapter::driver::StorageDriverManager;
use tokio::{select, sync::broadcast, sync::broadcast::Sender, time::sleep};
use tracing::{debug, error, info, warn};

pub struct DirectlyPushManager {
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<MQTTCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    message_storage: MessageStorage,
    group_offsets: DashMap<String, HashMap<String, u64>>,
    uuid: String,
}

impl DirectlyPushManager {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<MQTTCacheManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        connection_manager: Arc<ConnectionManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        uuid: String,
    ) -> Self {
        DirectlyPushManager {
            subscribe_manager,
            message_storage: MessageStorage::new(storage_driver_manager),
            cache_manager,
            rocksdb_engine_handler,
            connection_manager,
            group_offsets: DashMap::with_capacity(2),
            uuid,
        }
    }

    pub async fn start(&self, stop_sx: &Sender<bool>) {
        let mut stop_rx = stop_sx.subscribe();
        loop {
            select! {
                val = stop_rx.recv() =>{
                    match val {
                        Ok(true) => {
                            info!("DirectlyPushManager[{}] stopped", self.uuid);
                            break;
                        }
                        Ok(false) => {}
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("DirectlyPushManager[{}] stop channel closed, exiting.", self.uuid);
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            debug!(
                                "DirectlyPushManager[{}] stop channel lagged, skipped {} messages.",
                                self.uuid, skipped
                            );
                        }
                    }
                }
                res = self.send_messages(stop_sx) =>{
                    match res {
                        Ok(processed_count) => {
                            if processed_count == 0 {
                                sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                            } else if processed_count < LOW_LOAD_THRESHOLD as usize {
                                sleep(Duration::from_millis(LOW_LOAD_SLEEP_MS)).await;
                            } else {
                                sleep(Duration::from_millis(HIGH_LOAD_SLEEP_MS)).await;
                            }
                        }
                        Err(e) => {
                            error!("DirectlyPushManager[{}] send messages failed: {}", self.uuid, e);
                            sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn send_messages(&self, stop_sx: &Sender<bool>) -> Result<usize, MqttBrokerError> {
        let mut processed_count = 0;
        let mut stale_subs: Vec<(String, String, String)> = Vec::new();

        if let Some(data) = self
            .subscribe_manager
            .directly_push
            .buckets_data_list
            .get(&self.uuid)
        {
            for row in data.iter() {
                match self.process_subscriber_messages(&row, stop_sx).await {
                    Ok(count) => {
                        processed_count += count;
                    }
                    Err(e) => {
                        if stale_subscriber_error(&e) {
                            warn!(
                                "Removing stale subscriber [client_id: {}, topic: {}, sub_path: {}]: {}",
                                row.client_id, row.topic_name, row.sub_path, e
                            );
                            stale_subs.push((
                                row.tenant.clone(),
                                row.client_id.clone(),
                                row.sub_path.clone(),
                            ));
                        } else {
                            warn!(
                                "Failed to process messages for subscriber [client_id: {}, group: {}, topic: {}, sub_path: {}],error message: {}",
                                row.client_id, row.group_name, row.topic_name, row.sub_path, e
                            );
                        }
                    }
                }
            }
        }

        for (tenant, client_id, sub_path) in stale_subs {
            self.subscribe_manager
                .remove_by_sub(&tenant, &client_id, &sub_path);
        }

        Ok(processed_count)
    }

    async fn process_subscriber_messages(
        &self,
        subscriber: &Subscriber,
        stop_sx: &Sender<bool>,
    ) -> Result<usize, MqttBrokerError> {
        let mut processed_count = 0;

        let data_list = self
            .next_message(
                &subscriber.group_name,
                &subscriber.tenant,
                &subscriber.topic_name,
            )
            .await?;

        let data_list_len = data_list.len();
        if data_list_len == 0 {
            return Ok(0);
        }

        let model = get_push_model(&subscriber.client_id, &subscriber.topic_name);

        for record in data_list {
            if !is_discard_message(&self.cache_manager, &record, subscriber).await? {
                let success = match push_data(
                    &self.connection_manager,
                    &self.cache_manager,
                    &self.rocksdb_engine_handler,
                    subscriber,
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
                            warn!(
                                "Directly push fail, offset [{}], error message:{}",
                                record.metadata.offset, e
                            );
                        }
                        if model == PushModel::RetryFailure {
                            break;
                        }
                        false
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
            }

            if let Some(mut offsets) = self.group_offsets.get_mut(&subscriber.group_name) {
                offsets.insert(
                    record.metadata.shard.to_string(),
                    record.metadata.offset + 1,
                );
            }
        }

        if let Some(offsets) = self
            .group_offsets
            .get(&subscriber.group_name)
            .map(|r| r.clone())
        {
            self.commit_offset(&subscriber.tenant, &subscriber.group_name, &offsets)
                .await?;
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

        let data = self
            .message_storage
            .read_topic_message(tenant, topic_name, &offsets, BATCH_SIZE)
            .await?;
        Ok(data)
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

pub fn directly_group_name(client_id: &str, path: &str, topic_name: &str) -> String {
    format!("directly_sub_{client_id}_{path}_{topic_name}")
}

async fn is_discard_message(
    cache_manager: &Arc<MQTTCacheManager>,
    record: &StorageRecord,
    subscriber: &Subscriber,
) -> Result<bool, MqttBrokerError> {
    if message_is_expire(record) {
        return Ok(true);
    }

    if message_is_exceeds_max_message_size(cache_manager, &subscriber.client_id, record).await? {
        return Ok(true);
    }

    if message_is_same_client(subscriber, record) {
        return Ok(true);
    }

    Ok(false)
}
