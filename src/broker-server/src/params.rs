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
use common_base::{error::common::CommonError, task::TaskSupervisor};
use common_config::{broker::broker_config, storage::memory::StorageDriverMemoryConfig};
use connector::manager::ConnectorManager;
use delay_message::manager::DelayMessageManager;
use delay_task::manager::DelayTaskManager;
use grpc_clients::pool::ClientPool;
use meta_service::{
    core::cache::MetaCacheManager as PlacementCacheManager,
    raft::{manager::MultiRaftManager, route::DataRoute},
    MetaServiceServerParams,
};
use mqtt_broker::{
    broker::MqttBrokerServerParams,
    core::{cache::MQTTCacheManager as MqttCacheManager, retain::RetainMessageManager},
    security::AuthManager,
    storage::session::SessionBatcher,
    subscribe::{manager::SubscribeManager, PushManager},
};
use network_server::common::connection_manager::ConnectionManager as NetworkConnectionManager;
use node_call::NodeCallManager;
use rocksdb_engine::{metrics::mqtt::MQTTMetricsCache, rocksdb::RocksDBEngine};
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use storage_engine::{
    clients::manager::ClientConnectionManager,
    commitlog::memory::engine::MemoryStorageEngine,
    commitlog::rocksdb::engine::RocksDBStorageEngine,
    core::cache::StorageCacheManager,
    filesegment::write::WriteManager,
    group::OffsetManager,
    handler::adapter::{StorageEngineHandler, StorageEngineHandlerParams},
    StorageEngineParams,
};
use tokio::sync::broadcast;
use tracing::error;

/// Build [`MetaServiceServerParams`] on the caller's runtime so that all
/// openraft internal tasks (spawned during `Raft::new`) land on that runtime.
pub async fn build_meta_service(
    client_pool: Arc<ClientPool>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    delay_task_manager: Arc<DelayTaskManager>,
    node_call_manager: Arc<NodeCallManager>,
    broker_cache: Arc<NodeCacheManager>,
    task_supervisor: Arc<TaskSupervisor>,
) -> MetaServiceServerParams {
    let cache_manager = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));

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
        broker_cache,
        task_supervisor,
    }
}

/// Build [`MqttBrokerServerParams`] on the caller's runtime so that tasks
/// spawned during construction (e.g. `RetainMessageManager`) land there.
#[allow(clippy::too_many_arguments)]
pub async fn build_broker_mqtt_params(
    client_pool: Arc<ClientPool>,
    broker_cache: Arc<NodeCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    connection_manager: Arc<NetworkConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    offset_manager: Arc<OffsetManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_sx: broadcast::Sender<bool>,
) -> Result<MqttBrokerServerParams, CommonError> {
    let cache_manager = Arc::new(MqttCacheManager::new(
        client_pool.clone(),
        broker_cache.clone(),
    ));
    let subscribe_manager = Arc::new(SubscribeManager::new());
    let connector_manager = Arc::new(ConnectorManager::new());
    let auth_driver = Arc::new(AuthManager::new(cache_manager.clone()));
    let delay_message_manager = Arc::new(
        DelayMessageManager::new(client_pool.clone(), storage_driver_manager.clone(), 5).await?,
    );
    let metrics_cache_manager = Arc::new(MQTTMetricsCache::new(rocksdb_engine_handler.clone()));
    let schema_manager = Arc::new(SchemaRegisterManager::new());
    let retain_message_manager = RetainMessageManager::new(
        cache_manager.clone(),
        client_pool.clone(),
        connection_manager.clone(),
        stop_sx,
    );
    let push_manager = Arc::new(PushManager::new(
        cache_manager.clone(),
        storage_driver_manager.clone(),
        connection_manager.clone(),
        rocksdb_engine_handler.clone(),
        subscribe_manager.clone(),
        client_pool.clone(),
    ));

    let session_batcher = SessionBatcher::new();

    Ok(MqttBrokerServerParams {
        cache_manager,
        client_pool,
        session_batcher,
        storage_driver_manager,
        subscribe_manager,
        connection_manager,
        connector_manager,
        auth_driver,
        delay_message_manager,
        schema_manager,
        metrics_cache_manager,
        rocksdb_engine_handler,
        broker_cache,
        offset_manager,
        retain_message_manager,
        push_manager,
        task_supervisor,
    })
}

/// Build [`StorageEngineParams`] synchronously; no async context needed.
pub fn build_storage_engine_params(
    client_pool: Arc<ClientPool>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    broker_cache: Arc<NodeCacheManager>,
    connection_manager: Arc<NetworkConnectionManager>,
    offset_manager: Arc<OffsetManager>,
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
        rocksdb_engine_handler,
        connection_manager,
        client_connection_manager,
        memory_storage_engine,
        rocksdb_storage_engine,
        write_manager,
        storage_engine_handler,
    }
}
