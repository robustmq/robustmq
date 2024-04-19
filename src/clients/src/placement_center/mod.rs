use common_base::errors::RobustMQError;
use protocol::placement_center::generate::{
    engine::engine_service_client::EngineServiceClient, kv::kv_service_client::KvServiceClient,
    placement::placement_center_service_client::PlacementCenterServiceClient,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::ClientPool;

pub enum PlacementCenterInterface {
    Set,
    Get,
    Delete,
    Exists,
    RegisterNode,
}

pub mod engine;
pub mod kv;
pub mod placement;



pub async fn engine_client(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
) -> Result<EngineServiceClient<Channel>, RobustMQError> {
    let mut poll = client_poll.lock().await;
    match poll.get_engine_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn kv_client(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
) -> Result<KvServiceClient<Channel>, RobustMQError> {
    let mut poll = client_poll.lock().await;
    match poll.get_kv_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}
