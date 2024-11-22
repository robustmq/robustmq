use crate::handler::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_connection::mqtt_broker_connection_service_server::MqttBrokerConnectionService;
use protocol::broker_mqtt::broker_mqtt_connection::{ListConnectionReply, ListConnectionRequest};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcConnectionServices {
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
}

impl GrpcConnectionServices {
    pub fn new(client_pool: Arc<ClientPool>, cache_manager: Arc<CacheManager>) -> Self {
        GrpcConnectionServices {
            client_pool,
            cache_manager,
        }
    }
}

impl MqttBrokerConnectionService for GrpcConnectionServices {
    async fn mqtt_broker_list_connection(&self, _request: Request<ListConnectionRequest>) -> Result<Response<ListConnectionReply>, Status> {
        let mut reply = ListConnectionReply::default();

        let connection_manager = ConnectionManager::new(self.cache_manager.clone());
        let network_connection_info = connection_manager.list_connect();
        // let cache_manager = self.cache_manager.clone();

        // let mqtt_connection_infos = cache_manager.connection_info.clone();

        // reply.connections = connection_list;

        let network_connection_to_str = match serde_json::to_string(&network_connection_info) {
            Ok(network_connection_infos) => {
                network_connection_infos
            }
            Err(e) => {
                format!("Failed to serialize network connection info: {}", e.to_string())
            }
        };
        reply.network_connections = network_connection_to_str;

        Ok(Response::new(reply))
    }
}