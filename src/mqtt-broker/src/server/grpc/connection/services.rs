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
    async fn mqtt_broker_list_connection(&self, request: Request<ListConnectionRequest>) -> Result<Response<ListConnectionReply>, Status> {
        let mut reply = ListConnectionReply::default();

        let mut connection_list = Vec::new();
        
        let connection_manager = ConnectionManager::new(self.cache_manager.clone());
        let dash_map = connection_manager.list_connection().await;
           dash_map.for_each(|(k, v)| {
               connection_list.push(v.to_string());
           });
        reply.connections = connection_list;
        Ok(Response::new(reply))
    }
}