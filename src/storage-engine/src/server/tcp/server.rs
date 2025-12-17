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

use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use rocksdb_engine::rocksdb::RocksDBEngine;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

use crate::core1::cache::CacheManager;
use crate::handler::command::Command;
use crate::segment::manager::SegmentFileManager;
use crate::server::connection::NetworkConnectionType;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::{RequestPackage, ResponsePackage};
use crate::server::tcp::handler::handler_process;
use crate::server::tcp::response::response_process;
use crate::server::tcp::tcp_server::acceptor_process;

/// Start the TCP server in the journal engine from the config fire.
pub async fn start_tcp_server(
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_sx: broadcast::Sender<bool>,
) {
    let conf = broker_config();
    let command = Command::new(
        client_pool.clone(),
        cache_manager.clone(),
        segment_file_manager,
        rocksdb_engine_handler,
    );

    let proc_config = ProcessorConfig {
        accept_thread_num: conf.network.accept_thread_num,
        handler_process_num: conf.network.handler_thread_num,
        response_process_num: conf.network.response_thread_num,
    };

    let mut server = TcpServer::new(
        command.clone(),
        proc_config,
        stop_sx.clone(),
        connection_manager.clone(),
        cache_manager.clone(),
        client_pool.clone(),
    );
    server.start().await;
}

struct TcpServer {
    command: Command,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
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
        client_pool: Arc<ClientPool>,
    ) -> Self {
        Self {
            command,
            cache_manager,
            client_pool,
            connection_manager,
            accept_thread_num: proc_config.accept_thread_num,
            handler_process_num: proc_config.handler_process_num,
            response_process_num: proc_config.response_process_num,
            stop_sx,
            network_connection_type: NetworkConnectionType::Tcp,
        }
    }

    pub async fn start(&mut self) {
        let conf = broker_config();
        let addr = format!("0.0.0.0:{}", conf.journal_server.tcp_port);
        let listener = match TcpListener::bind(&addr).await {
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
            self.client_pool.clone(),
            self.stop_sx.clone(),
        )
        .await;

        self.network_connection_type = NetworkConnectionType::Tcp;
        info!("Journal Engine Server started successfully, addr: {addr}");
    }
}
