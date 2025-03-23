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
pub mod connector;
pub mod subscribe;
pub mod topic;
pub mod user;

use crate::handler::cache::CacheManager;
use crate::handler::flapping_detect::enable_flapping_detect;
use crate::observability::slow::sub::{enable_slow_sub, read_slow_sub_record, SlowSubData};
use crate::server::connection_manager::ConnectionManager;
use crate::{handler::error::MqttBrokerError, storage::cluster::ClusterStorage};
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::tools::serialize_value;
use common_base::utils::file_utils::get_project_root;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, EnableFlappingDetectReply, EnableFlappingDetectRequest,
    EnableSlowSubScribeReply, EnableSlowSubscribeRequest, ListConnectionRaw, ListConnectionReply,
    ListSlowSubScribeRaw, ListSlowSubscribeReply, ListSlowSubscribeRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn cluster_status_by_req(
    client_pool: &Arc<ClientPool>,
) -> Result<ClusterStatusReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

    let mut broker_node_list = Vec::new();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage.node_list().await?;
    for node in data {
        broker_node_list.push(format!("{}@{}", node.node_ip, node.node_id));
    }
    Ok(ClusterStatusReply {
        nodes: broker_node_list,
        cluster_name: config.cluster_name.clone(),
    })
}

pub async fn enable_flapping_detect_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<EnableFlappingDetectRequest>,
) -> Result<Response<EnableFlappingDetectReply>, Status> {
    let req = request.into_inner();

    match enable_flapping_detect(cache_manager, req).await {
        Ok(_) => Ok(Response::new(EnableFlappingDetectReply {
            is_enable: req.is_enable,
        })),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub fn list_connection_by_req(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
) -> Result<Response<ListConnectionReply>, Status> {
    let mut reply = ListConnectionReply::default();
    let mut list_connection_raw: Vec<ListConnectionRaw> = Vec::new();
    for (key, value) in connection_manager.list_connect() {
        if let Some(mqtt_value) = cache_manager.connection_info.clone().get(&key) {
            let mqtt_info = serialize_value(mqtt_value.value())?;
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

pub async fn enable_slow_subscribe_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<EnableSlowSubscribeRequest>,
) -> Result<Response<EnableSlowSubScribeReply>, Status> {
    let subscribe_request = request.into_inner();

    match enable_slow_sub(cache_manager, subscribe_request.is_enable).await {
        Ok(_) => Ok(Response::new(EnableSlowSubScribeReply {
            is_enable: subscribe_request.is_enable,
        })),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub fn list_slow_subscribe_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<ListSlowSubscribeRequest>,
) -> Result<Response<ListSlowSubscribeReply>, Status> {
    let list_slow_subscribe_request = request.into_inner();
    let mut list_slow_subscribe_raw: Vec<ListSlowSubScribeRaw> = Vec::new();
    let mqtt_config = broker_mqtt_conf();
    if cache_manager.get_slow_sub_config().enable {
        let path = mqtt_config.log.log_path.clone();
        let path_buf = get_project_root()?.join(path.replace("./", "") + "/slow_sub.log");
        let deque = read_slow_sub_record(list_slow_subscribe_request, path_buf)?;
        for slow_sub_data in deque {
            match serde_json::from_str::<SlowSubData>(slow_sub_data.as_str()) {
                Ok(data) => {
                    let raw = ListSlowSubScribeRaw {
                        client_id: data.client_id,
                        topic: data.topic,
                        time_ms: data.time_ms,
                        node_info: data.node_info,
                        create_time: data.create_time,
                        sub_name: data.sub_name,
                    };
                    list_slow_subscribe_raw.push(raw);
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }
    Ok(Response::new(ListSlowSubscribeReply {
        list_slow_subscribe_raw,
    }))
}
