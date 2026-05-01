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

use crate::core::cache::MQTTCacheManager;
use crate::core::error::MqttBrokerError;
use crate::core::sub_option::message_is_same_client;
use crate::subscribe::buckets::BucketsManager;
use crate::subscribe::common::{
    client_unavailable_error, message_is_exceeds_max_message_size, message_is_expire,
    record_sub_send_metrics, stale_subscriber_error, Subscriber,
};
use crate::subscribe::manager::{share_push_key, SubscribeManager};
use crate::subscribe::push::{adaptive_sleep, handle_stop_signal, push_data, BATCH_SIZE};
use metadata_struct::storage::{adapter_read_config::AdapterReadConfig, record::StorageRecord};
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use storage_adapter::{consumer::GroupConsumer, driver::StorageDriverManager};
use tokio::{select, sync::broadcast::Sender};
use tracing::{debug, error, info};

/// Manages message push for a single (tenant, group_name, topic_name) triple.
///
/// Each shared-subscription group+topic combination gets its own `SharePushManager`
/// instance, ensuring subscribers for different topics within the same group are
/// never mixed during dispatch.
pub struct SharePushManager {
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<MQTTCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    consumer: GroupConsumer,
    tenant: String,
    group_name: String,
    topic_name: String,
    /// share_push inner-map key: "group_name/topic_name"
    share_key: String,
    seq: AtomicU64,
}

impl SharePushManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<MQTTCacheManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        connection_manager: Arc<ConnectionManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        tenant: String,
        group_name: String,
        topic_name: String,
    ) -> Self {
        let share_key = share_push_key(&group_name, &topic_name);
        SharePushManager {
            subscribe_manager,
            consumer: GroupConsumer::new_manual(storage_driver_manager, group_name.clone()),
            cache_manager,
            rocksdb_engine_handler,
            connection_manager,
            tenant,
            topic_name,
            share_key,
            group_name,
            seq: AtomicU64::new(0),
        }
    }

    pub async fn start(&mut self, stop_sx: &Sender<bool>) {
        let label = format!("SharePushManager[{}/{}]", self.group_name, self.topic_name);
        info!("{} started", label);
        let mut stop_rx = stop_sx.subscribe();
        loop {
            select! {
                val = stop_rx.recv() => {
                    if handle_stop_signal(val, &label) {
                        break;
                    }
                }
                res = self.send_messages(stop_sx) => {
                    match res {
                        Ok(processed_count) => {
                            adaptive_sleep(processed_count).await;
                        }
                        Err(e) => {
                            if stale_subscriber_error(&e) {
                                info!("{} stopping: topic no longer exists ({})", label, e);
                                break;
                            }
                            error!("{} send messages failed: {}", label, e);
                            adaptive_sleep(0).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn send_messages(&mut self, stop_sx: &Sender<bool>) -> Result<u64, MqttBrokerError> {
        let Some(buckets) = self
            .subscribe_manager
            .share_push
            .get(&self.tenant)
            .and_then(|t| t.get(&self.share_key).map(|v| v.clone()))
        else {
            return Ok(0);
        };

        let seqs = buckets.get_sub_client_seqs(&self.share_key);
        if seqs.is_empty() {
            return Ok(0);
        }

        self.process_topic_messages(&buckets, &seqs, stop_sx).await
    }

    async fn process_topic_messages(
        &mut self,
        buckets: &Arc<BucketsManager>,
        seqs: &[u64],
        stop_sx: &Sender<bool>,
    ) -> Result<u64, MqttBrokerError> {
        let read_config = AdapterReadConfig {
            max_record_num: BATCH_SIZE,
            max_size: 1024 * 1024 * 30,
        };

        let tenant = self.tenant.as_str();
        let topic_name = self.topic_name.as_str();
        let data_list = self
            .consumer
            .next_messages(tenant, topic_name, &read_config)
            .await?;

        if data_list.is_empty() {
            return Ok(0);
        }

        let mut processed_count = 0;

        for record in data_list {
            if message_is_expire(&record) {
                continue;
            }

            if !self
                .dispatch_record_to_group(&record, buckets, seqs, stop_sx)
                .await?
            {
                // No subscriber could accept the message. Stop processing the batch and
                // skip commit so the same batch is re-read on the next iteration.
                // (pending_offsets is shard-level max: committing here would silently
                // skip every later record in this batch on restart.)
                debug!(
                    "Failed to push message after {} attempts for group {}/{}, stopping batch",
                    seqs.len(),
                    self.group_name,
                    self.topic_name
                );
                return Ok(processed_count);
            }

            processed_count += 1;
        }

        self.consumer.commit().await?;
        Ok(processed_count)
    }

    async fn dispatch_record_to_group(
        &self,
        record: &StorageRecord,
        buckets: &Arc<BucketsManager>,
        seqs: &[u64],
        stop_sx: &Sender<bool>,
    ) -> Result<bool, MqttBrokerError> {
        for _ in 0..seqs.len() {
            let row_seq = self.seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            // Map the monotonic counter to a position in the seqs slice, then use the
            // actual seq value stored there — seqs are global AtomicU64 values, not 0-based indices.
            let actual_seq = seqs[(row_seq % seqs.len() as u64) as usize];

            let Some(subscriber) = buckets.get_subscribe_by_key_seq(&self.share_key, actual_seq)
            else {
                continue;
            };

            if self
                .try_push_to_subscriber(record, &subscriber, stop_sx)
                .await?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn try_push_to_subscriber(
        &self,
        record: &StorageRecord,
        subscriber: &Subscriber,
        stop_sx: &Sender<bool>,
    ) -> Result<bool, MqttBrokerError> {
        if !self
            .subscribe_manager
            .allow_push_client(&subscriber.tenant, &subscriber.client_id)
        {
            return Ok(false);
        }

        let exceeds =
            message_is_exceeds_max_message_size(&self.cache_manager, &subscriber.client_id, record)
                .await;
        match exceeds {
            Ok(false) => {}
            Ok(true) => return Ok(false),
            Err(e) => {
                if !client_unavailable_error(&e) {
                    self.subscribe_manager
                        .add_not_push_client(&subscriber.tenant, &subscriber.client_id);
                }
                return Ok(false);
            }
        }

        if message_is_same_client(subscriber, record) {
            return Ok(false);
        }

        if let Err(e) = push_data(
            &self.connection_manager,
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            subscriber,
            record,
            stop_sx,
        )
        .await
        {
            if !client_unavailable_error(&e) {
                self.subscribe_manager
                    .add_not_push_client(&subscriber.tenant, &subscriber.client_id);
            }
            return Ok(false);
        }

        record_sub_send_metrics(
            &subscriber.tenant,
            &subscriber.client_id,
            &subscriber.sub_path,
            &subscriber.topic_name,
            0,
            true,
        );

        Ok(true)
    }
}
