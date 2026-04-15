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
use axum::extract::State;
use broker_core::{cache::NodeCacheManager, cluster::ClusterStorage};
use common_base::http_response::{error_response, success_response};
use common_base::version::version;
use metadata_struct::meta::{node::BrokerNode, status::MetaStatus};
use serde::{Deserialize, Serialize};

pub mod acl;
pub mod blacklist;
pub mod config;
pub mod connector;
pub mod health;
pub mod message;
pub mod offset;
pub mod schema;
pub mod tenant;
pub mod topic;
pub mod user;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClusterInfoResp {
    pub version: String,
    pub cluster_name: String,
    pub start_time: u64,
    pub broker_node_list: Vec<BrokerNode>,
    pub meta: HashMap<String, MetaStatus>,
    pub nodes: HashSet<String>,
}

pub async fn index(State(state): State<Arc<HttpState>>) -> String {
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
