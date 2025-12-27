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
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct ConnectionPool {
    pub node_id: u64,
    pub conn_type: &'static str,
    cache_manager: Arc<StorageCacheManager>,
    pub connections: DashMap<u64, Arc<NodeConnection>>,
    atom: AtomicU64,
    pool_size: u32,
}

impl ConnectionPool {
    pub fn new(
        node_id: u64,
        conn_type: &'static str,
        cache_manager: Arc<StorageCacheManager>,
        pool_size: u32,
    ) -> Self {
        Self {
            node_id,
            conn_type,
            cache_manager,
            connections: DashMap::with_capacity(pool_size as usize),
            atom: AtomicU64::new(0),
            pool_size,
        }
    }

    pub async fn get_or_create_conn(
        &self,
        seq: u64,
    ) -> Result<Arc<NodeConnection>, StorageEngineError> {
        if let Some(conn) = self.connections.get(&seq) {
            return Ok(conn.clone());
        }

        let node_conn = Arc::new(NodeConnection::new(
            self.node_id,
            self.cache_manager.clone(),
        ));
        node_conn.connect().await?;

        self.connections.insert(seq, node_conn.clone());
        Ok(node_conn)
    }

    pub fn get_next_seq(&self) -> u64 {
        let index = self.atom.fetch_add(1, Ordering::Relaxed);
        index % self.pool_size as u64
    }
}
