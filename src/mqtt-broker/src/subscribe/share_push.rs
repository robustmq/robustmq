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
use crate::subscribe::push::{
    send_message_validator_by_max_message_size, send_message_validator_by_message_expire,
};
use crate::{handler::cache::MQTTCacheManager, storage::message::MessageStorage};
use crate::{
    handler::tool::ResultMqttBrokerError,
    handler::{error::MqttBrokerError, sub_slow::record_slow_subscribe_data},
    subscribe::{
        common::{client_unavailable_error, Subscriber},
        manager::SubscribeManager,
        push::{build_publish_message, send_publish_packet_to_client},
    },
};
use common_base::tools::now_second;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::mqtt::message::MqttMessage;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::atomic::AtomicU64;
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast::Sender, time::sleep};
use tracing::{debug, error, info};

const BATCH_SIZE: u64 = 500;

const IDLE_SLEEP_MS: u64 = 100;
const LOW_LOAD_SLEEP_MS: u64 = 50;
const HIGH_LOAD_SLEEP_MS: u64 = 10;
const LOW_LOAD_THRESHOLD: u64 = 10;

pub struct SharePushManager {
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<MQTTCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    message_storage: MessageStorage,
    group_name: String,
    seq: AtomicU64,
}

impl SharePushManager {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<MQTTCacheManager>,
        storage_adapter: ArcStorageAdapter,
        connection_manager: Arc<ConnectionManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        group_name: String,
    ) -> Self {
        SharePushManager {
            subscribe_manager,
            message_storage: MessageStorage::new(storage_adapter),
            cache_manager,
            rocksdb_engine_handler,
            connection_manager,
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
                    if let Ok(flag) = val {
                        if flag {
                            info!("SharePushManager[{}] stopped", self.group_name);
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
                            error!("SharePushManager[{}] send messages failed: {}", self.group_name, e);
                            sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn send_messages(&self, stop_sx: &Sender<bool>) -> Result<u64, MqttBrokerError> {
        let topic_list = if let Some(topic_list) = self
            .subscribe_manager
            .share_group_topics
            .get(&self.group_name)
        {
            topic_list
        } else {
            return Ok(0);
        };

        if topic_list.is_empty() {
            return Ok(0);
        }

        let buckets = if let Some(data) = self.subscribe_manager.share_push.get(&self.group_name) {
            data.clone()
        } else {
            return Ok(0);
        };

        let seqs = buckets.get_sub_client_seqs(&self.group_name);
        if seqs.is_empty() {
            return Ok(0);
        }

        let mut processed_count = 0;
        for topic in topic_list.iter() {
            let data_list = self.next_message(&self.group_name, topic).await?;
            if data_list.is_empty() {
                continue;
            }

            let mut last_commit_offset: Option<u64> = None;
            for record in data_list {
                let record_offset = record.pkid;

                let msg = MqttMessage::decode_record(record.clone())?;

                // If the message has expired, it will not be delivered.
                if !send_message_validator_by_message_expire(&msg) {
                    last_commit_offset = Some(record_offset);
                    continue;
                }

                let max_attempts = seqs.len();
                let mut attempts = 0;
                let mut break_flag = false;
                loop {
                    if attempts >= max_attempts {
                        debug!(
                            "Failed to push message after {} attempts for group {}, skipping record at pkid {}",
                            attempts, self.group_name, record.pkid
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
                            .allow_push_client(&subscriber.client_id)
                        {
                            continue;
                        }

                        // If the message exceeds the size limit specified for the entire client ID, try the next client.
                        let allow = match send_message_validator_by_max_message_size(
                            &self.cache_manager,
                            &subscriber.client_id,
                            &msg,
                        )
                        .await
                        {
                            Ok(allow) => allow,
                            Err(e) => {
                                if !client_unavailable_error(&e) {
                                    self.subscribe_manager
                                        .add_not_push_client(&subscriber.client_id);
                                }
                                continue;
                            }
                        };

                        if !allow {
                            continue;
                        }

                        // If the message cannot be sent to the entire client group, try the next client.
                        if !is_send_msg_by_bo_local(
                            subscriber.no_local,
                            &subscriber.client_id,
                            &msg.client_id,
                        ) {
                            continue;
                        }

                        // Push data
                        let success = match self.push_data(&subscriber, &msg, stop_sx).await {
                            Ok(pushed) => {
                                if pushed {
                                    processed_count += 1;
                                }
                                last_commit_offset = Some(record_offset);
                                pushed
                            }
                            Err(e) => {
                                if !client_unavailable_error(&e) {
                                    self.subscribe_manager
                                        .add_not_push_client(&subscriber.client_id);
                                }

                                continue;
                            }
                        };

                        record_sub_send_metrics(
                            &subscriber.client_id,
                            &subscriber.sub_path,
                            &subscriber.topic_name,
                            record.size() as u64,
                            success,
                        );
                        break;
                    }
                }

                if break_flag {
                    break;
                }
            }

            if let Some(offset) = last_commit_offset {
                if let Err(e) = self.commit_offset(&self.group_name, topic, offset).await {
                    error!(
                        "Failed to commit offset for subscriber [group: {}, topic: {}, offset: {}]: {}. Messages may be redelivered on next poll",
                        &self.group_name, topic, offset, e
                    );
                } else {
                    debug!(
                        "Committed offset {} for share group [group: {}, topic: {}], total processed: {} messages",
                        offset, self.group_name, topic, processed_count
                    );
                }
            }
        }

        Ok(processed_count)
    }

    async fn push_data(
        &self,
        subscriber: &Subscriber,
        msg: &MqttMessage,
        stop_sx: &Sender<bool>,
    ) -> Result<bool, MqttBrokerError> {
        let sub_pub_param = if let Some(params) = build_publish_message(
            &self.cache_manager,
            &self.connection_manager,
            msg,
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
            msg.create_time,
        )
        .await?;

        Ok(true)
    }

    async fn next_message(
        &self,
        group: &str,
        topic_name: &str,
    ) -> Result<Vec<AdapterWriteRecord>, MqttBrokerError> {
        let offset = self
            .message_storage
            .get_group_offset(group, topic_name)
            .await?;

        Ok(self
            .message_storage
            .read_topic_message(topic_name, offset, BATCH_SIZE)
            .await?)
    }

    async fn commit_offset(
        &self,
        group: &str,
        topic_name: &str,
        offset: u64,
    ) -> ResultMqttBrokerError {
        self.message_storage
            .commit_group_offset(group, topic_name, offset + 1)
            .await?;
        Ok(())
    }
}
