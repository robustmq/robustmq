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
use common_config::{broker::broker_config, storage::memory::StorageDriverMemoryConfig};
use common_healthy::port::wait_for_engine_ready;
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager as NetworkConnectionManager;
use rate_limit::global::GlobalRateLimiterManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use storage_engine::{
    clients::manager::ClientConnectionManager,
    commitlog::memory::engine::MemoryStorageEngine,
    commitlog::rocksdb::engine::RocksDBStorageEngine,
    core::cache::StorageCacheManager,
    filesegment::write::WriteManager,
    group::OffsetManager,
    handler::adapter::{StorageEngineHandler, StorageEngineHandlerParams},
    StorageEngineParams, StorageEngineServer,
};
use tokio::sync::broadcast;

use crate::BrokerServer;

/// Build [`StorageEngineParams`] synchronously; no async context needed.
pub fn build_storage_engine_params(
    client_pool: Arc<ClientPool>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    broker_cache: Arc<NodeCacheManager>,
    connection_manager: Arc<NetworkConnectionManager>,
    offset_manager: Arc<OffsetManager>,
    global_limit_manager: Arc<GlobalRateLimiterManager>,
    task_supervisor: Arc<TaskSupervisor>,
) -> StorageEngineParams {
    let config = broker_config();

    let cache_manager = Arc::new(StorageCacheManager::new(broker_cache.clone()));
    let write_manager = Arc::new(WriteManager::new(
        rocksdb_engine_handler.clone(),
        cache_manager.clone(),
        client_pool.clone(),
        config.storage_runtime.io_thread_num,
    ));
    let memory_storage_engine = Arc::new(MemoryStorageEngine::new(
        rocksdb_engine_handler.clone(),
        cache_manager.clone(),
        StorageDriverMemoryConfig::default(),
    ));
    let rocksdb_storage_engine = Arc::new(RocksDBStorageEngine::new(
        cache_manager.clone(),
        rocksdb_engine_handler.clone(),
    ));
    let client_connection_manager =
        Arc::new(ClientConnectionManager::new(cache_manager.clone(), 4));
    let storage_engine_handler = Arc::new(StorageEngineHandler::new(StorageEngineHandlerParams {
        cache_manager: cache_manager.clone(),
        client_pool: client_pool.clone(),
        memory_storage_engine: memory_storage_engine.clone(),
        rocksdb_storage_engine: rocksdb_storage_engine.clone(),
        client_connection_manager: client_connection_manager.clone(),
        rocksdb_engine_handler: rocksdb_engine_handler.clone(),
        write_manager: write_manager.clone(),
        offset_manager: offset_manager.clone(),
    }));

    StorageEngineParams {
        cache_manager,
        client_pool,
        task_supervisor,
        rocksdb_engine_handler,
        connection_manager,
        client_connection_manager,
        memory_storage_engine,
        rocksdb_storage_engine,
        write_manager,
        storage_engine_handler,
        global_limit_manager,
    }
}

impl BrokerServer {
    pub fn start_engine_service(&self) -> Option<broadcast::Sender<bool>> {
        use common_base::role::is_engine_node;
        if !is_engine_node(&self.config.roles) {
            return None;
        }
        let (stop_send, _) = broadcast::channel(2);
        let stop_handle = stop_send.clone();

        self.engine_params
            .memory_storage_engine
            .start_expire_task(&stop_send);

        let server = StorageEngineServer::new(
            self.engine_params.clone(),
            stop_send,
            self.task_supervisor.clone(),
        );
        self.broker_runtime.spawn(Box::pin(async move {
            server.start().await;
        }));
        if !wait_for_engine_ready(self.config.storage_runtime.tcp_port) {
            std::process::exit(1);
        }
        Some(stop_handle)
    }
}
