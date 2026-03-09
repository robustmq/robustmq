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
use crate::filesegment::write::WriteManager;
use crate::handler::adapter::StorageEngineHandler;
use crate::server::Server;
use common_base::task::{TaskKind, TaskSupervisor};
use core::cache::StorageCacheManager;
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
    cache_manager: Arc<StorageCacheManager>,
    write_manager: Arc<WriteManager>,
    stop: broadcast::Sender<bool>,
    client_connection_manager: Arc<ClientConnectionManager>,
    rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    server: Arc<Server>,
    task_supervisor: Arc<TaskSupervisor>,
}

impl StorageEngineServer {
    pub fn new(
        params: StorageEngineParams,
        stop: Sender<bool>,
        task_supervisor: Arc<TaskSupervisor>,
    ) -> Self {
        let server = Arc::new(Server::new(
            crate::server::ServerParams {
                client_pool: params.client_pool.clone(),
                cache_manager: params.cache_manager.clone(),
                rocksdb_engine_handler: params.rocksdb_engine_handler.clone(),
                connection_manager: params.connection_manager.clone(),
                write_manager: params.write_manager.clone(),
                broker_cache: params.cache_manager.broker_cache.clone(),
                memory_storage_engine: params.memory_storage_engine.clone(),
                rocksdb_storage_engine: params.rocksdb_storage_engine.clone(),
                client_connection_manager: params.client_connection_manager.clone(),
            },
            stop.clone(),
        ));
        StorageEngineServer {
            client_pool: params.client_pool,
            cache_manager: params.cache_manager,
            write_manager: params.write_manager,
            client_connection_manager: params.client_connection_manager,
            rocksdb_storage_engine: params.rocksdb_storage_engine,
            stop,
            server,
            task_supervisor,
        }
    }

    pub async fn start(&self) {
        self.start_daemon_thread();

        self.start_tcp_server();

        self.waiting_stop().await;
    }

    fn start_tcp_server(&self) {
        let server = self.server.clone();
        tokio::spawn(async move { server.start().await });
    }

    fn start_daemon_thread(&self) {
        self.write_manager.start(self.stop.clone());

        let conn_manager = self.client_connection_manager.clone();
        let stop_sx = self.stop.clone();
        self.task_supervisor
            .spawn(TaskKind::StorageEngineConnGC.to_string(), async move {
                crate::clients::gc::start_conn_gc_thread(conn_manager, stop_sx).await;
            });

        // segment engine
        let client_pool = self.client_pool.clone();
        let cache_manager = self.cache_manager.clone();
        let stop_sx = self.stop.clone();
        self.task_supervisor.spawn(
            TaskKind::StorageEngineSegmentExpire.to_string(),
            async move {
                start_segment_expire_thread(client_pool, cache_manager, &stop_sx).await;
            },
        );

        // rocksdb engine
        let rocksdb_storage_engine = self.rocksdb_storage_engine.clone();
        let stop_sx = self.stop.clone();
        self.task_supervisor.spawn(
            TaskKind::StorageEngineRocksDBExpire.to_string(),
            async move {
                rocksdb_storage_engine.start_expire_thread(&stop_sx).await;
            },
        );
    }

    async fn waiting_stop(&self) {
        let mut recv = self.stop.subscribe();
        match recv.recv().await {
            Ok(_) => {
                info!("Storage Engine has stopped.");
            }
            Err(e) => {
                error!("{}", e)
            }
        }
    }
}
