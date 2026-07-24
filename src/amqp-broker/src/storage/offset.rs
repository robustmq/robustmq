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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::meta::common::call::{get_offset_data, save_offset_data};
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;
use protocol::meta::meta_service_common::{
    GetOffsetDataRequest, SaveOffsetData, SaveOffsetDataRequest, SaveOffsetDataRequestOffset,
};
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::core::unacked_index;

pub struct OffsetStorage {
    client_pool: Arc<ClientPool>,
}

impl OffsetStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        OffsetStorage { client_pool }
    }

    fn group_name(queue_name: &str) -> String {
        format!("amqp:{queue_name}")
    }

    pub async fn read_committed_offset(
        &self,
        tenant: &str,
        queue_name: &str,
        shard_name: &str,
    ) -> Result<u64, CommonError> {
        let config = broker_config();
        let request = GetOffsetDataRequest {
            tenant: tenant.to_string(),
            group: Self::group_name(queue_name),
        };
        let reply =
            get_offset_data(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(reply
            .offsets
            .iter()
            .find(|o| o.shard_name == shard_name)
            .map(|o| o.offset)
            .unwrap_or(0))
    }

    pub async fn commit_offset_cas(
        &self,
        tenant: &str,
        queue_name: &str,
        shard_name: &str,
        expected: u64,
        new_offset: u64,
    ) -> Result<bool, CommonError> {
        let config = broker_config();
        let request = SaveOffsetDataRequest {
            offsets: vec![SaveOffsetData {
                tenant: tenant.to_string(),
                group: Self::group_name(queue_name),
                offsets: vec![SaveOffsetDataRequestOffset {
                    shard_name: shard_name.to_string(),
                    offset: new_offset,
                    topic: queue_name.to_string(),
                    partition: 0,
                    expected_offset: Some(expected),
                }],
            }],
        };
        let reply =
            save_offset_data(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(reply.committed)
    }

    /// Claims the next message off the shared cursor for a single `Basic.Get`.
    /// One attempt only: if the conditional commit loses a race to another
    /// node, this returns `Ok(None)` rather than retrying internally, since
    /// the client will simply see it as an empty queue and can call `Get`
    /// again, at which point it reads the offset the winner just committed.
    #[allow(clippy::too_many_arguments)]
    pub async fn read_next_message(
        &self,
        sdm: &Arc<StorageDriverManager>,
        tenant: &str,
        queue: &str,
        shard_name: &str,
        no_ack: bool,
        connection_id: u64,
        channel_id: u16,
    ) -> Result<Option<(StorageRecord, u64, Option<u64>)>, CommonError> {
        let current = self
            .read_committed_offset(tenant, queue, shard_name)
            .await?;

        let read_config = AdapterReadConfig::new();
        let mut offsets = HashMap::new();
        offsets.insert(shard_name.to_string(), current);
        let records = sdm
            .read_by_offset(tenant, queue, &offsets, &read_config)
            .await?;
        let Some(record) = records.into_iter().next() else {
            return Ok(None);
        };

        let msg_offset = record.metadata.offset;
        let new_offset = msg_offset + 1;

        let index_offset = if no_ack {
            None
        } else {
            Some(
                unacked_index::write_entry(
                    sdm,
                    tenant,
                    queue,
                    msg_offset,
                    connection_id,
                    channel_id,
                    broker_config().broker_id,
                )
                .await?,
            )
        };

        if self
            .commit_offset_cas(tenant, queue, shard_name, current, new_offset)
            .await?
        {
            return Ok(Some((record, msg_offset, index_offset)));
        }

        if let Some(index_offset) = index_offset {
            if let Err(e) = unacked_index::delete_entry(sdm, index_offset).await {
                warn!(
                    "AMQP Basic.Get: failed to clean up a stale index entry: {}",
                    e
                );
            }
        }
        Ok(None)
    }
}
