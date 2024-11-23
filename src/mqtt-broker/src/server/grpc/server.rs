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
use protocol::broker_mqtt::broker_mqtt_connection::mqtt_broker_connection_service_server::MqttBrokerConnectionServiceServer;
use protocol::broker_mqtt::broker_mqtt_placement::mqtt_broker_placement_service_server::MqttBrokerPlacementServiceServer;
use storage_adapter::storage::StorageAdapter;
use tonic::transport::Server;

use super::placement::GrpcPlacementServices;
use crate::handler::cache::CacheManager;
use crate::server::grpc::admin::services::GrpcAdminServices;
use crate::server::grpc::connection::services::GrpcConnectionServices;
use crate::subscribe::subscribe_manager::SubscribeManager;

pub struct GrpcServer<S> {
    port: u32,
    metadata_cache: Arc<CacheManager>,
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
        client_pool: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        Self {
            port,
            metadata_cache,
            subscribe_manager,
            client_pool,
            message_storage_adapter,
        }
    }
    pub async fn start(&self) -> Result<(), CommonError> {
        let addr = format!("0.0.0.0:{}", self.port).parse()?;
        info!("Broker Grpc Server start success. port:{}", self.port);
        let placement_handler = GrpcPlacementServices::new(
            self.metadata_cache.clone(),
            self.subscribe_manager.clone(),
            self.client_pool.clone(),
            self.message_storage_adapter.clone(),
        );
        let admin_handler =
            GrpcAdminServices::new(self.client_pool.clone(), self.metadata_cache.clone());
        let connection_handler =
            GrpcConnectionServices::new(self.client_pool.clone(), self.metadata_cache.clone());
        Server::builder()
            .add_service(MqttBrokerPlacementServiceServer::new(placement_handler))
            .add_service(MqttBrokerAdminServiceServer::new(admin_handler))
            .add_service(MqttBrokerConnectionServiceServer::new(connection_handler))
            .serve(addr)
            .await?;
        Ok(())
    }
}
