// Copyright 2023 RobustMQ Team
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

use super::services::GrpcBrokerServices;
use crate::{handler::cache::CacheManager, subscribe::subscribe_manager::SubscribeManager};
use clients::poll::ClientPool;

use log::info;
use protocol::broker_server::generate::mqtt::mqtt_broker_service_server::MqttBrokerServiceServer;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tonic::transport::Server;

pub struct GrpcServer<S> {
    port: u32,
    metadata_cache: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
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
        client_poll: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        return Self {
            port,
            metadata_cache,
            subscribe_manager,
            client_poll,
            message_storage_adapter,
        };
    }
    pub async fn start(&self) {
        let addr = format!("0.0.0.0:{}", self.port).parse().unwrap();
        info!("Broker Grpc Server start success. port:{}", self.port);
        let service_handler = GrpcBrokerServices::new(
            self.metadata_cache.clone(),
            self.subscribe_manager.clone(),
            self.client_poll.clone(),
            self.message_storage_adapter.clone(),
        );
        Server::builder()
            .add_service(MqttBrokerServiceServer::new(service_handler))
            .serve(addr)
            .await
            .unwrap();
    }
}
