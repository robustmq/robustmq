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

// #![allow(dead_code, unused_variables)]
use common_config::broker::broker_config;
use common_config::config::BrokerConfig;
use core::cache::{load_metadata_cache, StorageCacheManager};
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use segment::manager::{
    load_local_segment_cache, metadata_and_local_segment_diff_check, SegmentFileManager,
};
use segment::scroll::SegmentScrollManager;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tracing::{error, info};

use crate::server::Server;

pub mod core;
pub mod index;
pub mod isr;
pub mod segment;
pub mod server;

#[derive(Clone)]
pub struct StorageEngineParams {
    pub cache_manager: Arc<StorageCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub segment_file_manager: Arc<SegmentFileManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub connection_manager: Arc<ConnectionManager>,
}

pub struct JournalServer {
    config: BrokerConfig,
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<StorageCacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    main_stop: broadcast::Sender<bool>,
    inner_stop: broadcast::Sender<bool>,
}

impl JournalServer {
    pub fn new(params: StorageEngineParams, main_stop: Sender<bool>) -> Self {
        let config = broker_config();

        let (inner_stop, _) = broadcast::channel(2);
        JournalServer {
            config: config.clone(),
            client_pool: params.client_pool,
            cache_manager: params.cache_manager,
            segment_file_manager: params.segment_file_manager,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
            connection_manager: params.connection_manager,
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
        let tcp_server = Server::new(
            self.client_pool.clone(),
            self.cache_manager.clone(),
            self.segment_file_manager.clone(),
            self.rocksdb_engine_handler.clone(),
            self.connection_manager.clone(),
            self.cache_manager.broker_cache.clone(),
        );
        let stop_sx = self.inner_stop.clone();
        tokio::spawn(async move { tcp_server.start(stop_sx).await });
    }

    fn start_daemon_thread(&self) {
        let segment_scroll = SegmentScrollManager::new(
            self.cache_manager.clone(),
            self.client_pool.clone(),
            self.segment_file_manager.clone(),
        );
        tokio::spawn(async move {
            segment_scroll.trigger_segment_scroll().await;
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
                    JournalServer::stop_server(cache_manager, client_pool).await;
                }
            }
            Err(e) => {
                error!("{}", e)
            }
        }
    }

    async fn init(&self) {
        load_metadata_cache(&self.cache_manager, &self.client_pool).await;

        for path in self.config.journal_storage.data_path.clone() {
            let path = Path::new(&path);
            match load_local_segment_cache(
                path,
                &self.rocksdb_engine_handler,
                &self.segment_file_manager,
                &self.config.journal_storage.data_path,
            ) {
                Ok(()) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }

        metadata_and_local_segment_diff_check();

        info!("Journal Node was initialized successfully");
    }

    async fn stop_server(cache_manager: Arc<StorageCacheManager>, _client_pool: Arc<ClientPool>) {
        cache_manager.stop_all_build_index_thread();
    }
}
