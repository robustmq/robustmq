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
#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]
use common_config::broker::broker_config;
use common_config::config::BrokerConfig;
use core::cache::{load_metadata_cache, CacheManager};
use grpc_clients::pool::ClientPool;
use rocksdb_engine::rocksdb::RocksDBEngine;
use segment::manager::{
    load_local_segment_cache, metadata_and_local_segment_diff_check, SegmentFileManager,
};
use segment::scroll::SegmentScrollManager;
use server::connection_manager::ConnectionManager;
use server::tcp::server::start_tcp_server;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tracing::{error, info};

pub mod core;
pub mod handler;
pub mod index;
pub mod inner;
pub mod isr;
pub mod segment;
pub mod server;

#[derive(Clone)]
pub struct JournalServerParams {
    pub cache_manager: Arc<CacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub connection_manager: Arc<ConnectionManager>,
    pub segment_file_manager: Arc<SegmentFileManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}

pub struct JournalServer {
    config: BrokerConfig,
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    main_stop: broadcast::Sender<bool>,
    inner_stop: broadcast::Sender<bool>,
}

impl JournalServer {
    pub fn new(params: JournalServerParams, main_stop: Sender<bool>) -> Self {
        let config = broker_config();

        let (inner_stop, _) = broadcast::channel(2);
        JournalServer {
            config: config.clone(),
            client_pool: params.client_pool,
            connection_manager: params.connection_manager,
            cache_manager: params.cache_manager,
            segment_file_manager: params.segment_file_manager,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
            main_stop,
            inner_stop,
        }
    }

    pub async fn start(&self) {
        self.init_node().await;

        self.start_tcp_server();

        self.start_daemon_thread();

        self.waiting_stop().await;
    }

    fn start_tcp_server(&self) {
        let client_pool = self.client_pool.clone();
        let connection_manager = self.connection_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let inner_stop = self.inner_stop.clone();
        let segment_file_manager = self.segment_file_manager.clone();
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        tokio::spawn(async {
            start_tcp_server(
                client_pool,
                connection_manager,
                cache_manager,
                segment_file_manager,
                rocksdb_engine_handler,
                inner_stop,
            )
            .await;
        });
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

    async fn init_node(&self) {
        // todo
        self.cache_manager.init_cluster();

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

    async fn stop_server(cache_manager: Arc<CacheManager>, _client_pool: Arc<ClientPool>) {
        cache_manager.stop_all_build_index_thread();
    }
}
