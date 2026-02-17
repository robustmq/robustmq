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

use crate::{
    clients::manager::ClientConnectionManager, commitlog::memory::engine::MemoryStorageEngine,
    commitlog::rocksdb::engine::RocksDBStorageEngine, core::cache::StorageCacheManager,
    filesegment::write::WriteManager, handler::command::StorageEngineHandlerCommand,
};
use broker_core::cache::BrokerCacheManager;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use network_server::{
    command::Command,
    common::connection_manager::ConnectionManager,
    context::{ProcessorConfig, ServerContext},
    tcp::server::TcpServer,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::error;

pub mod inner;

pub struct ServerParams {
    pub client_pool: Arc<ClientPool>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub connection_manager: Arc<ConnectionManager>,
    pub write_manager: Arc<WriteManager>,
    pub broker_cache: Arc<BrokerCacheManager>,
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
}

pub struct Server {
    tcp_server: TcpServer,
}

impl Server {
    pub fn new(params: ServerParams, stop_sx: broadcast::Sender<bool>) -> Server {
        let conf = broker_config();
        let storage: Box<dyn Command + Send + Sync> = Box::new(StorageEngineHandlerCommand::new(
            params.cache_manager.clone(),
            params.rocksdb_engine_handler.clone(),
            params.write_manager.clone(),
            params.memory_storage_engine.clone(),
            params.rocksdb_storage_engine.clone(),
            params.client_connection_manager.clone(),
            params.connection_manager.clone(),
        ));
        let command = Arc::new(storage);

        let proc_config = ProcessorConfig {
            accept_thread_num: conf.network.accept_thread_num,
            handler_process_num: conf.network.handler_thread_num,
            channel_size: conf.network.queue_size,
        };

        let context: ServerContext = ServerContext {
            connection_manager: params.connection_manager.clone(),
            client_pool: params.client_pool.clone(),
            command: command.clone(),
            network_type: NetworkConnectionType::Tcp,
            proc_config,
            stop_sx: stop_sx.clone(),
            broker_cache: params.broker_cache.clone(),
        };

        // TCP Server
        let name = "Storage Engine".to_string();
        let tcp_server = TcpServer::new(name.clone(), context.clone());
        Server { tcp_server }
    }

    pub async fn start(&self) {
        let conf = broker_config();
        if let Err(e) = self
            .tcp_server
            .start(false, conf.storage_runtime.tcp_port)
            .await
        {
            error!("Storage Engine tCP server start fail, error:{}", e);
        }
    }
}
