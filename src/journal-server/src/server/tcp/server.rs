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

use common_base::config::journal_server::journal_server_conf;
use grpc_clients::poll::ClientPool;
use log::info;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

use crate::core::cache::CacheManager;
use crate::core::command::Command;
use crate::server::connection::NetworkConnectionType;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::{RequestPackage, ResponsePackage};
use crate::server::tcp::handler::handler_process;
use crate::server::tcp::response::response_process;
use crate::server::tcp::tcp_server::acceptor_process;
use crate::server::tcp::tls_server::acceptor_tls_process;

pub async fn start_tcp_server(
    client_poll: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    stop_sx: broadcast::Sender<bool>,
) {
    let conf = journal_server_conf();
    let command = Command::new();

    let proc_config = ProcessorConfig {
        accept_thread_num: conf.tcp_thread.accept_thread_num,
        handler_process_num: conf.tcp_thread.handler_thread_num,
        response_process_num: conf.tcp_thread.response_thread_num,
    };

    let mut server = TcpServer::new(
        command.clone(),
        proc_config,
        stop_sx.clone(),
        connection_manager.clone(),
        cache_manager.clone(),
        client_poll.clone(),
    );
    server.start(conf.network.tcp_port).await;

    let mut server = TcpServer::new(
        command,
        proc_config,
        stop_sx.clone(),
        connection_manager,
        cache_manager,
        client_poll,
    );
    server.start_tls(conf.network.tcps_port).await;
}

struct TcpServer {
    command: Command,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    client_poll: Arc<ClientPool>,
    accept_thread_num: usize,
    handler_process_num: usize,
    response_process_num: usize,
    stop_sx: broadcast::Sender<bool>,
    network_connection_type: NetworkConnectionType,
}

#[derive(Debug, Clone, Copy)]
struct ProcessorConfig {
    pub accept_thread_num: usize,
    pub handler_process_num: usize,
    pub response_process_num: usize,
}

impl TcpServer {
    pub fn new(
        command: Command,
        proc_config: ProcessorConfig,
        stop_sx: broadcast::Sender<bool>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        Self {
            command,
            cache_manager,
            client_poll,
            connection_manager,
            accept_thread_num: proc_config.accept_thread_num,
            handler_process_num: proc_config.handler_process_num,
            response_process_num: proc_config.response_process_num,
            stop_sx,
            network_connection_type: NetworkConnectionType::Tcp,
        }
    }

    pub async fn start(&mut self, port: u32) {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(tl) => tl,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let (request_queue_sx, request_queue_rx) = mpsc::channel::<RequestPackage>(1000);
        let (response_queue_sx, response_queue_rx) = mpsc::channel::<ResponsePackage>(1000);

        let arc_listener = Arc::new(listener);

        acceptor_process(
            self.accept_thread_num,
            self.connection_manager.clone(),
            self.stop_sx.clone(),
            arc_listener.clone(),
            request_queue_sx,
            self.cache_manager.clone(),
            self.network_connection_type.clone(),
        )
        .await;

        handler_process(
            self.handler_process_num,
            request_queue_rx,
            self.connection_manager.clone(),
            response_queue_sx,
            self.stop_sx.clone(),
            self.command.clone(),
        )
        .await;

        response_process(
            self.response_process_num,
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            response_queue_rx,
            self.client_poll.clone(),
            self.stop_sx.clone(),
        )
        .await;

        self.network_connection_type = NetworkConnectionType::Tcp;
        info!("MQTT TCP Server started successfully, listening port: {port}");
    }

    pub async fn start_tls(&mut self, port: u32) {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(tl) => tl,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let (request_queue_sx, request_queue_rx) = mpsc::channel::<RequestPackage>(1000);
        let (response_queue_sx, response_queue_rx) = mpsc::channel::<ResponsePackage>(1000);

        let arc_listener = Arc::new(listener);

        acceptor_tls_process(
            self.accept_thread_num,
            arc_listener.clone(),
            self.stop_sx.clone(),
            self.network_connection_type.clone(),
            self.connection_manager.clone(),
            request_queue_sx,
        )
        .await;

        handler_process(
            self.handler_process_num,
            request_queue_rx,
            self.connection_manager.clone(),
            response_queue_sx,
            self.stop_sx.clone(),
            self.command.clone(),
        )
        .await;

        response_process(
            self.response_process_num,
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            response_queue_rx,
            self.client_poll.clone(),
            self.stop_sx.clone(),
        )
        .await;
        self.network_connection_type = NetworkConnectionType::Tcps;
        info!("MQTT TCP TLS Server started successfully, listening port: {port}");
    }
}
