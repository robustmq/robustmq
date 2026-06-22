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

use crate::clients::connection::NodeConnection;
use crate::clients::gc::CONNECTION_IDLE_TIMEOUT_SECS;
use crate::clients::packet::{build_fetch_req, read_resp_parse, write_resp_parse};
use crate::clients::pool::ConnectionPool;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::storage::{adapter_read_config::AdapterWriteRespRow, record::StorageRecord};
use protocol::storage::codec::StorageEnginePacket;
use protocol::storage::protocol::{
    FetchReqBody, FetchRespBody, OffsetsForLeaderEpochReq, OffsetsForLeaderEpochReqBody,
    OffsetsForLeaderEpochRespBody, ReadReq, ShardOffsetReq, ShardOffsetReqBody,
    ShardOffsetRespBody, WriteReq, WriteReqBody,
};
use std::sync::Arc;
use tracing::error;

pub struct ClientConnectionManager {
    cache_manager: Arc<StorageCacheManager>,
    read_pools: DashMap<u64, Arc<ConnectionPool>>,
    write_pools: DashMap<u64, Arc<ConnectionPool>>,
    pool_size: u32,
}

impl ClientConnectionManager {
    pub fn new(cache_manager: Arc<StorageCacheManager>, pool_size: u32) -> Self {
        Self {
            cache_manager,
            read_pools: DashMap::with_capacity(8),
            write_pools: DashMap::with_capacity(8),
            pool_size,
        }
    }

    // ── typed send methods ────────────────────────────────────────────────────

    pub async fn send_write(
        &self,
        node_id: u64,
        shard_name: &str,
        messages: Vec<Vec<u8>>,
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        let body = WriteReqBody::new(shard_name.to_string(), messages);
        self.send_write_body(node_id, body).await
    }

    // send_write with a fully-formed body (e.g. custom acks, timeout_ms).
    pub async fn send_write_body(
        &self,
        node_id: u64,
        body: WriteReqBody,
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        let resp = self
            .write_send(node_id, StorageEnginePacket::WriteReq(WriteReq::new(body)))
            .await?;
        match resp {
            StorageEnginePacket::WriteResp(r) => Ok(write_resp_parse(&r)?),
            other => Err(StorageEngineError::ReceivedPacketError(
                node_id,
                format!("Expected WriteResp, got {other}"),
            )),
        }
    }

    pub async fn send_read(
        &self,
        node_id: u64,
        req: ReadReq,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let resp = self
            .write_send(node_id, StorageEnginePacket::ReadReq(req))
            .await?;
        match resp {
            StorageEnginePacket::ReadResp(r) => Ok(read_resp_parse(&r)?),
            other => Err(StorageEngineError::ReceivedPacketError(
                node_id,
                format!("Expected ReadResp, got {other}"),
            )),
        }
    }

    pub async fn send_shard_offset(
        &self,
        node_id: u64,
        body: ShardOffsetReqBody,
    ) -> Result<ShardOffsetRespBody, StorageEngineError> {
        let req = ShardOffsetReq::new(body);
        let resp = self
            .write_send(node_id, StorageEnginePacket::ShardOffsetReq(req))
            .await?;
        match resp {
            StorageEnginePacket::ShardOffsetResp(r) => Ok(r.body),
            other => Err(StorageEngineError::ReceivedPacketError(
                node_id,
                format!("Expected ShardOffsetResp, got {other}"),
            )),
        }
    }

    pub async fn send_fetch(
        &self,
        node_id: u64,
        body: FetchReqBody,
    ) -> Result<FetchRespBody, StorageEngineError> {
        let req = build_fetch_req(body);
        let resp = self
            .read_send(node_id, StorageEnginePacket::FetchReq(req))
            .await?;
        match resp {
            StorageEnginePacket::FetchResp(r) => Ok(r.body),
            other => Err(StorageEngineError::ReceivedPacketError(
                node_id,
                format!("Expected FetchResp, got {other}"),
            )),
        }
    }

    pub async fn send_offsets_for_leader_epoch(
        &self,
        node_id: u64,
        body: OffsetsForLeaderEpochReqBody,
    ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
        let req = OffsetsForLeaderEpochReq::new(body);
        let resp = self
            .read_send(node_id, StorageEnginePacket::OffsetsForLeaderEpochReq(req))
            .await?;
        match resp {
            StorageEnginePacket::OffsetsForLeaderEpochResp(r) => Ok(r.body),
            other => Err(StorageEngineError::ReceivedPacketError(
                node_id,
                format!("Expected OffsetsForLeaderEpochResp, got {other}"),
            )),
        }
    }

    // ── GC helpers ────────────────────────────────────────────────────────────

    pub fn get_inactive_conns(&self) -> Vec<Arc<NodeConnection>> {
        let now = now_second();
        let mut results = Vec::new();
        for pools in [&self.read_pools, &self.write_pools] {
            for pool_entry in pools.iter() {
                for conn in pool_entry.value().iter_connections() {
                    if let Some(last_active) = conn.get_last_active_time() {
                        if (now - last_active) > CONNECTION_IDLE_TIMEOUT_SECS {
                            results.push(conn.clone());
                        }
                    }
                }
            }
        }
        results
    }

    pub async fn close(&self) {
        let all: Vec<Arc<NodeConnection>> = [&self.read_pools, &self.write_pools]
            .iter()
            .flat_map(|pools| pools.iter())
            .flat_map(|entry| {
                entry
                    .value()
                    .iter_connections()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();

        for conn in all {
            if let Err(e) = conn.close_connection().await {
                error!("Failed to close connection to node {}: {}", conn.node_id, e);
            }
        }

        self.read_pools.clear();
        self.write_pools.clear();
    }

    // ── internal channel send ─────────────────────────────────────────────────

    async fn write_send(
        &self,
        node_id: u64,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let pool = self.get_or_create_pool(&self.write_pools, node_id, "write");
        let seq = pool.get_next_seq();
        pool.get_or_create_conn(seq).send(req_packet).await
    }

    async fn read_send(
        &self,
        node_id: u64,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let pool = self.get_or_create_pool(&self.read_pools, node_id, "read");
        let seq = pool.get_next_seq();
        pool.get_or_create_conn(seq).send(req_packet).await
    }

    fn get_or_create_pool(
        &self,
        pools: &DashMap<u64, Arc<ConnectionPool>>,
        node_id: u64,
        conn_type: &'static str,
    ) -> Arc<ConnectionPool> {
        if let Some(p) = pools.get(&node_id) {
            return p.clone();
        }
        Arc::clone(&*pools.entry(node_id).or_insert_with(|| {
            Arc::new(ConnectionPool::new(
                node_id,
                conn_type,
                self.cache_manager.clone(),
                self.pool_size,
            ))
        }))
    }
}
