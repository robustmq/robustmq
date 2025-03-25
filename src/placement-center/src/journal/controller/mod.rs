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

use std::sync::Arc;
use std::time::Duration;

use gc::{gc_segment_thread, gc_shard_thread};
use grpc_clients::pool::ClientPool;
use log::info;
use preferred_election::PreferredElection;
use tokio::time::sleep;

use super::cache::JournalCacheManager;
use crate::core::cache::PlacementCacheManager;
use crate::route::apply::RaftMachineApply;

pub mod call_node;
pub mod gc;
pub mod preferred_election;

pub struct StorageEngineController {
    raft_machine_apply: Arc<RaftMachineApply>,
    engine_cache: Arc<JournalCacheManager>,
    cluster_cache: Arc<PlacementCacheManager>,
    client_pool: Arc<ClientPool>,
}

impl StorageEngineController {
    pub fn new(
        raft_machine_apply: Arc<RaftMachineApply>,
        engine_cache: Arc<JournalCacheManager>,
        cluster_cache: Arc<PlacementCacheManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        StorageEngineController {
            raft_machine_apply,
            engine_cache,
            cluster_cache,
            client_pool,
        }
    }

    pub async fn start(&self) {
        self.delete_shard_gc_thread();
        self.delete_segment_gc_thread();
        self.preferred_replica_election();
        info!("Storage Engine Controller started successfully");
    }

    pub fn delete_shard_gc_thread(&self) {
        let raft_machine_apply = self.raft_machine_apply.clone();
        let engine_cache = self.engine_cache.clone();
        let cluster_cache = self.cluster_cache.clone();
        let client_pool = self.client_pool.clone();
        tokio::spawn(async move {
            loop {
                gc_shard_thread(
                    raft_machine_apply.clone(),
                    engine_cache.clone(),
                    cluster_cache.clone(),
                    client_pool.clone(),
                )
                .await;
                sleep(Duration::from_secs(1)).await;
            }
        });
    }

    pub fn delete_segment_gc_thread(&self) {
        let raft_machine_apply = self.raft_machine_apply.clone();
        let engine_cache = self.engine_cache.clone();
        let cluster_cache = self.cluster_cache.clone();
        let client_pool = self.client_pool.clone();
        tokio::spawn(async move {
            loop {
                gc_segment_thread(
                    raft_machine_apply.clone(),
                    engine_cache.clone(),
                    cluster_cache.clone(),
                    client_pool.clone(),
                )
                .await;
                sleep(Duration::from_secs(1)).await;
            }
        });
    }

    pub fn preferred_replica_election(&self) {
        let election = PreferredElection::new();
        tokio::spawn(async move {
            election.start().await;
        });
    }
}
