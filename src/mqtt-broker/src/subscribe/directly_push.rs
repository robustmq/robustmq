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

use crate::handler::sub_option::is_send_msg_by_bo_local;
use crate::subscribe::common::record_sub_send_metrics;
use crate::subscribe::push::send_message_validator;
use crate::{handler::cache::MQTTCacheManager, storage::message::MessageStorage};
use crate::{
    handler::tool::ResultMqttBrokerError,
    handler::{error::MqttBrokerError, sub_slow::record_slow_subscribe_data},
    subscribe::{
        common::{client_unavailable_error, Subscriber},
        manager::SubscribeManager,
        push::{build_publish_message, send_publish_packet_to_client},
        push_model::{get_push_model, PushModel},
    },
};
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::storage::convert::convert_engine_record_to_adapter;
use metadata_struct::storage::storage_record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::{sync::Arc, time::Duration};
use storage_adapter::driver::StorageDriverManager;
use tokio::{select, sync::broadcast::Sender, time::sleep};
use tracing::{debug, error, info, warn};

const BATCH_SIZE: u64 = 500;
const IDLE_SLEEP_MS: u64 = 100;
const LOW_LOAD_SLEEP_MS: u64 = 50;
const HIGH_LOAD_SLEEP_MS: u64 = 10;
const LOW_LOAD_THRESHOLD: usize = 10;

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
                    if let Ok(flag) = val {
                        if flag {
                            info!("DirectlyPushManager[{}] stopped", self.uuid);
                            break;
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
        if let Some(data) = self
            .subscribe_manager
            .directly_push
            .buckets_data_list
            .get(&self.uuid)
        {
            let subscriber_count = data.len();
            for row in data.iter() {
                match self.process_subscriber_messages(&row, stop_sx).await {
                    Ok(count) => {
                        processed_count += count;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to process messages for subscriber [client_id: {}, group: {}, topic: {}, sub_path: {}],error message: {}",
                            row.client_id, row.group_name, row.topic_name, row.sub_path, e
                        );
                    }
                }
            }

            if processed_count > 0 {
                debug!(
                    "Processed {} messages across {} subscribers in bucket [{}]",
                    processed_count, subscriber_count, self.uuid
                );
            }
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
            .next_message(&subscriber.group_name, &subscriber.topic_name)
            .await?;

        let data_list_len = data_list.len();
        if data_list_len == 0 {
            return Ok(0);
        }

        let model = get_push_model(&subscriber.client_id, &subscriber.topic_name);

        for record in data_list {
            let success = match self.push_data(subscriber, &record, stop_sx).await {
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
                &subscriber.client_id,
                &subscriber.sub_path,
                &subscriber.topic_name,
                0,
                success,
            );

            if let Some(mut offsets) = self.group_offsets.get_mut(&subscriber.group_name) {
                offsets.insert(
                    record.metadata.shard.to_string(),
                    record.metadata.offset + 1,
                );
            }
        }

        if let Some(offsets) = self.group_offsets.get(&subscriber.group_name) {
            if let Err(e) = self.commit_offset(&subscriber.group_name, &offsets).await {
                error!(
                    "Failed to commit offset for subscriber [client_id: {}, group: {}, topic: {}]: {}. Messages may be redelivered on next poll",
                    subscriber.client_id, subscriber.group_name, subscriber.topic_name, e
                );
            } else {
                debug!(
                    "Committed offset for subscriber [client_id: {}, topic: {}], processed: {} messages",
                    subscriber.client_id, subscriber.topic_name, processed_count
                );
            }
        } else if data_list_len > 0 {
            debug!(
                "No offset to commit for subscriber [client_id: {}, topic: {}], all messages failed or skipped",
                subscriber.client_id, subscriber.topic_name
            );
        }

        Ok(processed_count)
    }

    async fn push_data(
        &self,
        subscriber: &Subscriber,
        record: &StorageRecord,
        stop_sx: &Sender<bool>,
    ) -> Result<bool, MqttBrokerError> {
        let new_record = convert_engine_record_to_adapter(record.clone());

        let msg = MqttMessage::decode_record(new_record)?;
        if !send_message_validator(&self.cache_manager, &subscriber.client_id, &msg).await? {
            return Ok(false);
        }

        if !is_send_msg_by_bo_local(subscriber.no_local, &subscriber.client_id, &msg.client_id) {
            debug!(
                "Message dropping: no_local constraint, client_id: {}, topic: {}",
                subscriber.client_id, subscriber.topic_name
            );
            return Ok(false);
        }
        let sub_pub_param = if let Some(params) = build_publish_message(
            &self.cache_manager,
            &self.connection_manager,
            &msg,
            subscriber,
        )
        .await?
        {
            params
        } else {
            // Message skipped (expired, no_local, packet too large, etc.)
            return Ok(false);
        };

        let send_time = now_second();
        send_publish_packet_to_client(
            &self.connection_manager,
            &self.cache_manager,
            &sub_pub_param,
            stop_sx,
        )
        .await?;

        record_slow_subscribe_data(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            subscriber,
            send_time,
            record.metadata.create_t,
        )
        .await?;

        Ok(true)
    }

    async fn next_message(
        &self,
        group: &str,
        topic_name: &str,
    ) -> Result<Vec<StorageRecord>, MqttBrokerError> {
        let offsets = if let Some(offsets) = self.group_offsets.get(group) {
            offsets.clone()
        } else {
            let offsets = self.message_storage.get_group_offset(group).await?;
            self.group_offsets
                .insert(group.to_string(), offsets.clone());
            offsets
        };

        let data = self
            .message_storage
            .read_topic_message(topic_name, &offsets, BATCH_SIZE)
            .await?;
        Ok(data)
    }

    async fn commit_offset(
        &self,
        group: &str,
        offsets: &HashMap<String, u64>,
    ) -> ResultMqttBrokerError {
        self.message_storage
            .commit_group_offset(group, offsets)
            .await?;
        Ok(())
    }
}

pub fn directly_group_name(client_id: &str, path: &str, topic_name: &str) -> String {
    format!("directly_sub_{client_id}_{path}_{topic_name}")
}
