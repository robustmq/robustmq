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
use std::sync::atomic::Ordering;
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

use crate::core::unacked_index;
use crate::push::manager::{AmqpPushManager, UNSEEDED};

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

    /// No CAS: leader election already guarantees a single writer, so this
    /// only needs to be durable enough for the next leader to resume from.
    async fn persist_offset(
        &self,
        tenant: &str,
        queue_name: &str,
        shard_name: &str,
        new_offset: u64,
    ) -> Result<(), CommonError> {
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
                    expected_offset: None,
                }],
            }],
        };
        save_offset_data(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    /// Claims the next message for a queue this node leads, via an in-memory
    /// cursor (seeded from the last committed offset on first use). Shared
    /// by `Basic.Get` and `Basic.Consume`'s push loop; unacked-table
    /// bookkeeping is left to the caller.
    pub async fn claim_next_record(
        &self,
        push_manager: &AmqpPushManager,
        sdm: &Arc<StorageDriverManager>,
        tenant: &str,
        queue: &str,
        shard_name: &str,
    ) -> Result<Option<(StorageRecord, u64)>, CommonError> {
        let cursor = push_manager.cursor(tenant, queue, shard_name);
        let mut current = cursor.load(Ordering::SeqCst);
        if current == UNSEEDED {
            current = self
                .read_committed_offset(tenant, queue, shard_name)
                .await?;
            cursor.store(current, Ordering::SeqCst);
        }

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
        cursor.store(new_offset, Ordering::SeqCst);
        self.persist_offset(tenant, queue, shard_name, new_offset)
            .await?;

        Ok(Some((record, msg_offset)))
    }

    /// `claim_next_record` plus, for `no_ack == false`, an unacked-index
    /// write in the same step. Used by `Basic.Get`; Consume's push loop
    /// calls `claim_next_record` directly since it may retry against a
    /// different member.
    #[allow(clippy::too_many_arguments)]
    pub async fn claim_and_track(
        &self,
        push_manager: &AmqpPushManager,
        sdm: &Arc<StorageDriverManager>,
        tenant: &str,
        queue: &str,
        shard_name: &str,
        no_ack: bool,
        connection_id: u64,
        channel_id: u16,
        broker_id: u64,
    ) -> Result<Option<(StorageRecord, u64, Option<u64>)>, CommonError> {
        let Some((record, msg_offset)) = self
            .claim_next_record(push_manager, sdm, tenant, queue, shard_name)
            .await?
        else {
            return Ok(None);
        };

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
                    broker_id,
                )
                .await?,
            )
        };

        Ok(Some((record, msg_offset, index_offset)))
    }
}
