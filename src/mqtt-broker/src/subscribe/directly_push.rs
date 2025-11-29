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

use crate::{
    common::types::ResultMqttBrokerError,
    handler::{error::MqttBrokerError, sub_slow::record_slow_subscribe_data},
    subscribe::{
        common::{is_ignore_push_error, Subscriber},
        manager::SubscribeManager,
        push::{build_publish_message, send_publish_packet_to_client},
        PushModel,
    },
};
use crate::{handler::cache::MQTTCacheManager, storage::message::MessageStorage};
use common_base::tools::now_second;
use common_metrics::mqtt::subscribe::{
    record_subscribe_bytes_sent, record_subscribe_messages_sent, record_subscribe_topic_bytes_sent,
    record_subscribe_topic_messages_sent,
};
use dashmap::DashMap;
use metadata_struct::adapter::record::Record;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::sync::broadcast::Sender;
use tracing::warn;

pub struct DirectlyPushManager {
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<MQTTCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    message_storage: MessageStorage,
    offset_cache: DashMap<String, u64>,
    uuid: String,
}

impl DirectlyPushManager {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<MQTTCacheManager>,
        storage_adapter: ArcStorageAdapter,
        connection_manager: Arc<ConnectionManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        uuid: String,
    ) -> Self {
        DirectlyPushManager {
            subscribe_manager,
            message_storage: MessageStorage::new(storage_adapter),
            offset_cache: DashMap::with_capacity(2),
            cache_manager,
            rocksdb_engine_handler,
            connection_manager,
            uuid,
        }
    }

    pub async fn start(&self, stop_sx: &Sender<bool>) -> ResultMqttBrokerError {
        if let Some(data) = self
            .subscribe_manager
            .directly_push
            .buckets_data_list
            .get(&self.uuid)
        {
            for row in data.iter() {
                let data_list = self.next_message(&row.group_name, &row.topic_name).await?;
                for record in data_list {
                    let record_offset = if let Some(offset) = record.offset {
                        offset
                    } else {
                        continue;
                    };

                    let mut is_commit_offset = true;
                    let mut success = true;
                    let model = PushModel::QuickFailure;
                    if let Err(e) = self.push_data(&row, &record, stop_sx).await {
                        if !is_ignore_push_error(&e) {
                            warn!(
                                "Exclusive push fail, offset [{:?}], error message:{},",
                                record.offset, e
                            );
                        }
                        if model == PushModel::RetryFailure {
                            is_commit_offset = false
                        }
                        success = false;
                    }

                    self.record_metrics(
                        &row.client_id,
                        &row.sub_path,
                        &row.topic_name,
                        record.data.len() as u64,
                        success,
                    );

                    if is_commit_offset {
                        self.commit_offset(&row.group_name, &row.topic_name, record_offset)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn push_data(
        &self,
        subscriber: &Subscriber,
        record: &Record,
        stop_sx: &Sender<bool>,
    ) -> ResultMqttBrokerError {
        // build publish params
        let sub_pub_param = if let Some(params) = build_publish_message(
            &self.cache_manager,
            &self.connection_manager,
            record,
            subscriber,
        )
        .await?
        {
            params
        } else {
            return Ok(());
        };

        let send_time = now_second();

        // publish data to client
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
            record.timestamp,
        )
        .await?;

        Ok(())
    }

    async fn next_message(
        &self,
        group: &str,
        topic_name: &str,
    ) -> Result<Vec<Record>, MqttBrokerError> {
        let offset = self.get_offset(group, topic_name).await?;
        Ok(self
            .message_storage
            .read_topic_message(topic_name, offset, 100)
            .await?)
    }

    async fn commit_offset(
        &self,
        group: &str,
        topic_name: &str,
        offset: u64,
    ) -> ResultMqttBrokerError {
        let key = format!("{group}_{}", topic_name);
        self.offset_cache.insert(key, offset);
        self.message_storage
            .commit_group_offset(group, topic_name, offset)
            .await?;
        Ok(())
    }

    async fn get_offset(&self, group: &str, topic_name: &str) -> Result<u64, MqttBrokerError> {
        let key = format!("{group}_{}", topic_name);
        if let Some(offset) = self.offset_cache.get(&key) {
            return Ok(*offset);
        }

        let offset = self
            .message_storage
            .get_group_offset(group, topic_name)
            .await?;
        self.offset_cache.insert(key, offset);
        Ok(offset)
    }

    fn record_metrics(
        &self,
        client_id: &str,
        path: &str,
        topic_name: &str,
        data_size: u64,
        success: bool,
    ) {
        record_subscribe_bytes_sent(client_id, path, data_size, success);
        record_subscribe_topic_bytes_sent(client_id, path, topic_name, data_size, success);

        record_subscribe_messages_sent(client_id, path, true);
        record_subscribe_topic_messages_sent(client_id, path, topic_name, true);
    }
}

pub fn directly_group_name(client_id: &str, path: &str, topic_name: &str) -> String {
    format!("directly_sub_{client_id}_{path}_{topic_name}")
}
