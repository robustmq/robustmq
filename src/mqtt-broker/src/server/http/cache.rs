// Copyright 2023 RobustMQ Team
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

use std::collections::HashMap;

use super::server::HttpServerState;
use crate::{
    handler::cache::{ClientPkidData, ConnectionLiveTime},
    subscribe::{
        subscribe_manager::{ShareLeaderSubscribeData, ShareSubShareSub},
        subscriber::{SubscribeData, Subscriber},
    },
};
use axum::extract::State;
use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    http_response::success_response,
    metrics::dump_metrics,
};
use dashmap::DashMap;
use metadata_struct::mqtt::{
    cluster::MQTTClusterDynamicConfig, session::MQTTSession, topic::MQTTTopic, user::MQTTUser,
};
use serde::{Deserialize, Serialize};

pub async fn metrics() -> String {
    return dump_metrics();
}

pub async fn cache_info(State(state): State<HttpServerState>) -> String {
    let connection_info = DashMap::with_capacity(8);
    for raw in state.cache_metadata.connection_info.clone() {
        let data = format!("{:?}", raw.1);
        connection_info.insert(raw.0, data);
    }
    let result = MetadataCacheResult {
        config: broker_mqtt_conf().clone(),

        cluster_name: state.cache_metadata.cluster_name.clone(),
        cluster_info: state.cache_metadata.cluster_info.clone(),
        user_info: state.cache_metadata.user_info.clone(),
        session_info: state.cache_metadata.session_info.clone(),
        connection_info: connection_info,
        topic_info: state.cache_metadata.topic_info.clone(),
        topic_id_name: state.cache_metadata.topic_id_name.clone(),
        subscribe_filter: state.cache_metadata.subscribe_filter.clone(),
        publish_pkid_info: state.cache_metadata.publish_pkid_info.clone(),

        heartbeat_data: state.cache_metadata.heartbeat_data.clone(),

        exclusive_subscribe: state.subscribe_cache.exclusive_subscribe.clone(),
        share_leader_subscribe: state.subscribe_cache.share_leader_subscribe.clone(),
        share_follower_subscribe: state.subscribe_cache.share_follower_subscribe.clone(),
        share_follower_identifier_id: state.subscribe_cache.share_follower_identifier_id.clone(),

        exclusive_push_thread: state.subscribe_cache.exclusive_push_thread_keys(),
        share_leader_push_thread: state.subscribe_cache.share_leader_push_thread_keys(),
        share_follower_resub_thread: state.subscribe_cache.share_follower_resub_thread_keys(),
        client_pkid_data: state.cache_metadata.client_pkid_data.clone(),
    };

    return success_response(result);
}

pub async fn index() -> String {
    let mut cluster: HashMap<String, String> = HashMap::new();
    cluster.insert("version".to_string(), "1.0.0".to_string());
    return success_response(cluster);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MetadataCacheResult {
    // config
    pub config: BrokerMQTTConfig,

    // metadata_cache
    pub cluster_name: String,
    pub cluster_info: DashMap<String, MQTTClusterDynamicConfig>,
    pub user_info: DashMap<String, MQTTUser>,
    pub session_info: DashMap<String, MQTTSession>,
    pub connection_info: DashMap<u64, String>,
    pub topic_info: DashMap<String, MQTTTopic>,
    pub topic_id_name: DashMap<String, String>,
    pub subscribe_filter: DashMap<String, DashMap<String, SubscribeData>>,
    pub publish_pkid_info: DashMap<String, Vec<u16>>,

    // heartbeat data
    pub heartbeat_data: DashMap<String, ConnectionLiveTime>,

    // subscribe data
    pub exclusive_subscribe: DashMap<String, Subscriber>,
    pub share_leader_subscribe: DashMap<String, ShareLeaderSubscribeData>,
    pub share_follower_subscribe: DashMap<String, ShareSubShareSub>,
    pub share_follower_identifier_id: DashMap<usize, String>,

    pub exclusive_push_thread: Vec<String>,
    pub share_leader_push_thread: Vec<String>,
    pub share_follower_resub_thread: Vec<String>,

    // QosMemory
    pub client_pkid_data: DashMap<String, ClientPkidData>,
}
