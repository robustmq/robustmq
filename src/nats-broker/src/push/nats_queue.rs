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
use crate::push::common::{adaptive_sleep, should_stop, BATCH_SIZE};
use crate::push::manager::NatsSubscribeManager;
use crate::push::nats_fanout::send_packet;
use grpc_clients::pool::ClientPool;
use metadata_struct::nats::subscriber::NatsSubscriber;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
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
    client_pool: Arc<ClientPool>,
    tenant: String,
    group_name: String,
    consumer: Option<GroupConsumer>,
    round_robin: u64,
}

impl QueuePushManager {
    pub fn new(
        subscribe_manager: Arc<NatsSubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        client_pool: Arc<ClientPool>,
        tenant: String,
        group_name: String,
    ) -> Self {
        QueuePushManager {
            subscribe_manager,
            connection_manager,
            storage_driver_manager,
            client_pool,
            tenant,
            group_name,
            consumer: None,
            round_robin: 0,
        }
    }

    fn queue_key(&self) -> String {
        format!("{}#{}", self.tenant, self.group_name)
    }

    pub async fn start(&mut self, stop_sx: &broadcast::Sender<bool>) {
        let mut stop_rx = stop_sx.subscribe();
        let label = format!("NATS QueuePushManager[{}]", self.queue_key());
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
        // Collect subscribers grouped by subject from all buckets in this group.
        let subjects: Vec<(String, Vec<NatsSubscriber>)> = {
            let Some(bucket_mgr) = self
                .subscribe_manager
                .nats_core_queue_push
                .get(&self.queue_key())
            else {
                return Ok(0);
            };
            if bucket_mgr.sub_len() == 0 {
                return Ok(0);
            }
            let mut by_subject: std::collections::HashMap<String, Vec<NatsSubscriber>> =
                std::collections::HashMap::new();
            for bucket in bucket_mgr.buckets_data_list.iter() {
                for sub in bucket.value().iter() {
                    by_subject
                        .entry(sub.subject.clone())
                        .or_default()
                        .push(sub.value().clone());
                }
            }
            by_subject.into_iter().collect()
        };

        if subjects.is_empty() {
            return Ok(0);
        }

        let read_config = AdapterReadConfig {
            max_record_num: BATCH_SIZE,
            max_size: 1024 * 1024 * 30,
        };

        // Lazily init consumer; kept across calls for offset continuity.
        let queue_key = self.queue_key();
        if self.consumer.is_none() {
            self.consumer = Some(GroupConsumer::new_manual(
                self.storage_driver_manager.clone(),
                queue_key,
            ));
        }
        let consumer = self.consumer.as_ref().unwrap();

        let mut pushed = 0;

        for (topic_name, subs) in subjects {
            let records = consumer
                .next_messages(&self.tenant, &topic_name, &read_config)
                .await
                .map_err(NatsBrokerError::from)?;

            if records.is_empty() {
                continue;
            }

            let mut topic_all_delivered = true;
            for record in &records {
                let start_idx = self.round_robin as usize % subs.len();
                self.round_robin = self.round_robin.wrapping_add(1);
                match round_robin_send(
                    record,
                    &subs,
                    start_idx,
                    &self.subscribe_manager,
                    &self.connection_manager,
                    &self.client_pool,
                )
                .await
                {
                    Ok(true) => pushed += 1,
                    Ok(false) => {
                        topic_all_delivered = false;
                        warn!(
                            "NATS queue [{}] topic {}: no subscriber available, will retry",
                            self.queue_key(),
                            topic_name
                        );
                    }
                    Err(e) => warn!(
                        "NATS queue send error [{}] topic {}: {}",
                        self.queue_key(),
                        topic_name,
                        e
                    ),
                }
            }

            // Commit per-topic: only advance offset when all records in this topic were delivered.
            if topic_all_delivered {
                consumer.commit().await.map_err(NatsBrokerError::from)?;
            }
        }

        Ok(pushed)
    }
}

async fn round_robin_send(
    record: &StorageRecord,
    subscribers: &[NatsSubscriber],
    start_idx: usize,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    connection_manager: &Arc<ConnectionManager>,
    _client_pool: &Arc<ClientPool>,
) -> Result<bool, NatsBrokerError> {
    for i in 0..subscribers.len() {
        let subscriber = &subscribers[(start_idx + i) % subscribers.len()];

        if !subscribe_manager.allow_push_client(subscriber.connect_id) {
            continue;
        }

        match send_packet(connection_manager, subscriber, record).await {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(NatsBrokerError::ConnectionNotFound(_)) => {
                warn!(
                    "NATS queue subscriber gone: connect_id={} sid={}",
                    subscriber.connect_id, subscriber.sid
                );
                subscribe_manager.add_not_push_client(subscriber.connect_id);
            }
            Err(e) => {
                debug!(
                    "NATS queue send failed [connect_id={}, sid={}]: {}",
                    subscriber.connect_id, subscriber.sid, e
                );
                subscribe_manager.add_not_push_client(subscriber.connect_id);
            }
        }
    }

    Ok(false)
}
