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

use std::sync::Arc;

use common_base::config::journal_server::{journal_server_conf, JournalServerConfig};
use common_base::metrics::register_prometheus_export;
use common_base::runtime::create_runtime;
use core::cache::CacheManager;
use core::cluster::{register_journal_node, report_heartbeat, unregister_journal_node};
use grpc_clients::poll::ClientPool;
use log::{error, info};
use server::connection_manager::ConnectionManager;
use server::grpc::server::GrpcServer;
use server::tcp::server::start_tcp_server;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::broadcast;

mod core;
mod index;
mod record;
mod server;
mod shard;

pub struct JournalServer {
    config: JournalServerConfig,
    stop_send: broadcast::Sender<bool>,
    server_runtime: Runtime,
    daemon_runtime: Runtime,
    client_poll: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
}

impl JournalServer {
    pub fn new(stop_send: broadcast::Sender<bool>) -> Self {
        let config = journal_server_conf().clone();
        let server_runtime = create_runtime(
            "storage-engine-server-runtime",
            config.system.runtime_work_threads,
        );
        let daemon_runtime = create_runtime("daemon-runtime", config.system.runtime_work_threads);

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let connection_manager: Arc<ConnectionManager> = Arc::new(ConnectionManager::new());
        let cache_manager: Arc<CacheManager> = Arc::new(CacheManager::new());
        JournalServer {
            config,
            stop_send,
            server_runtime,
            daemon_runtime,
            client_poll,
            connection_manager,
            cache_manager,
        }
    }

    pub fn start(&self) {
        self.register_node();

        self.start_grpc_server();

        self.start_tcp_server();

        self.start_daemon_thread();

        self.start_prometheus();

        self.waiting_stop();
    }

    fn start_grpc_server(&self) {
        let server = GrpcServer::new(self.config.network.grpc_port, self.client_poll.clone());
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
        let client_poll = self.client_poll.clone();
        let connection_manager = self.connection_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let stop_sx = self.stop_send.clone();
        self.server_runtime.spawn(async {
            start_tcp_server(client_poll, connection_manager, cache_manager, stop_sx).await;
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
        let client_poll = self.client_poll.clone();
        let stop_sx = self.stop_send.clone();
        self.daemon_runtime
            .spawn(async move { report_heartbeat(client_poll, stop_sx).await });
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

    fn register_node(&self) {
        self.daemon_runtime.block_on(async move {
            match register_journal_node(self.client_poll.clone(), self.config.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        });
    }

    async fn stop_server(&self) {
        match unregister_journal_node(self.client_poll.clone(), self.config.clone()).await {
            Ok(()) => {}
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}
