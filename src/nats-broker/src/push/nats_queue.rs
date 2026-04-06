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

use crate::core::error::NatsBrokerError;
use crate::core::tenant::get_tenant;
use crate::push::common::{adaptive_sleep, should_stop, BATCH_SIZE};
use crate::push::nats_fanout::send_packet;
use crate::push::manager::NatsSubscribeManager;
use crate::push::manager::NatsSubscriber;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use storage_adapter::consumer::GroupConsumer;
use storage_adapter::driver::StorageDriverManager;
use tokio::select;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

pub struct QueuePushManager {
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    queue_key: String,
    topic_name: String,
    consumer: Option<GroupConsumer>,
    round_robin: Arc<AtomicU64>,
}

impl QueuePushManager {
    pub fn new(
        subscribe_manager: Arc<NatsSubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        queue_key: String,
    ) -> Self {
        let topic_name = queue_key
            .split_once('#')
            .map(|(t, _)| t.to_string())
            .unwrap_or_else(|| queue_key.clone());

        QueuePushManager {
            subscribe_manager,
            connection_manager,
            storage_driver_manager,
            queue_key,
            topic_name,
            consumer: None,
            round_robin: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn start(&mut self, stop_sx: &broadcast::Sender<bool>) {
        let mut stop_rx = stop_sx.subscribe();
        let label = format!("NATS QueuePushManager[{}]", self.queue_key);
        loop {
            select! {
                val = stop_rx.recv() => {
                    if should_stop(val, &label) { break; }
                }
                res = self.send_messages() => {
                    match res {
                        Ok(count) => adaptive_sleep(count).await,
                        Err(e) => {
                            error!("{} error: {}", label, e);
                            adaptive_sleep(0).await;
                        }
                    }
                }
            }
        }
    }

    async fn send_messages(&mut self) -> Result<usize, NatsBrokerError> {
        let is_empty = self
            .subscribe_manager
            .nats_core_queue_push
            .get(&self.queue_key)
            .map(|b| b.sub_len() == 0)
            .unwrap_or(true);

        if is_empty {
            return Ok(0);
        }

        let tenant = self
            .subscribe_manager
            .nats_core_queue_push
            .get(&self.queue_key)
            .and_then(|b| {
                b.buckets_data_list
                    .iter()
                    .next()
                    .and_then(|bucket| bucket.iter().next().map(|e| e.tenant.clone()))
            })
            .unwrap_or_else(get_tenant);
        let read_config = AdapterReadConfig {
            max_record_num: BATCH_SIZE,
            max_size: 1024 * 1024 * 30,
        };

        let consumer = self.consumer.get_or_insert_with(|| {
            GroupConsumer::new_manual(self.storage_driver_manager.clone(), self.queue_key.clone())
        });

        let records = consumer
            .next_messages(&tenant, &self.topic_name, &read_config)
            .await
            .map_err(NatsBrokerError::from)?;

        if records.is_empty() {
            return Ok(0);
        }

        let mut pushed = 0;
        let mut all_delivered = true;
        for record in &records {
            match self.round_robin_send(record).await {
                Ok(true) => pushed += 1,
                Ok(false) => {
                    all_delivered = false;
                    warn!(
                        "NATS queue [{}]: no subscriber available for record, will retry",
                        self.queue_key
                    );
                }
                Err(e) => warn!("NATS queue send error [{}]: {}", self.queue_key, e),
            }
        }

        if all_delivered {
            self.consumer
                .as_mut()
                .unwrap()
                .commit()
                .await
                .map_err(NatsBrokerError::from)?;
        }

        Ok(pushed)
    }

    async fn round_robin_send(&self, record: &StorageRecord) -> Result<bool, NatsBrokerError> {
        let subscribers: Vec<NatsSubscriber> = {
            let Some(bucket_mgr) = self
                .subscribe_manager
                .nats_core_queue_push
                .get(&self.queue_key)
            else {
                return Ok(false);
            };
            let Some(bucket) = bucket_mgr.buckets_data_list.get(&self.queue_key) else {
                return Ok(false);
            };
            if bucket.is_empty() {
                return Ok(false);
            }
            let mut list: Vec<NatsSubscriber> = bucket.iter().map(|e| e.value().clone()).collect();
            list.sort_unstable_by(|a, b| a.connect_id.cmp(&b.connect_id).then(a.sid.cmp(&b.sid)));
            list
        };

        let len = subscribers.len();
        let start_idx = self.round_robin.fetch_add(1, Ordering::Relaxed) as usize % len;

        for i in 0..len {
            let subscriber = &subscribers[(start_idx + i) % len];

            if !self
                .subscribe_manager
                .allow_push_client(subscriber.connect_id)
            {
                continue;
            }

            match send_packet(&self.connection_manager, subscriber, record).await {
                Ok(true) => return Ok(true),
                Ok(false) => {}
                Err(NatsBrokerError::ConnectionNotFound(_)) => {
                    warn!(
                        "NATS queue subscriber gone: connect_id={} sid={}",
                        subscriber.connect_id, subscriber.sid
                    );
                    self.subscribe_manager
                        .remove_push_by_sid(subscriber.connect_id, &subscriber.sid);
                    self.subscribe_manager
                        .add_not_push_client(subscriber.connect_id);
                }
                Err(e) => {
                    debug!(
                        "NATS queue send failed [connect_id={}, sid={}]: {}",
                        subscriber.connect_id, subscriber.sid, e
                    );
                    self.subscribe_manager
                        .add_not_push_client(subscriber.connect_id);
                }
            }
        }

        Ok(false)
    }
}
