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

use crate::state::HttpState;
use axum::{extract::State, Json};
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
    pub meta: MetaStatus,
}
use broker_core::cluster::ClusterStorage;
use common_base::{
    enum_type::feature_type::FeatureType,
    http_response::{error_response, success_response},
    version::version,
};
use std::str::FromStr;

pub async fn index(State(_state): State<Arc<HttpState>>) -> String {
    format!("RobustMQ API {}", version())
}

pub async fn cluster_config_set(
    State(_state): State<Arc<HttpState>>,
    Json(params): Json<ClusterConfigSetReq>,
) -> String {
    match FeatureType::from_str(params.config_type.as_str()) {
        Ok(FeatureType::SlowSubscribe) => {
            // let mut config = cache_manager.get_slow_sub_config();
            // config.enable = request.is_enable;
            // cache_manager.update_slow_sub_config(config.clone());
            // save_cluster_dynamic_config(
            //     client_pool,
            //     ClusterDynamicConfig::MqttFlappingDetect,
            //     config.encode(),
            // )
            // .await?;
        }

        Ok(FeatureType::OfflineMessage) => {
            // let mut config = cache_manager.get_offline_message_config();
            // config.enable = request.is_enable;
            // cache_manager.update_offline_message_config(config.clone());
            // save_cluster_dynamic_config(
            //     client_pool,
            //     ClusterDynamicConfig::MqttOfflineMessage,
            //     config.encode(),
            // )
            // .await?;
        }

        Ok(FeatureType::SystemAlarm) => {}

        Ok(FeatureType::FlappingDetect) => {}

        Err(e) => {
            return error_response(format!("Failed to parse feature type: {e}"));
        }
    }
    success_response("success")
}

pub async fn cluster_config_get(State(state): State<Arc<HttpState>>) -> String {
    let broker_config = state.broker_cache.get_cluster_config().await;
    success_response(broker_config)
}

pub async fn cluster_info(State(state): State<Arc<HttpState>>) -> String {
    let cluster_storage = ClusterStorage::new(state.client_pool.clone());
    let meta_data = match cluster_storage.meta_cluster_status().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };
    let data = match serde_json::from_str::<MetaStatus>(&meta_data) {
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
        meta: data,
    };
    success_response(cluster_info)
}
