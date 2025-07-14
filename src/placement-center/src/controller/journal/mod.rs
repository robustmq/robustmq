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

use crate::core::cache::CacheManager;
use crate::raft::route::apply::StorageDriver;
use gc::{gc_segment_thread, gc_shard_thread};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

pub mod call_node;
pub mod gc;

pub struct StorageEngineController {
    raft_machine_apply: Arc<StorageDriver>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
}

impl StorageEngineController {
    pub fn new(
        raft_machine_apply: Arc<StorageDriver>,
        cache_manager: Arc<CacheManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        StorageEngineController {
            raft_machine_apply,
            cache_manager,
            client_pool,
        }
    }

    pub async fn start(&self) {
        self.delete_shard_gc_thread();
        self.delete_segment_gc_thread();
        info!("Storage Engine Controller started successfully");
    }

    pub fn delete_shard_gc_thread(&self) {
        let raft_machine_apply = self.raft_machine_apply.clone();
        let cache_manager = self.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        tokio::spawn(async move {
            loop {
                gc_shard_thread(
                    raft_machine_apply.clone(),
                    cache_manager.clone(),
                    client_pool.clone(),
                )
                .await;
                sleep(Duration::from_secs(1)).await;
            }
        });
    }

    pub fn delete_segment_gc_thread(&self) {
        let raft_machine_apply = self.raft_machine_apply.clone();
        let cache_manager = self.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        tokio::spawn(async move {
            loop {
                gc_segment_thread(
                    raft_machine_apply.clone(),
                    cache_manager.clone(),
                    client_pool.clone(),
                )
                .await;
                sleep(Duration::from_secs(1)).await;
            }
        });
    }
}
