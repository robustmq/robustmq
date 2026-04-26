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
use tracing::{debug, error, info, warn};

pub struct QueuePushManager {
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    client_pool: Arc<ClientPool>,
    tenant: String,
    group_name: String,
    subject: String,
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
        subject: String,
    ) -> Self {
        println!("QueuePushManager.subject:{}", subject);
        QueuePushManager {
            subscribe_manager,
            connection_manager,
            storage_driver_manager,
            client_pool,
            tenant,
            group_name,
            subject,
            consumer: None,
            round_robin: 0,
        }
    }

    fn queue_key(&self) -> String {
        format!("{}#{}#{}", self.tenant, self.group_name, self.subject)
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
                        Err(NatsBrokerError::QueueGroupEmpty(_)) => {
                            info!("{} queue group empty, exiting.", label);
                            break;
                        }
                        Err(e) => {
                            error!("{} error: {}", label, e);
                            adaptive_sleep(0).await;
                        }
                    }
                }
            }
        }
    }

    fn active_subscribers(&self, queue_key: &str) -> Option<Vec<NatsSubscriber>> {
        let bucket_mgr = self.subscribe_manager.nats_core_queue_push.get(queue_key)?;
        if bucket_mgr.buckets_data_list.is_empty() {
            return None;
        }
        Some(
            bucket_mgr
                .buckets_data_list
                .iter()
                .flat_map(|b| {
                    b.value()
                        .iter()
                        .map(|s| s.value().clone())
                        .collect::<Vec<_>>()
                })
                .filter(|s| self.subscribe_manager.allow_push_client(s.connect_id))
                .collect(),
        )
    }

    async fn send_messages(&mut self) -> Result<usize, NatsBrokerError> {
        let queue_key = self.queue_key();
        let Some(subs) = self.active_subscribers(&queue_key) else {
            return Err(NatsBrokerError::QueueGroupEmpty(queue_key));
        };
        println!("subs:{:?}", subs.len());
        if subs.is_empty() {
            return Ok(0);
        }

        let read_config = AdapterReadConfig {
            max_record_num: BATCH_SIZE,
            max_size: 1024 * 1024 * 30,
        };

        if self.consumer.is_none() {
            self.consumer = Some(GroupConsumer::new_manual(
                self.storage_driver_manager.clone(),
                queue_key.clone(),
            ));
        }
        let consumer = self.consumer.as_ref().unwrap();

        let records = consumer
            .next_messages(&self.tenant, &self.subject, &read_config)
            .await
            .map_err(NatsBrokerError::from)?;

        // update last_pull_time on every pull attempt regardless of result
        if let Some(info) = self
            .subscribe_manager
            .nats_core_queue_push_thread
            .get(&queue_key)
        {
            info.last_pull_time
                .lock()
                .map(|mut t| *t = common_base::tools::now_second())
                .ok();
        }

        if records.is_empty() {
            return Ok(0);
        }

        let mut pushed = 0;
        let mut all_delivered = true;

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
                    all_delivered = false;
                    warn!(
                        "NATS queue [{}]: no subscriber available, will retry",
                        queue_key
                    );
                }
                Err(e) => warn!("NATS queue send error [{}]: {}", queue_key, e),
            }
        }

        if all_delivered {
            consumer.commit().await.map_err(NatsBrokerError::from)?;
        }

        if pushed > 0 {
            if let Some(info) = self
                .subscribe_manager
                .nats_core_queue_push_thread
                .get(&queue_key)
            {
                info.record_push(pushed as u64);
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
