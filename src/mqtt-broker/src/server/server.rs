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
use crate::handler::command::create_command;
use crate::server::tcp::server::{ProcessorConfig, TcpServer, TcpServerContext};
use crate::{
    handler::{cache::CacheManager, command::CommandContext},
    security::AuthDriver,
    subscribe::manager::SubscribeManager,
};
use common_config::broker::broker_config;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::connection_manager::ConnectionManager;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::sync::broadcast;

pub struct Server {
    tcp_server: TcpServer,
    tls_server: TcpServer,
}

#[derive(Clone)]
pub struct ServerContext {
    pub subscribe_manager: Arc<SubscribeManager>,
    pub cache_manager: Arc<CacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub message_storage_adapter: ArcStorageAdapter,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub client_pool: Arc<ClientPool>,
    pub stop_sx: broadcast::Sender<bool>,
    pub auth_driver: Arc<AuthDriver>,
}

impl Server {
    pub fn new(context: ServerContext) -> Self {
        let conf = broker_config();
        let command_context = CommandContext {
            cache_manager: context.cache_manager.clone(),
            message_storage_adapter: context.message_storage_adapter.clone(),
            delay_message_manager: context.delay_message_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            client_pool: context.client_pool.clone(),
            connection_manager: context.connection_manager.clone(),
            schema_manager: context.schema_manager.clone(),
            auth_driver: context.auth_driver.clone(),
        };
        let command = create_command(command_context);

        let proc_config = ProcessorConfig {
            accept_thread_num: conf.network.accept_thread_num,
            handler_process_num: conf.network.handler_thread_num,
            response_process_num: conf.network.response_thread_num,
            channel_size: conf.network.queue_size,
        };

        let tcp_server = TcpServer::new(TcpServerContext {
            connection_manager: context.connection_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            cache_manager: context.cache_manager.clone(),
            client_pool: context.client_pool.clone(),
            command: command.clone(),
            network_type: NetworkConnectionType::Tcp,
            proc_config,
            stop_sx: context.stop_sx.clone(),
        });

        let tls_server = TcpServer::new(TcpServerContext {
            connection_manager: context.connection_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            cache_manager: context.cache_manager.clone(),
            client_pool: context.client_pool.clone(),
            command: command.clone(),
            network_type: NetworkConnectionType::Tls,
            proc_config,
            stop_sx: context.stop_sx.clone(),
        });
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
