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

use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use log::info;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminServiceServer;
use protocol::broker_mqtt::broker_mqtt_inner::mqtt_broker_inner_service_server::MqttBrokerInnerServiceServer;
use storage_adapter::storage::StorageAdapter;
use tonic::transport::Server;

use super::inner::GrpcInnerServices;
use crate::handler::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;
use crate::server::grpc::admin::services::GrpcAdminServices;
use crate::subscribe::subscribe_manager::SubscribeManager;

pub struct GrpcServer<S> {
    port: u32,
    metadata_cache: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_pool: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
}

impl<S> GrpcServer<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        port: u32,
        metadata_cache: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        client_pool: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        Self {
            port,
            metadata_cache,
            connection_manager,
            subscribe_manager,
            client_pool,
            message_storage_adapter,
        }
    }
    pub async fn start(&self) -> Result<(), CommonError> {
        let addr = format!("0.0.0.0:{}", self.port).parse()?;
        info!("Broker Grpc Server start success. port:{}", self.port);
        let inner_handler = GrpcInnerServices::new(
            self.metadata_cache.clone(),
            self.subscribe_manager.clone(),
            self.client_pool.clone(),
            self.message_storage_adapter.clone(),
        );
        let admin_handler = GrpcAdminServices::new(
            self.client_pool.clone(),
            self.metadata_cache.clone(),
            self.connection_manager.clone(),
        );
        Server::builder()
            .add_service(MqttBrokerInnerServiceServer::new(inner_handler))
            .add_service(MqttBrokerAdminServiceServer::new(admin_handler))
            .serve(addr)
            .await?;
        Ok(())
    }
}
