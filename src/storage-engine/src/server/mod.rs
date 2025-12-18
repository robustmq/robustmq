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
    core::{cache::StorageCacheManager, command::StorageEngineHandlerCommand},
    segment::manager::SegmentFileManager,
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
pub struct Server {
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<StorageCacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    connection_manager: Arc<ConnectionManager>,
    broker_cache: Arc<BrokerCacheManager>,
}

impl Server {
    pub fn new(
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<StorageCacheManager>,
        segment_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        connection_manager: Arc<ConnectionManager>,
        broker_cache: Arc<BrokerCacheManager>,
    ) -> Server {
        Server {
            client_pool,
            cache_manager,
            segment_file_manager,
            rocksdb_engine_handler,
            connection_manager,
            broker_cache,
        }
    }

    pub async fn start(&self, stop_sx: broadcast::Sender<bool>) {
        let conf = broker_config();
        let command = self.create_command();

        let proc_config = ProcessorConfig {
            accept_thread_num: conf.network.accept_thread_num,
            handler_process_num: conf.network.handler_thread_num,
            response_process_num: conf.network.response_thread_num,
            channel_size: conf.network.queue_size,
        };

        let context: ServerContext = ServerContext {
            connection_manager: self.connection_manager.clone(),
            client_pool: self.client_pool.clone(),
            command: command.clone(),
            network_type: NetworkConnectionType::Tcp,
            proc_config,
            stop_sx: stop_sx.clone(),
            broker_cache: self.broker_cache.clone(),
        };

        // TCP Server

        let name = "Storage Engine".to_string();
        let tcp_server = TcpServer::new(name.clone(), context.clone());
        if let Err(e) = tcp_server.start(false, conf.journal_server.tcp_port).await {
            error!("Storage Engine tCP server start fail, error:{}", e);
        }
    }

    fn create_command(&self) -> Arc<Box<dyn Command + Send + Sync>> {
        let storage: Box<dyn Command + Send + Sync> = Box::new(StorageEngineHandlerCommand::new(
            self.client_pool.clone(),
            self.cache_manager.clone(),
            self.segment_file_manager.clone(),
            self.rocksdb_engine_handler.clone(),
        ));
        Arc::new(storage)
    }
}
