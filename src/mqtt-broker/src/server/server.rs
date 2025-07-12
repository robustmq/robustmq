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

use crate::common::types::ResultMqttBrokerError;
use crate::{
    handler::{cache::CacheManager, command::Command},
    security::AuthDriver,
    server::common::{connection::NetworkConnectionType, connection_manager::ConnectionManager},
    server::tcp::v1::server::{ProcessorConfig, TcpServer},
    subscribe::manager::SubscribeManager,
};
use common_config::mqtt::broker_mqtt_conf;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::sync::broadcast;

pub struct Server {
    tcp_server: TcpServer,
    tls_server: TcpServer,
}

impl Server {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<CacheManager>,
        connection_manager: Arc<ConnectionManager>,
        message_storage_adapter: ArcStorageAdapter,
        delay_message_manager: Arc<DelayMessageManager>,
        schema_manager: Arc<SchemaRegisterManager>,
        client_pool: Arc<ClientPool>,
        stop_sx: broadcast::Sender<bool>,
        auth_driver: Arc<AuthDriver>,
    ) -> Self {
        let conf = broker_mqtt_conf();
        let command = Command::new(
            cache_manager.clone(),
            message_storage_adapter.clone(),
            delay_message_manager.clone(),
            subscribe_manager.clone(),
            client_pool.clone(),
            connection_manager.clone(),
            schema_manager.clone(),
            auth_driver.clone(),
        );

        let proc_config = ProcessorConfig {
            accept_thread_num: conf.network_thread.accept_thread_num,
            handler_process_num: conf.network_thread.handler_thread_num,
            response_process_num: conf.network_thread.response_thread_num,
            channel_size: conf.network_thread.queue_size,
        };

        let tcp_server = TcpServer::new(
            connection_manager.clone(),
            subscribe_manager.clone(),
            cache_manager.clone(),
            client_pool.clone(),
            command.clone(),
            NetworkConnectionType::Tcp,
            proc_config,
            stop_sx.clone(),
        );

        let tls_server = TcpServer::new(
            connection_manager.clone(),
            subscribe_manager.clone(),
            cache_manager.clone(),
            client_pool.clone(),
            command.clone(),
            NetworkConnectionType::Tls,
            proc_config,
            stop_sx.clone(),
        );
        Server {
            tcp_server,
            tls_server,
        }
    }

    pub async fn start(&self) -> ResultMqttBrokerError {
        self.tcp_server.start(false).await?;
        self.tls_server.start(true).await?;
        Ok(())
    }

    pub async fn stop(&self) {
        self.tcp_server.stop().await;
        self.tls_server.stop().await;
    }
}
