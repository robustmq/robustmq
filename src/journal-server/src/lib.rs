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

use clients::poll::ClientPool;
use cluster::{register_storage_engine_node, report_heartbeat, unregister_storage_engine_node};
use common_base::config::journal_server::{journal_server_conf, JournalServerConfig};
use common_base::metrics::register_prometheus_export;
use common_base::runtime::create_runtime;
use log::info;
use server::start_tcp_server;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::broadcast;

mod cluster;
mod index;
mod network;
mod raft;
mod record;
mod server;
mod shard;
mod storage;

pub struct JournalServer {
    config: JournalServerConfig,
    stop_send: broadcast::Sender<bool>,
    server_runtime: Runtime,
    daemon_runtime: Runtime,
    client_poll: Arc<ClientPool>,
}

impl JournalServer {
    pub fn new(stop_send: broadcast::Sender<bool>) -> Self {
        let config = journal_server_conf().clone();
        let server_runtime =
            create_runtime("storage-engine-server-runtime", config.runtime_work_threads);
        let daemon_runtime = create_runtime("daemon-runtime", config.runtime_work_threads);

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));

        JournalServer {
            config,
            stop_send,
            server_runtime,
            daemon_runtime,
            client_poll,
        }
    }

    pub fn start(&self) {
        self.register_node();

        self.start_prometheus_export();

        self.start_tcp_server();

        self.start_daemon_thread();

        self.waiting_stop();
    }

    fn start_tcp_server(&self) {
        self.server_runtime.spawn(async {
            start_tcp_server().await;
        });
    }

    fn start_prometheus_export(&self) {
        let prometheus_port = self.config.prometheus_port;
        self.server_runtime.spawn(async move {
            register_prometheus_export(prometheus_port).await;
        });
    }

    fn start_daemon_thread(&self) {
        let config = self.config.clone();
        let client_poll = self.client_poll.clone();
        self.daemon_runtime
            .spawn(async move { report_heartbeat(client_poll, config).await });
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
            register_storage_engine_node(self.client_poll.clone(), self.config.clone()).await;
        });
    }

    async fn stop_server(&self) {
        // unregister node
        unregister_storage_engine_node(self.client_poll.clone(), self.config.clone()).await;
    }
}
