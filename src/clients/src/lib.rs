use std::collections::HashMap;
use lazy_static::lazy_static;
use placement_center::build_placement_center_client;
use protocol::placement_center::placement::placement_center_service_client::PlacementCenterServiceClient;
use tonic::transport::Channel;

pub mod broker_server;
pub mod placement_center;
pub mod storage_engine;

lazy_static! {
    static ref PLACEMENT_CENTER_CLUSTER_CLIENTS: HashMap<String, PlacementCenterServiceClient<Channel>> =
        HashMap::new();
}

pub async fn placement_center_client<'a>(addr: String) -> Option<&'a PlacementCenterServiceClient<Channel>>{
    if PLACEMENT_CENTER_CLUSTER_CLIENTS.contains_key(&addr) {
        if let Some(client) = PLACEMENT_CENTER_CLUSTER_CLIENTS.get(&addr) {
            return Some(client);
        }
    }
    let client = build_placement_center_client(&addr).await;
    return None;
}
