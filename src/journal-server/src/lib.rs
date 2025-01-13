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

use core::cache::{load_metadata_cache, CacheManager};
use core::cluster::{
    register_journal_node, report_heartbeat, report_monitor, unregister_journal_node,
};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use common_base::config::journal_server::{journal_server_conf, JournalServerConfig};
use common_base::metrics::register_prometheus_export;
use common_base::runtime::create_runtime;
use grpc_clients::pool::ClientPool;
use index::engine::{column_family_list, storage_data_fold};
use log::{error, info};
use rocksdb_engine::RocksDBEngine;
use segment::manager::{
    load_local_segment_cache, metadata_and_local_segment_diff_check, SegmentFileManager,
};
use segment::scroll::SegmentScrollManager;
use server::connection_manager::ConnectionManager;
use server::grpc::server::GrpcServer;
use server::tcp::server::start_tcp_server;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::time::sleep;

mod core;
mod handler;
mod index;
mod isr;
mod segment;
mod server;

pub struct JournalServer {
    config: JournalServerConfig,
    stop_send: broadcast::Sender<bool>,
    server_runtime: Runtime,
    daemon_runtime: Runtime,
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl JournalServer {
    pub fn new(stop_send: broadcast::Sender<bool>) -> Self {
        let config = journal_server_conf().clone();
        let server_runtime = create_runtime(
            "storage-engine-server-runtime",
            config.system.runtime_work_threads,
        );
        let daemon_runtime = create_runtime("daemon-runtime", config.system.runtime_work_threads);

        let client_pool = Arc::new(ClientPool::new(3));
        let connection_manager = Arc::new(ConnectionManager::new());
        let cache_manager = Arc::new(CacheManager::new());
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.storage.data_path),
            10000,
            column_family_list(),
        ));

        let segment_file_manager =
            Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

        JournalServer {
            config,
            stop_send,
            server_runtime,
            daemon_runtime,
            client_pool,
            connection_manager,
            cache_manager,
            segment_file_manager,
            rocksdb_engine_handler,
        }
    }

    pub fn start(&self) {
        self.start_grpc_server();

        self.start_tcp_server();

        self.start_prometheus();

        self.init_node();

        self.start_daemon_thread();

        self.waiting_stop();
    }

    fn start_grpc_server(&self) {
        let server = GrpcServer::new(
            self.cache_manager.clone(),
            self.segment_file_manager.clone(),
            self.rocksdb_engine_handler.clone(),
        );
        self.server_runtime.spawn(async move {
            match server.start().await {
                Ok(()) => {}
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
        });
    }

    fn start_tcp_server(&self) {
        let client_pool = self.client_pool.clone();
        let connection_manager = self.connection_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let stop_sx = self.stop_send.clone();
        let segment_file_manager = self.segment_file_manager.clone();
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        self.server_runtime.spawn(async {
            start_tcp_server(
                client_pool,
                connection_manager,
                cache_manager,
                segment_file_manager,
                rocksdb_engine_handler,
                stop_sx,
            )
            .await;
        });
    }

    fn start_prometheus(&self) {
        if self.config.prometheus.enable {
            let prometheus_port = self.config.prometheus.port;
            self.server_runtime.spawn(async move {
                register_prometheus_export(prometheus_port).await;
            });
        }
    }

    fn start_daemon_thread(&self) {
        let client_pool = self.client_pool.clone();
        let stop_sx = self.stop_send.clone();
        self.daemon_runtime
            .spawn(async move { report_heartbeat(client_pool, stop_sx).await });

        let client_pool = self.client_pool.clone();
        let stop_sx = self.stop_send.clone();
        self.daemon_runtime
            .spawn(async move { report_monitor(client_pool, stop_sx).await });

        let segment_scroll = SegmentScrollManager::new(
            self.cache_manager.clone(),
            self.client_pool.clone(),
            self.segment_file_manager.clone(),
        );
        self.daemon_runtime.spawn(async move {
            segment_scroll.trigger_segment_scroll().await;
        });
    }

    fn waiting_stop(&self) {
        self.daemon_runtime.block_on(async move {
            loop {
                signal::ctrl_c().await.expect("failed to listen for event");
                if self.stop_send.send(true).is_ok() {
                    info!(
                        "{}",
                        "When ctrl + c is received, the service starts to stop"
                    );
                    self.stop_server().await;
                    break;
                }
            }
        });
    }

    fn init_node(&self) {
        self.daemon_runtime.block_on(async move {
            // todo
            self.cache_manager.init_cluster();

            load_metadata_cache(&self.cache_manager, &self.client_pool).await;

            for path in self.config.storage.data_path.clone() {
                let path = Path::new(&path);
                match load_local_segment_cache(
                    path,
                    &self.rocksdb_engine_handler,
                    &self.segment_file_manager,
                    &self.config.storage.data_path,
                ) {
                    Ok(()) => {}
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }

            metadata_and_local_segment_diff_check();

            // todo
            sleep(Duration::from_secs(1)).await;
            match register_journal_node(self.client_pool.clone(), self.config.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }

            info!("Journal Node was initialized successfully");
        });
    }

    async fn stop_server(&self) {
        match unregister_journal_node(self.client_pool.clone(), self.config.clone()).await {
            Ok(()) => {}
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}
