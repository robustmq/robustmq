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

use crate::common::error::ResultKafkaBrokerError;
use crate::handler::command::create_command;
use broker_core::cache::BrokerCacheManager;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::connection_manager::ConnectionManager;
use network_server::context::{ProcessorConfig, ServerContext};
use network_server::tcp::server::TcpServer;
use std::sync::Arc;
use tokio::sync::broadcast::{self};
use tracing::{error, info};

#[derive(Clone)]
pub struct KafkaBrokerServerParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub proc_config: ProcessorConfig,
    pub broker_cache: Arc<BrokerCacheManager>,
}

pub struct KafkaBrokerServer {
    server: Arc<TcpServer>,
    connection_manager: Arc<ConnectionManager>,
    main_stop: broadcast::Sender<bool>,
    inner_stop: broadcast::Sender<bool>,
}

impl KafkaBrokerServer {
    pub fn new(params: KafkaBrokerServerParams, main_stop: broadcast::Sender<bool>) -> Self {
        let (inner_stop, _) = broadcast::channel(2);
        let command = create_command();
        let context = ServerContext {
            connection_manager: params.connection_manager.clone(),
            client_pool: params.client_pool.clone(),
            command: command.clone(),
            network_type: NetworkConnectionType::Tcp,
            proc_config: params.proc_config,
            stop_sx: inner_stop.clone(),
            broker_cache: params.broker_cache.clone(),
        };

        let server = Arc::new(TcpServer::new(context.clone()));

        KafkaBrokerServer {
            server,
            connection_manager: params.connection_manager,
            main_stop,
            inner_stop,
        }
    }

    pub async fn start(&self) {
        self.start_server();

        self.awaiting_stop().await;
    }

    fn start_server(&self) {
        let conf = broker_config();
        let server = self.server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.start(false, conf.kafka_config.port).await {
                error!("Kafka server start fail, error:{}", e);
            }
        });
    }

    pub async fn awaiting_stop(&self) {
        let server = self.server.clone();
        let mut recv = self.main_stop.subscribe();
        let connection_manager = self.connection_manager.clone();
        let raw_inner_stop = self.inner_stop.clone();
        match recv.recv().await {
            Ok(_) => {
                info!("Broker has stopped, starting to stop the kafka service...");
                server.stop().await;
                match raw_inner_stop.send(true) {
                    Ok(_) => {
                        info!("Process stop signal was sent successfully.");
                        if let Err(e) = KafkaBrokerServer::stop_server(&connection_manager).await {
                            error!("{}", e);
                        }
                        info!("Kafka Service has been successfully stopped.");
                    }
                    Err(_) => {
                        error!("Failed to send stop signal");
                    }
                }
            }
            Err(e) => {
                error!("recv error {}", e);
            }
        }
    }

    async fn stop_server(connection_manager: &Arc<ConnectionManager>) -> ResultKafkaBrokerError {
        connection_manager.close_all_connect().await;
        info!("Kafka broker connections have been successfully closed.");
        Ok(())
    }
}
