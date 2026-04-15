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

use crate::state::HttpState;
use axum::{extract::State, Json};
use broker_core::dynamic_config::{
    save_cluster_dynamic_config, update_cluster_dynamic_config, ClusterDynamicConfig,
};
use bytes::Bytes;
use common_base::http_response::{error_response, success_response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterConfigGetReq {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClusterConfigSetReq {
    pub config_type: String,
    pub config: String,
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
