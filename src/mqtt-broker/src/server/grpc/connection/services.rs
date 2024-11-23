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

use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_connection::mqtt_broker_connection_service_server::MqttBrokerConnectionService;
use protocol::broker_mqtt::broker_mqtt_connection::{ListConnectionReply, ListConnectionRequest};
use tonic::{Request, Response, Status};

use crate::handler::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;

pub struct GrpcConnectionServices {
    #[allow(dead_code)]
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
}

impl GrpcConnectionServices {
    pub fn new(client_pool: Arc<ClientPool>, connection_manager: Arc<ConnectionManager>, cache_manager: Arc<CacheManager>) -> Self {
        GrpcConnectionServices {
            client_pool,
            connection_manager,
            cache_manager,
        }
    }
}

#[tonic::async_trait]
impl MqttBrokerConnectionService for GrpcConnectionServices {
    async fn mqtt_broker_list_connection(
        &self,
        _: Request<ListConnectionRequest>,
    ) -> Result<Response<ListConnectionReply>, Status> {
        let mut reply = ListConnectionReply::default();

        let network_connection_info = self.connection_manager.list_connect();
        let cache_manager = self.cache_manager.clone();

        let mqtt_connection_info_map = cache_manager.connection_info.clone();

        // reply.connections = connection_list;

        let network_connection_to_str = match serde_json::to_string(&network_connection_info) {
            Ok(network_connection_infos) => network_connection_infos,
            Err(e) => {
                format!("Failed to serialize network connection info: {}", e)
            }
        };
        let mqtt_connection_infos = match serde_json::to_string(&mqtt_connection_info_map) {
            Ok(mqtt_connection_infos) => mqtt_connection_infos,
            Err(e) => {
                format!("Failed to serialize mqtt connection info: {}", e)
            }
        };
        reply.network_connections = network_connection_to_str;
        reply.mqtt_connections = mqtt_connection_infos;

        Ok(Response::new(reply))
    }
}
