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
use std::time::Duration;

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use storage_adapter::storage::StorageAdapter;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{error, info};

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::handler::command::Command;
use crate::observability::metrics::server::record_broker_thread_num;
use crate::server::common::channel::RequestChannel;
use crate::server::common::connection::NetworkConnectionType;
use crate::server::common::connection_manager::ConnectionManager;
use crate::server::common::handler::handler_process;
use crate::server::common::response::response_process;
use crate::server::tcp::v1::tcp_acceptor::acceptor_process;
use crate::server::tcp::v1::tls_acceptor::acceptor_tls_process;
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
    network_type: NetworkConnectionType,
    request_channel: Arc<RequestChannel>,
    acceptor_stop_send: broadcast::Sender<bool>,
    stop_sx: broadcast::Sender<bool>,
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
        info!(
            "network type:{}, process thread num: {:?}",
            network_type, proc_config
        );
        let request_channel = Arc::new(RequestChannel::new(proc_config.channel_size));
        let (acceptor_stop_send, _) = broadcast::channel(2);
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
            acceptor_stop_send,
        }
    }

    pub async fn start(&self, tls: bool) -> ResultMqttBrokerError {
        let conf = broker_mqtt_conf();
        let port = if tls {
            conf.network_port.tcps_port
        } else {
            conf.network_port.tcp_port
        };

        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        let arc_listener = Arc::new(listener);
        let request_recv_channel = self
            .request_channel
            .create_request_channel(&self.network_type);
        let response_recv_channel = self
            .request_channel
            .create_response_channel(&self.network_type);

        if tls {
            acceptor_tls_process(
                self.proc_config.accept_thread_num,
                arc_listener.clone(),
                self.acceptor_stop_send.clone(),
                self.network_type.clone(),
                self.connection_manager.clone(),
                self.request_channel.clone(),
            )
            .await?;
        } else {
            acceptor_process(
                self.proc_config.accept_thread_num,
                self.connection_manager.clone(),
                self.acceptor_stop_send.clone(),
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
            self.network_type.clone(),
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
            self.network_type.clone(),
            self.stop_sx.clone(),
        )
        .await;

        self.record_pre_server_metrics();
        info!("MQTT TCP Server started successfully, listening port: {port}");
        Ok(())
    }

    pub async fn stop(&self) {
        // Stop the acceptor thread and refuse to receive new data
        if let Err(e) = self.acceptor_stop_send.send(true) {
            error!(
                "Network type:{}, Failed to stop the acceptor thread. Error message: {:?}",
                self.network_type, e
            );
        }

        // Determine whether the channel for request processing is empty. If it is empty,
        // it indicates that the request packet has been processed and subsequent stop logic can be carried out.

        loop {
            sleep(Duration::from_secs(1)).await;
            let mut flag = false;

            // request main channel
            let cap = self
                .request_channel
                .get_request_send_channel(&self.network_type)
                .capacity();
            if cap != self.proc_config.channel_size {
                info!("Request main queue is not empty, current length {}, waiting for request packet processing to complete....", self.proc_config.channel_size - cap);
                flag = true;
            }

            // request child channel
            for (index, send) in self.request_channel.handler_child_channels.clone() {
                let cap = send.capacity();
                if cap != self.proc_config.channel_size {
                    info!("Request child queue {} is not empty, current length {}, waiting for request packet processing to complete....", index, self.proc_config.channel_size - cap);
                    flag = true;
                }
            }

            // response main channel
            if self
                .request_channel
                .get_response_send_channel(&self.network_type)
                .capacity()
                != self.proc_config.channel_size
            {
                info!("Response main queue is not empty, current length {}, waiting for response packet processing to complete....", self.proc_config.channel_size - cap);
                flag = true;
            }

            // response child channel
            for (index, send) in self.request_channel.response_child_channels.clone() {
                let cap = send.capacity();
                if cap != self.proc_config.channel_size {
                    info!("Response child queue {} is not empty, current length {}, waiting for response packet processing to complete....", index, self.proc_config.channel_size - cap);
                    flag = true;
                }
            }

            if !flag {
                info!("[{}] All the request packets have been processed. Start to stop the request processing thread.", self.network_type);
                break;
            }
        }
    }

    // Record the metrics before the service starts,
    // which usually happens when you need to record the static metrics of the server
    fn record_pre_server_metrics(&self) {
        // number of execution threads of the server
        record_broker_thread_num(
            &self.network_type,
            (
                self.proc_config.accept_thread_num,
                self.proc_config.handler_process_num,
                self.proc_config.response_process_num,
            ),
        );
    }
}
