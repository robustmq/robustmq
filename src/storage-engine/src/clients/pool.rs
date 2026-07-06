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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

pub struct ConnectionPool {
    pub node_id: u64,
    pub conn_type: &'static str,
    cache_manager: Arc<StorageCacheManager>,
    // Fixed-size slot array indexed by seq. OnceLock guarantees each slot is
    // initialized exactly once; after that, get() is a single atomic load with
    // no heap allocation or locking.
    slots: Vec<OnceLock<Arc<NodeConnection>>>,
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
        let mut slots = Vec::with_capacity(pool_size as usize);
        for _ in 0..pool_size {
            slots.push(OnceLock::new());
        }
        Self {
            node_id,
            conn_type,
            cache_manager,
            slots,
            atom: AtomicU64::new(0),
            pool_size,
        }
    }

    // Returns the connection for the given slot, creating it lazily if needed.
    // After the first call per slot this is a single atomic load — no lock, no hash.
    pub fn get_or_create_conn(&self, seq: u64) -> &Arc<NodeConnection> {
        self.slots[seq as usize].get_or_init(|| {
            Arc::new(NodeConnection::new(
                self.node_id,
                self.cache_manager.clone(),
            ))
        })
    }

    pub fn get_next_seq(&self) -> u64 {
        self.atom.fetch_add(1, Ordering::Relaxed) % self.pool_size as u64
    }

    pub fn iter_connections(&self) -> impl Iterator<Item = &Arc<NodeConnection>> {
        self.slots.iter().filter_map(OnceLock::get)
    }
}
