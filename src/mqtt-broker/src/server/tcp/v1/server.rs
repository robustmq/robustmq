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

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use storage_adapter::storage::StorageAdapter;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::info;

use crate::handler::cache::CacheManager;
use crate::handler::command::Command;
use crate::handler::error::MqttBrokerError;
use crate::server::connection::NetworkConnectionType;
use crate::server::connection_manager::ConnectionManager;
use crate::server::tcp::v1::channel::RequestChannel;
use crate::server::tcp::v1::handler::handler_process;
use crate::server::tcp::v1::response::response_process;
use crate::server::tcp::v1::tcp_server::acceptor_process;
use crate::server::tcp::v1::tls_server::acceptor_tls_process;
use crate::subscribe::manager::SubscribeManager;

#[derive(Debug, Clone, Copy)]
pub struct ProcessorConfig {
    pub accept_thread_num: usize,
    pub handler_process_num: usize,
    pub response_process_num: usize,
    pub channel_size: usize,
}

// U: codec: encoder + decoder
// S: message storage adapter
pub struct TcpServer<S> {
    command: Command<S>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_pool: Arc<ClientPool>,
    proc_config: ProcessorConfig,
    stop_sx: broadcast::Sender<bool>,
    network_type: NetworkConnectionType,
    request_channel: Arc<RequestChannel>,
}

impl<S> TcpServer<S>
where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<CacheManager>,
        client_pool: Arc<ClientPool>,
        command: Command<S>,
        network_type: NetworkConnectionType,
        proc_config: ProcessorConfig,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        info!("process thread num: {:?}", proc_config);
        let request_channel = Arc::new(RequestChannel::new(proc_config.channel_size));
        Self {
            network_type,
            command,
            cache_manager,
            client_pool,
            connection_manager,
            proc_config,
            stop_sx,
            subscribe_manager,
            request_channel,
        }
    }

    pub async fn start(&self, tls: bool) -> Result<(), MqttBrokerError> {
        let conf = broker_mqtt_conf();
        let port = if tls {
            conf.network_port.tcps_port
        } else {
            conf.network_port.tcp_port
        };

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        let arc_listener = Arc::new(listener);
        let request_recv_channel = self.request_channel.create_request_channel();
        let response_recv_channel = self.request_channel.create_response_channel();

        if tls {
            acceptor_tls_process(
                self.proc_config.accept_thread_num,
                arc_listener.clone(),
                self.stop_sx.clone(),
                self.network_type.clone(),
                self.connection_manager.clone(),
                self.request_channel.clone(),
            )
            .await?;
        } else {
            acceptor_process(
                self.proc_config.accept_thread_num,
                self.connection_manager.clone(),
                self.stop_sx.clone(),
                arc_listener.clone(),
                self.request_channel.clone(),
                self.network_type.clone(),
            )
            .await;
        }

        handler_process(
            self.proc_config.handler_process_num,
            request_recv_channel,
            self.connection_manager.clone(),
            self.command.clone(),
            self.request_channel.clone(),
            self.stop_sx.clone(),
        )
        .await;

        response_process(
            self.proc_config.response_process_num,
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.subscribe_manager.clone(),
            response_recv_channel,
            self.client_pool.clone(),
            self.request_channel.clone(),
            self.stop_sx.clone(),
        )
        .await;

        info!("MQTT TCP Server started successfully, listening port: {port}");
        Ok(())
    }

    pub async fn stop(&self) {}
}
