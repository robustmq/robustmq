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

pub mod acl;
pub mod blacklist;
pub mod client;
pub mod cluster;
pub mod connector;
pub mod observability;
pub mod query;
pub mod schema;
pub mod session;
pub mod subscribe;
pub mod topic;
pub mod user;

use crate::handler::cache::CacheManager;
use crate::handler::flapping_detect::enable_flapping_detect;
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::manager::SubscribeManager;
use crate::{handler::error::MqttBrokerError, storage::cluster::ClusterStorage};

use common_base::tools::serialize_value;
use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, EnableFlappingDetectReply, EnableFlappingDetectRequest, ListConnectionRaw,
    ListConnectionReply,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn cluster_status_by_req(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
) -> Result<ClusterStatusReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

    let mut broker_node_list = Vec::new();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage.node_list().await?;
    for node in data {
        broker_node_list.push(format!("{}@{}", node.node_ip, node.node_id));
    }

    let subsceibe_info = subscribe_manager.snapshot_info();

    Ok(ClusterStatusReply {
        nodes: broker_node_list,
        cluster_name: config.cluster_name.clone(),
        subscribe_info: serde_json::to_string(&subsceibe_info)?,
    })
}

pub async fn enable_flapping_detect_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<EnableFlappingDetectRequest>,
) -> Result<Response<EnableFlappingDetectReply>, Status> {
    let req = request.into_inner();

    match enable_flapping_detect(client_pool, cache_manager, req).await {
        Ok(_) => Ok(Response::new(EnableFlappingDetectReply {
            is_enable: req.is_enable,
        })),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn list_connection_by_req(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
) -> Result<Response<ListConnectionReply>, Status> {
    let mut reply = ListConnectionReply::default();
    let mut list_connection_raw: Vec<ListConnectionRaw> = Vec::new();
    for (key, value) in connection_manager.list_connect() {
        if let Some(mqtt_value) = cache_manager.get_connection(key) {
            let mqtt_info = serialize_value(&mqtt_value)?;
            let raw = ListConnectionRaw {
                connection_id: value.connection_id,
                connection_type: value.connection_type.to_string(),
                protocol: match value.protocol {
                    Some(protocol) => protocol.into(),
                    None => "None".to_string(),
                },
                source_addr: value.addr.to_string(),
                info: mqtt_info,
            };
            list_connection_raw.push(raw);
        }
    }
    reply.list_connection_raw = list_connection_raw;
    Ok(Response::new(reply))
}
