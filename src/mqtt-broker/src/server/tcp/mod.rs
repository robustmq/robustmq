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

use super::connection_manager::ConnectionManager;
use crate::{
    handler::{cache::CacheManager, command::Command},
    security::AuthDriver,
    server::packet::{RequestPackage, ResponsePackage},
    subscribe::subscribe_manager::SubscribeManager,
};
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use handler::handler_process;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tcp_server::TcpServer;
use tls_server::acceptor_tls_process;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::{
    io, select,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tokio_util::codec::{FramedRead, FramedWrite};

pub mod handler;
pub mod response;
pub mod tcp_server;
pub mod tls_server;

pub async fn start_tcp_server<S>(
    sucscribe_manager: Arc<SubscribeManager>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
    auth_driver: Arc<AuthDriver>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let conf = broker_mqtt_conf();
    let command = Command::new(
        cache_manager.clone(),
        message_storage_adapter.clone(),
        sucscribe_manager.clone(),
        client_poll.clone(),
        connection_manager.clone(),
        auth_driver.clone(),
    );

    let mut server = TcpServer::<S>::new(
        command.clone(),
        conf.tcp_thread.accept_thread_num,
        conf.tcp_thread.handler_thread_num,
        conf.tcp_thread.response_thread_num,
        stop_sx.clone(),
        connection_manager.clone(),
        sucscribe_manager.clone(),
        cache_manager.clone(),
        client_poll.clone(),
    );
    server.start(conf.network.tcp_port).await;

    let mut server = TcpServer::<S>::new(
        command,
        conf.tcp_thread.accept_thread_num,
        conf.tcp_thread.handler_thread_num,
        conf.tcp_thread.response_thread_num,
        stop_sx.clone(),
        connection_manager,
        sucscribe_manager.clone(),
        cache_manager,
        client_poll,
    );
    server.start_tls(conf.network.tcps_port).await;
}

async fn start<S>(
    port: u32,
    sucscribe_manager: Arc<SubscribeManager>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
    auth_driver: Arc<AuthDriver>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
        Ok(tl) => tl,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };
    let (request_queue_sx, request_queue_rx) = mpsc::channel::<RequestPackage>(1000);
    let (response_queue_sx, response_queue_rx) = mpsc::channel::<ResponsePackage>(1000);

    let arc_listener = Arc::new(listener);
    self.acceptor_process(arc_listener.clone(), request_queue_sx).await;

    handler_process(
        self.handler_process_num,
        request_queue_rx,
        self.connection_manager.clone(),
        response_queue_sx,
        self.stop_sx.clone(),
        self.command.clone(),
    )
    .await;
    self.response_process(response_queue_rx).await;
    self.network_connection_type = NetworkConnectionType::TCP;
    info!("MQTT TCP Server started successfully, listening port: {port}");
}

async fn start_tls<S>(
    port: u32,
    sucscribe_manager: Arc<SubscribeManager>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
    auth_driver: Arc<AuthDriver>,
) where
S: StorageAdapter + Sync + Send + 'static + Clone,{
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

    self.response_process(response_queue_rx).await;
    self.network_connection_type = NetworkConnectionType::TCPS;
    info!("MQTT TCP TLS Server started successfully, listening port: {port}");
}
