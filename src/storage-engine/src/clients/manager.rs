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

use crate::clients::gc::CONNECTION_IDLE_TIMEOUT_SECS;
use crate::clients::pool::ConnectionPool;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use common_base::tools::now_second;
use dashmap::DashMap;
use protocol::storage::codec::StorageEnginePacket;
use std::sync::Arc;
use tracing::error;

pub const CONN_TYPE_READ: &str = "read";
pub const CONN_TYPE_WRITE: &str = "write";

pub struct ClientConnectionManager {
    cache_manager: Arc<StorageCacheManager>,
    read_pools: DashMap<u64, ConnectionPool>,
    write_pools: DashMap<u64, ConnectionPool>,
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

    pub async fn write_send(
        &self,
        node_id: u64,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let pool = self.write_pools.entry(node_id).or_insert_with(|| {
            ConnectionPool::new(
                node_id,
                CONN_TYPE_WRITE,
                self.cache_manager.clone(),
                self.pool_size,
            )
        });

        let seq = pool.get_next_seq();
        let conn = pool.get_or_create_conn(seq).await?;
        conn.send(req_packet).await
    }

    pub async fn read_send(
        &self,
        node_id: u64,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let pool = self.read_pools.entry(node_id).or_insert_with(|| {
            ConnectionPool::new(
                node_id,
                CONN_TYPE_READ,
                self.cache_manager.clone(),
                self.pool_size,
            )
        });

        let seq = pool.get_next_seq();
        let conn = pool.get_or_create_conn(seq).await?;
        conn.send(req_packet).await
    }

    pub async fn get_inactive_conn(&self) -> Vec<(u64, u64, &'static str)> {
        let mut results = Vec::new();

        for pool_entry in self.read_pools.iter() {
            let node_id = *pool_entry.key();
            let conn_type = pool_entry.value().conn_type;
            for conn_entry in pool_entry.value().connections.iter() {
                let seq = *conn_entry.key();
                let node_conn = conn_entry.value();

                if let Some(last_active) = node_conn.get_last_active_time().await {
                    if (now_second() - last_active) > CONNECTION_IDLE_TIMEOUT_SECS {
                        results.push((node_id, seq, conn_type));
                    }
                }
            }
        }

        for pool_entry in self.write_pools.iter() {
            let node_id = *pool_entry.key();
            let conn_type = pool_entry.value().conn_type;
            for conn_entry in pool_entry.value().connections.iter() {
                let seq = *conn_entry.key();
                let node_conn = conn_entry.value();

                if let Some(last_active) = node_conn.get_last_active_time().await {
                    if (now_second() - last_active) > CONNECTION_IDLE_TIMEOUT_SECS {
                        results.push((node_id, seq, conn_type));
                    }
                }
            }
        }

        results
    }

    pub async fn close_conn(&self, node_id: u64, seq: u64, conn_type: &'static str) {
        let pool = match conn_type {
            CONN_TYPE_READ => self.read_pools.get(&node_id),
            CONN_TYPE_WRITE => self.write_pools.get(&node_id),
            _ => return,
        };

        let Some(pool) = pool else {
            return;
        };

        if let Some(node_conn) = pool.connections.get(&seq) {
            if let Err(e) = node_conn.close_connection().await {
                error!(
                    "Failed to close connection: node={}, seq={}, type={}, error={}",
                    node_id, seq, conn_type, e
                );
            }
        }

        pool.connections.remove(&seq);

        if pool.connections.is_empty() {
            drop(pool);
            match conn_type {
                CONN_TYPE_READ => self.read_pools.remove(&node_id),
                CONN_TYPE_WRITE => self.write_pools.remove(&node_id),
                _ => None,
            };
        }
    }

    pub async fn close(&self) {
        for pool_entry in self.read_pools.iter() {
            for node_conn in pool_entry.value().connections.iter() {
                if let Err(e) = node_conn.close_connection().await {
                    error!("Failed to close read connection: {}", e);
                }
            }
        }

        for pool_entry in self.write_pools.iter() {
            for node_conn in pool_entry.value().connections.iter() {
                if let Err(e) = node_conn.close_connection().await {
                    error!("Failed to close write connection: {}", e);
                }
            }
        }

        self.read_pools.clear();
        self.write_pools.clear();
    }
}
