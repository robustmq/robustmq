use std::collections::HashMap;

use common::errors::RobustMQError;
use protocol::placement_center::placement::placement_center_service_client::PlacementCenterServiceClient;

pub mod broker_server;
pub mod placement_center;
pub mod storage_engine;

pub async fn build_storage_engine_client(addr: String) -> Result<CommonReply, RobustMQError>{
    let mut client = match PlacementCenterServiceClient::connect(format!("http://{}", addr)).await {
        Ok(client) => client,
        Err(err) => return (RobustMQError::TonicTransport(err)),
    };
}