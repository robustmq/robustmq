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

use std::collections::HashMap;
use std::sync::Arc;

use broker_core::cluster::ClusterStorage;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::meta::status::MetaStatus;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::{error, warn};

use crate::core::unacked_index;

const SCAN_INTERVAL_MS: u64 = 60_000;
const STALE_THRESHOLD_SECS: u64 = 300;
const METADATA_SHARD_NAME: &str = "metadata_0";

pub(crate) async fn requeue_message(
    sdm: &Arc<StorageDriverManager>,
    tenant: &str,
    queue: &str,
    offset: u64,
    index_offset: u64,
) -> Result<(), CommonError> {
    let Some(topic) = sdm.broker_cache.get_topic_by_name(tenant, queue) else {
        warn!(
            "AMQP requeue: queue {} no longer exists, dropping index entry",
            queue
        );
        return unacked_index::delete_entry(sdm, index_offset).await;
    };
    let Some(shard_name) = topic.storage_name_list.get(&0).cloned() else {
        warn!(
            "AMQP requeue: queue {} has no shard, dropping index entry",
            queue
        );
        return unacked_index::delete_entry(sdm, index_offset).await;
    };

    let read_config = AdapterReadConfig::new();
    let mut offsets = HashMap::new();
    offsets.insert(shard_name, offset);
    let records = sdm
        .read_by_offset(tenant, queue, &offsets, &read_config)
        .await?;
    let Some(record) = records.into_iter().next() else {
        return unacked_index::delete_entry(sdm, index_offset).await;
    };

    let mut protocol_data = record.protocol_data.unwrap_or_default();
    let mut amqp = protocol_data.amqp.unwrap_or_default();
    amqp.redelivered = true;
    protocol_data.amqp = Some(amqp);

    let new_record = AdapterWriteRecord::new(queue.to_string(), record.data.to_vec())
        .with_protocol_data(Some(protocol_data));
    sdm.write(tenant, queue, std::slice::from_ref(&new_record), 1)
        .await?;

    sdm.delete_by_offsets(tenant, queue, &[offset]).await?;

    unacked_index::delete_entry(sdm, index_offset).await
}

#[derive(Clone)]
pub struct AmqpRecoveryScanner {
    client_pool: Arc<ClientPool>,
    storage_driver_manager: Arc<StorageDriverManager>,
}

impl AmqpRecoveryScanner {
    pub fn new(
        client_pool: Arc<ClientPool>,
        storage_driver_manager: Arc<StorageDriverManager>,
    ) -> Self {
        AmqpRecoveryScanner {
            client_pool,
            storage_driver_manager,
        }
    }

    pub async fn start(&self, stop_send: &broadcast::Sender<bool>) {
        let ac_fn = async || -> ResultCommonError { self.tick().await };
        loop_select_ticket(ac_fn, SCAN_INTERVAL_MS, stop_send).await;
    }

    async fn tick(&self) -> ResultCommonError {
        if !self.is_meta_leader().await {
            return Ok(());
        }

        let entries = unacked_index::scan_all(&self.storage_driver_manager).await?;
        let now = now_second();
        for (index_offset, entry) in entries {
            if now.saturating_sub(entry.timestamp) < STALE_THRESHOLD_SECS {
                continue;
            }
            if let Err(e) = requeue_message(
                &self.storage_driver_manager,
                &entry.tenant,
                &entry.queue,
                entry.offset,
                index_offset,
            )
            .await
            {
                error!(
                    "AMQP recovery scanner: failed to requeue message from {}: {}",
                    entry.queue, e
                );
            }
        }
        Ok(())
    }

    async fn is_meta_leader(&self) -> bool {
        let cluster_storage = ClusterStorage::new(self.client_pool.clone());
        let content = match cluster_storage.meta_cluster_status().await {
            Ok(content) => content,
            Err(e) => {
                warn!(
                    "AMQP recovery scanner: failed to read cluster status: {}",
                    e
                );
                return false;
            }
        };
        let status: HashMap<String, MetaStatus> = match serde_json::from_str(&content) {
            Ok(status) => status,
            Err(e) => {
                warn!(
                    "AMQP recovery scanner: failed to parse cluster status: {}",
                    e
                );
                return false;
            }
        };
        status
            .get(METADATA_SHARD_NAME)
            .and_then(|s| s.current_leader)
            .map(|leader_id| leader_id == broker_config().broker_id)
            .unwrap_or(false)
    }
}
