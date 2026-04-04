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
use crate::subscribe::directly_push::send_packet;
use crate::subscribe::{NatsSubscribeManager, NatsSubscriber};
use dashmap::DashMap;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::consumer::GroupConsumer;
use storage_adapter::driver::StorageDriverManager;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

const BATCH_SIZE: u64 = 500;
const IDLE_SLEEP_MS: u64 = 100;
const LOW_LOAD_SLEEP_MS: u64 = 50;
const HIGH_LOAD_SLEEP_MS: u64 = 10;
const LOW_LOAD_THRESHOLD: usize = 10;

pub struct QueuePushManager {
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    // queue_key ({topic}#{group}) → GroupConsumer
    consumers: DashMap<String, GroupConsumer>,
}

impl QueuePushManager {
    pub fn new(
        subscribe_manager: Arc<NatsSubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
    ) -> Self {
        QueuePushManager {
            subscribe_manager,
            connection_manager,
            storage_driver_manager,
            consumers: DashMap::with_capacity(16),
        }
    }

    pub async fn start(&self, stop_sx: &broadcast::Sender<bool>) {
        let mut stop_rx = stop_sx.subscribe();
        loop {
            select! {
                val = stop_rx.recv() => {
                    match val {
                        Ok(true) => {
                            info!("NATS QueuePushManager stopped");
                            break;
                        }
                        Ok(false) => {}
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("NATS QueuePushManager stop channel closed");
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            debug!("NATS QueuePushManager stop channel lagged, skipped {}", n);
                        }
                    }
                }
                res = self.push_all() => {
                    match res {
                        Ok(count) => {
                            if count == 0 {
                                sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                            } else if count < LOW_LOAD_THRESHOLD {
                                sleep(Duration::from_millis(LOW_LOAD_SLEEP_MS)).await;
                            } else {
                                sleep(Duration::from_millis(HIGH_LOAD_SLEEP_MS)).await;
                            }
                        }
                        Err(e) => {
                            error!("NATS QueuePushManager push error: {}", e);
                            sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                        }
                    }
                }
            }
        }
    }

    async fn push_all(&self) -> Result<usize, NatsBrokerError> {
        let mut total = 0;

        // Collect queue keys to avoid holding DashMap locks across awaits.
        let queue_keys: Vec<String> = self
            .subscribe_manager
            .queue_push
            .iter()
            .map(|e| e.key().clone())
            .collect();

        for queue_key in queue_keys {
            match self.push_queue_group(&queue_key).await {
                Ok(count) => total += count,
                Err(e) => {
                    error!("NATS queue push error [key={}]: {}", queue_key, e);
                    sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                }
            }
        }

        Ok(total)
    }

    // One GroupConsumer per queue_key reads messages, then round-robin picks a subscriber.
    async fn push_queue_group(&self, queue_key: &str) -> Result<usize, NatsBrokerError> {
        let is_empty = self
            .subscribe_manager
            .queue_push
            .get(queue_key)
            .map(|g| g.subscribers.is_empty())
            .unwrap_or(true);

        if is_empty {
            return Ok(0);
        }

        // Derive topic_name from queue_key: "{topic_name}#{queue_group}"
        let topic_name = queue_key
            .split_once('#')
            .map(|(t, _)| t)
            .unwrap_or(queue_key)
            .to_string();

        let tenant = get_tenant();
        let read_config = AdapterReadConfig {
            max_record_num: BATCH_SIZE,
            max_size: 1024 * 1024 * 30,
        };

        // Use queue_key as group_name so all members share a single offset.
        let mut consumer_entry = self
            .consumers
            .entry(queue_key.to_string())
            .or_insert_with(|| {
                GroupConsumer::new_manual(
                    self.storage_driver_manager.clone(),
                    queue_key.to_string(),
                )
            });

        let records = consumer_entry
            .next_messages(&tenant, &topic_name, &read_config)
            .await
            .map_err(NatsBrokerError::from)?;

        if records.is_empty() {
            return Ok(0);
        }

        let mut pushed = 0;
        let mut stale_seqs: Vec<u64> = Vec::new();

        for record in &records {
            match self
                .round_robin_send(queue_key, &topic_name, record, &mut stale_seqs)
                .await
            {
                Ok(true) => pushed += 1,
                Ok(false) => {}
                Err(e) => {
                    warn!("NATS queue send error [key={}]: {}", queue_key, e);
                }
            }
        }

        consumer_entry
            .commit()
            .await
            .map_err(NatsBrokerError::from)?;
        drop(consumer_entry);

        for seq in stale_seqs {
            self.subscribe_manager
                .remove_queue_subscriber(queue_key, seq);
        }

        Ok(pushed)
    }

    // Pick one subscriber via round-robin and send the record to it.
    async fn round_robin_send(
        &self,
        queue_key: &str,
        topic_name: &str,
        record: &StorageRecord,
        stale_seqs: &mut Vec<u64>,
    ) -> Result<bool, NatsBrokerError> {
        // Collect (seq, subscriber) while holding the lock briefly, then release.
        let (seqs, idx) = {
            let Some(group) = self.subscribe_manager.queue_push.get(queue_key) else {
                return Ok(false);
            };
            let len = group.subscribers.len() as u64;
            if len == 0 {
                return Ok(false);
            }
            let mut seqs: Vec<(u64, NatsSubscriber)> = group
                .subscribers
                .iter()
                .map(|e| (*e.key(), e.value().clone()))
                .collect();
            seqs.sort_unstable_by_key(|(seq, _)| *seq);
            let idx = (group.round_robin.fetch_add(1, Ordering::Relaxed) % len) as usize;
            (seqs, idx)
        };

        // Try each subscriber starting at idx, skipping unavailable ones.
        for i in 0..seqs.len() {
            let (seq, ref subscriber) = seqs[(idx + i) % seqs.len()];

            if !self
                .subscribe_manager
                .allow_push_client(subscriber.connect_id, topic_name)
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
                    stale_seqs.push(seq);
                    self.subscribe_manager
                        .add_not_push_client(subscriber.connect_id, topic_name);
                }
                Err(e) => {
                    debug!(
                        "NATS queue send failed [connect_id={}, sid={}]: {}",
                        subscriber.connect_id, subscriber.sid, e
                    );
                    self.subscribe_manager
                        .add_not_push_client(subscriber.connect_id, topic_name);
                }
            }
        }

        Ok(false)
    }
}
