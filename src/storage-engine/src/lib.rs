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

#![allow(clippy::result_large_err)]

use crate::clients::manager::ClientConnectionManager;
use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::filesegment::expire::start_segment_expire_thread;
use crate::handler::adapter::StorageEngineHandler;
use crate::server::Server;
use crate::{clients::gc::start_conn_gc_thread, filesegment::write::WriteManager};
use core::cache::{load_metadata_cache, StorageCacheManager};
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tracing::{error, info};

pub mod clients;
pub mod commitlog;
pub mod core;
pub mod filesegment;
pub mod group;
pub mod handler;
pub mod isr;
pub mod server;

#[derive(Clone)]
pub struct StorageEngineParams {
    pub cache_manager: Arc<StorageCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub connection_manager: Arc<ConnectionManager>,
    pub write_manager: Arc<WriteManager>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    pub storage_engine_handler: Arc<StorageEngineHandler>,
}

pub struct StorageEngineServer {
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    write_manager: Arc<WriteManager>,
    main_stop: broadcast::Sender<bool>,
    inner_stop: broadcast::Sender<bool>,
    client_connection_manager: Arc<ClientConnectionManager>,
    memory_storage_engine: Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
}

impl StorageEngineServer {
    pub fn new(params: StorageEngineParams, main_stop: Sender<bool>) -> Self {
        let (inner_stop, _) = broadcast::channel(2);
        StorageEngineServer {
            client_pool: params.client_pool,
            cache_manager: params.cache_manager,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
            connection_manager: params.connection_manager,
            write_manager: params.write_manager,
            client_connection_manager: params.client_connection_manager,
            memory_storage_engine: params.memory_storage_engine,
            rocksdb_storage_engine: params.rocksdb_storage_engine,
            main_stop,
            inner_stop,
        }
    }

    pub async fn start(&self) {
        self.init().await;

        self.start_tcp_server();

        self.start_daemon_thread();

        self.waiting_stop().await;
    }

    fn start_tcp_server(&self) {
        let tcp_server = Server::new(crate::server::ServerParams {
            client_pool: self.client_pool.clone(),
            cache_manager: self.cache_manager.clone(),
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            connection_manager: self.connection_manager.clone(),
            write_manager: self.write_manager.clone(),
            broker_cache: self.cache_manager.broker_cache.clone(),
            memory_storage_engine: self.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.rocksdb_storage_engine.clone(),
            client_connection_manager: self.client_connection_manager.clone(),
        });
        let stop_sx = self.inner_stop.clone();
        tokio::spawn(async move { tcp_server.start(stop_sx).await });
    }

    fn start_daemon_thread(&self) {
        self.write_manager.start(self.inner_stop.clone());
        start_conn_gc_thread(
            self.client_connection_manager.clone(),
            self.inner_stop.clone(),
        );

        // segment engine
        let client_pool = self.client_pool.clone();
        let cache_manager = self.cache_manager.clone();
        let stop_sx = self.inner_stop.clone();
        tokio::spawn(async move {
            start_segment_expire_thread(client_pool, cache_manager, &stop_sx).await
        });

        // rocksdb engine
        let rocksdb_storage_engine = self.rocksdb_storage_engine.clone();
        let stop_sx = self.inner_stop.clone();
        tokio::spawn(async move {
            rocksdb_storage_engine.start_expire_thread(&stop_sx).await;
        });
    }

    async fn waiting_stop(&self) {
        let inner_stop = self.inner_stop.clone();
        let client_pool = self.client_pool.clone();
        let cache_manager = self.cache_manager.clone();
        let mut recv = self.main_stop.subscribe();

        match recv.recv().await {
            Ok(_) => {
                info!("Journal has stopped.");
                if inner_stop.send(true).is_ok() {
                    StorageEngineServer::stop_server(cache_manager, client_pool).await;
                }
            }
            Err(e) => {
                error!("{}", e)
            }
        }
    }

    async fn init(&self) {
        if let Err(e) = load_metadata_cache(&self.cache_manager, &self.client_pool).await {
            error!("{}", e);
        }
        info!("Journal Node was initialized successfully");
    }

    async fn stop_server(_cache_manager: Arc<StorageCacheManager>, _client_pool: Arc<ClientPool>) {}
}
