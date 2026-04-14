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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::state::HttpState;
use axum::{extract::State, Json};
use broker_core::{
    cache::NodeCacheManager,
    cluster::ClusterStorage,
    dynamic_config::{
        save_cluster_dynamic_config, update_cluster_dynamic_config, ClusterDynamicConfig,
    },
};
use bytes::Bytes;
use common_base::http_response::{error_response, success_response};
use common_base::version::version;
use metadata_struct::meta::{node::BrokerNode, status::MetaStatus};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterConfigGetReq {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClusterConfigSetReq {
    pub config_type: String,
    pub config: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClusterInfoResp {
    pub version: String,
    pub cluster_name: String,
    pub start_time: u64,
    pub broker_node_list: Vec<BrokerNode>,
    pub meta: HashMap<String, MetaStatus>,
    pub nodes: HashSet<String>,
}

pub mod acl;
pub mod blacklist;
pub mod connector;
pub mod health;
pub mod schema;
pub mod tenant;
pub mod topic;
pub mod user;

pub async fn index(State(_state): State<Arc<HttpState>>) -> String {
    format!("RobustMQ API {}", version())
}

pub async fn cluster_config_set(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ClusterConfigSetReq>,
) -> String {
    let resource_type = match params.config_type.as_str() {
        "MqttSlowSubscribeConfig" => ClusterDynamicConfig::MqttSlowSubscribeConfig,
        "MqttFlappingDetect" => ClusterDynamicConfig::MqttFlappingDetect,
        "MqttProtocol" => ClusterDynamicConfig::MqttProtocol,
        "MqttOfflineMessage" => ClusterDynamicConfig::MqttOfflineMessage,
        "MqttSystemMonitor" => ClusterDynamicConfig::MqttSystemMonitor,
        "MqttSchema" => ClusterDynamicConfig::MqttSchema,
        "MqttLimit" => ClusterDynamicConfig::MqttLimit,
        "ClusterLimit" => ClusterDynamicConfig::ClusterLimit,
        other => {
            return error_response(format!("Unknown config_type: {other}"));
        }
    };

    let config_bytes = Bytes::from(params.config.into_bytes());

    if let Err(e) =
        save_cluster_dynamic_config(&state.client_pool, resource_type, config_bytes.to_vec()).await
    {
        return error_response(format!("Failed to save config: {e}"));
    }

    if let Err(e) = update_cluster_dynamic_config(&state.broker_cache, resource_type, config_bytes)
    {
        return error_response(format!("Failed to update in-memory config: {e}"));
    }

    success_response("success")
}

pub async fn cluster_config_get(State(state): State<Arc<HttpState>>) -> String {
    let broker_config = state.broker_cache.get_cluster_config();
    success_response(broker_config)
}

pub async fn healthy() -> String {
    success_response(true)
}

pub async fn cluster_info(State(state): State<Arc<HttpState>>) -> String {
    let cluster_storage = ClusterStorage::new(state.client_pool.clone());
    let result = match cluster_storage.meta_cluster_status().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let data = match serde_json::from_str::<HashMap<String, MetaStatus>>(&result) {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };
    let cluster_info = ClusterInfoResp {
        version: version(),
        cluster_name: state.broker_cache.cluster_name.clone(),
        start_time: state.broker_cache.get_start_time(),
        broker_node_list: state.broker_cache.node_list(),
        nodes: calc_node_num(&state.broker_cache, &data),
        meta: data,
    };
    success_response(cluster_info)
}

fn calc_node_num(
    broker_cache: &Arc<NodeCacheManager>,
    meta: &HashMap<String, MetaStatus>,
) -> HashSet<String> {
    let mut node_list = HashSet::new();

    for node in broker_cache.node_lists.iter() {
        if let Some(ip) = node.grpc_addr.split(':').next() {
            node_list.insert(ip.to_string());
        }
    }

    for status in meta.values() {
        for node_info in status.membership_config.membership.nodes.values() {
            if let Some(ip) = node_info.rpc_addr.split(':').next() {
                node_list.insert(ip.to_string());
            }
        }
    }

    node_list
}
