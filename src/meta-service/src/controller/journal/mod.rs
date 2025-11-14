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

use crate::{core::cache::CacheManager, raft::manager::MultiRaftManager};
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use gc::{gc_segment_thread, gc_shard_thread};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

pub mod call_node;
pub mod gc;

pub struct StorageEngineController {
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
}

impl StorageEngineController {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<CacheManager>,
        client_pool: Arc<ClientPool>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        StorageEngineController {
            raft_manager,
            cache_manager,
            client_pool,
            stop_sx,
        }
    }

    pub async fn start(&self) {
        self.delete_shard_gc_thread();
        self.delete_segment_gc_thread();
        info!("Storage Engine Controller started successfully");
    }

    pub fn delete_shard_gc_thread(&self) {
        let raft_machine_apply = self.raft_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        let stop_sx = self.stop_sx.clone();
        tokio::spawn(async move {
            let ac_fn = async || -> ResultCommonError {
                gc_shard_thread(&raft_machine_apply, &cache_manager, &client_pool).await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 1, &stop_sx).await;
        });
    }

    pub fn delete_segment_gc_thread(&self) {
        let raft_machine_apply = self.raft_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        let stop_sx = self.stop_sx.clone();
        tokio::spawn(async move {
            let ac_fn = async || -> ResultCommonError {
                gc_segment_thread(&raft_machine_apply, &cache_manager, &client_pool).await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 1, &stop_sx).await;
        });
    }
}
