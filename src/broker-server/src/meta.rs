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

use broker_core::cache::NodeCacheManager;
use common_base::task::TaskSupervisor;
use common_config::broker::broker_config;
use delay_task::manager::DelayTaskManager;
use grpc_clients::pool::ClientPool;
use meta_service::{
    core::cache::MetaCacheManager as PlacementCacheManager,
    raft::{manager::MultiRaftManager, route::DataRoute},
    MetaServiceServer, MetaServiceServerParams,
};
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::error;

use crate::BrokerServer;

/// Build [`MetaServiceServerParams`] on the caller's runtime so that all
/// openraft internal tasks (spawned during `Raft::new`) land on that runtime.
pub async fn build_meta_service(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    delay_task_manager: Arc<DelayTaskManager>,
    node_call_manager: Arc<NodeCallManager>,
    broker_cache: Arc<NodeCacheManager>,
    task_supervisor: Arc<TaskSupervisor>,
) -> MetaServiceServerParams {
    let cache_manager = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
    // network channels for interaction between raft state machines are independent and should not be crowded with other communications;
    // otherwise, it will cause channel congestion.
    // 1. When the client writes to the raft state machine, the state machine processes it slowly
    // 2. Requests are piled up in grpc and cannot be sent out in raft state
    // 3. The client writes more slowly, and raft cannot be sent out even more
    let config = broker_config();
    let client_pool = Arc::new(ClientPool::new(config.runtime.channels_per_address));
    let data_route = Arc::new(DataRoute::new(
        rocksdb_engine_handler.clone(),
        cache_manager.clone(),
        delay_task_manager.clone(),
        broker_cache.clone(),
    ));
    let raft_manager = Arc::new(
        match MultiRaftManager::new(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            data_route,
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to create MultiRaftManager: {}", e);
                std::process::exit(1);
            }
        },
    );

    MetaServiceServerParams {
        cache_manager,
        rocksdb_engine_handler,
        client_pool,
        node_call_manager,
        raft_manager,
        delay_task_manager,
        node_cache: broker_cache,
        task_supervisor,
    }
}

impl BrokerServer {
    pub fn start_meta_service(&self) -> Option<broadcast::Sender<bool>> {
        use common_base::role::is_meta_node;
        if !is_meta_node(&self.config.roles) {
            return None;
        }
        let (stop_send, _) = broadcast::channel(2);
        let place_params = self.meta_params.clone();
        let tx = stop_send.clone();
        self.meta_runtime.spawn(Box::pin(async move {
            MetaServiceServer::new(place_params, tx).start().await;
        }));
        Some(stop_send)
    }
}
