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

use std::collections::HashSet;
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
    pub nodes: HashSet<String>,
}
use broker_core::{cache::BrokerCacheManager, cluster::ClusterStorage};
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
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ClusterConfigSetReq>,
) -> String {
    let cache_manager = &state.mqtt_context.cache_manager;
    let client_pool = &state.client_pool;

    match FeatureType::from_str(params.config_type.as_str()) {
        Ok(FeatureType::SlowSubscribe) => {
            match serde_json::from_str::<common_config::config::MqttSlowSubscribeConfig>(
                params.config.as_str(),
            ) {
                Ok(config) => {
                    cache_manager.update_slow_sub_config(config.clone()).await;
                    if let Err(e) = mqtt_broker::core::dynamic_config::save_cluster_dynamic_config(
                        client_pool,
                        mqtt_broker::core::dynamic_config::ClusterDynamicConfig::MqttSlowSubscribeConfig,
                        serde_json::to_vec(&config).unwrap(),
                    )
                    .await
                    {
                        return error_response(format!("Failed to persist config: {e}"));
                    }
                }
                Err(e) => {
                    return error_response(format!("Failed to parse config: {e}"));
                }
            }
        }

        Ok(FeatureType::OfflineMessage) => {
            match serde_json::from_str::<common_config::config::MqttOfflineMessage>(
                params.config.as_str(),
            ) {
                Ok(config) => {
                    cache_manager
                        .update_offline_message_config(config.clone())
                        .await;
                    if let Err(e) = mqtt_broker::core::dynamic_config::save_cluster_dynamic_config(
                        client_pool,
                        mqtt_broker::core::dynamic_config::ClusterDynamicConfig::MqttOfflineMessage,
                        serde_json::to_vec(&config).unwrap(),
                    )
                    .await
                    {
                        return error_response(format!("Failed to persist config: {e}"));
                    }
                }
                Err(e) => {
                    return error_response(format!("Failed to parse config: {e}"));
                }
            }
        }

        Ok(FeatureType::SystemAlarm) => {
            match serde_json::from_str::<common_config::config::MqttSystemMonitor>(
                params.config.as_str(),
            ) {
                Ok(config) => {
                    cache_manager
                        .update_system_monitor_config(config.clone())
                        .await;
                    if let Err(e) = mqtt_broker::core::dynamic_config::save_cluster_dynamic_config(
                        client_pool,
                        mqtt_broker::core::dynamic_config::ClusterDynamicConfig::MqttSystemMonitor,
                        serde_json::to_vec(&config).unwrap(),
                    )
                    .await
                    {
                        return error_response(format!("Failed to persist config: {e}"));
                    }
                }
                Err(e) => {
                    return error_response(format!("Failed to parse config: {e}"));
                }
            }
        }

        Ok(FeatureType::FlappingDetect) => {
            match serde_json::from_str::<common_config::config::MqttFlappingDetect>(
                params.config.as_str(),
            ) {
                Ok(config) => {
                    cache_manager
                        .update_flapping_detect_config(config.clone())
                        .await;
                    if let Err(e) = mqtt_broker::core::dynamic_config::save_cluster_dynamic_config(
                        client_pool,
                        mqtt_broker::core::dynamic_config::ClusterDynamicConfig::MqttFlappingDetect,
                        serde_json::to_vec(&config).unwrap(),
                    )
                    .await
                    {
                        return error_response(format!("Failed to persist config: {e}"));
                    }
                }
                Err(e) => {
                    return error_response(format!("Failed to parse config: {e}"));
                }
            }
        }

        Ok(FeatureType::MqttProtocol) => {
            match serde_json::from_str::<common_config::config::MqttProtocolConfig>(
                params.config.as_str(),
            ) {
                Ok(config) => {
                    cache_manager
                        .update_mqtt_protocol_config(config.clone())
                        .await;
                    if let Err(e) = mqtt_broker::core::dynamic_config::save_cluster_dynamic_config(
                        client_pool,
                        mqtt_broker::core::dynamic_config::ClusterDynamicConfig::MqttProtocol,
                        serde_json::to_vec(&config).unwrap(),
                    )
                    .await
                    {
                        return error_response(format!("Failed to persist config: {e}"));
                    }
                }
                Err(e) => {
                    return error_response(format!("Failed to parse config: {e}"));
                }
            }
        }

        Ok(FeatureType::MqttSecurity) => {
            match serde_json::from_str::<common_config::config::MqttSecurity>(
                params.config.as_str(),
            ) {
                Ok(config) => {
                    cache_manager.update_security_config(config.clone()).await;
                    if let Err(e) = mqtt_broker::core::dynamic_config::save_cluster_dynamic_config(
                        client_pool,
                        mqtt_broker::core::dynamic_config::ClusterDynamicConfig::MqttSecurity,
                        serde_json::to_vec(&config).unwrap(),
                    )
                    .await
                    {
                        return error_response(format!("Failed to persist config: {e}"));
                    }
                }
                Err(e) => {
                    return error_response(format!("Failed to parse config: {e}"));
                }
            }
        }

        Ok(FeatureType::MqttSchema) => {
            match serde_json::from_str::<common_config::config::MqttSchema>(params.config.as_str())
            {
                Ok(config) => {
                    cache_manager.update_schema_config(config.clone()).await;
                    if let Err(e) = mqtt_broker::core::dynamic_config::save_cluster_dynamic_config(
                        client_pool,
                        mqtt_broker::core::dynamic_config::ClusterDynamicConfig::MqttSchema,
                        serde_json::to_vec(&config).unwrap(),
                    )
                    .await
                    {
                        return error_response(format!("Failed to persist config: {e}"));
                    }
                }
                Err(e) => {
                    return error_response(format!("Failed to parse config: {e}"));
                }
            }
        }

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
        meta: data.clone(),
        nodes: calc_node_num(&state.broker_cache, &data),
    };
    success_response(cluster_info)
}

fn calc_node_num(broker_cache: &Arc<BrokerCacheManager>, meta: &MetaStatus) -> HashSet<String> {
    let mut node_list = HashSet::new();

    for node in broker_cache.node_lists.iter() {
        if let Some(ip) = node.grpc_addr.split(':').next() {
            node_list.insert(ip.to_string());
        }
    }

    for (_, node_info) in meta.membership_config.membership.nodes.iter() {
        if let Some(ip) = node_info.rpc_addr.split(':').next() {
            node_list.insert(ip.to_string());
        }
    }

    node_list
}
