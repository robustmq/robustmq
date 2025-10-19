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
use axum::extract::State;
use broker_core::{cache::BrokerCacheManager, cluster::ClusterStorage};
use common_base::{
    error::common::CommonError,
    http_response::{error_response, success_response},
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use mqtt_broker::{
    common::metrics_cache::MetricsCacheManager, handler::cache::MQTTCacheManager,
    subscribe::manager::SubscribeManager,
};
use network_server::common::connection_manager::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct OverViewResp {
    pub node_list: Vec<BrokerNode>,
    pub cluster_name: String,
    pub message_in_rate: u64,
    pub message_out_rate: u64,
    pub connection_num: u32,
    pub session_num: u32,
    pub topic_num: u32,
    pub placement_status: String,
    pub tcp_connection_num: u32,
    pub tls_connection_num: u32,
    pub websocket_connection_num: u32,
    pub quic_connection_num: u32,
    pub subscribe_num: u32,
    pub exclusive_subscribe_num: u32,
    pub share_subscribe_leader_num: u32,
    pub share_subscribe_resub_num: u32,
    pub exclusive_subscribe_thread_num: u32,
    pub share_subscribe_leader_thread_num: u32,
    pub share_subscribe_follower_thread_num: u32,
}

pub async fn overview(State(state): State<Arc<HttpState>>) -> String {
    match cluster_overview_by_req(
        &state.client_pool,
        &state.mqtt_context.subscribe_manager,
        &state.connection_manager,
        &state.mqtt_context.cache_manager,
        &state.broker_cache,
        &state.mqtt_context.metrics_manager,
    )
    .await
    {
        Ok(data) => success_response(data),
        Err(e) => error_response(e.to_string()),
    }
}

async fn cluster_overview_by_req(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    broker_cache: &Arc<BrokerCacheManager>,
    metrics_manager: &Arc<MetricsCacheManager>,
) -> Result<OverViewResp, CommonError> {
    let config = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let placement_status = cluster_storage.meta_cluster_status().await?;
    let node_list = broker_cache.node_list();

    let reply = OverViewResp {
        cluster_name: config.cluster_name.clone(),
        message_in_rate: metrics_manager.get_message_out_rate(),
        message_out_rate: metrics_manager.get_message_out_rate(),
        connection_num: connection_manager.connections.len() as u32,
        session_num: cache_manager.session_info.len() as u32,
        subscribe_num: subscribe_manager.subscribe_list_len() as u32,
        exclusive_subscribe_num: subscribe_manager.exclusive_push_len() as u32,
        exclusive_subscribe_thread_num: subscribe_manager.exclusive_push_thread_len() as u32,
        share_subscribe_leader_num: subscribe_manager.share_leader_push_len() as u32,
        share_subscribe_leader_thread_num: subscribe_manager.share_leader_push_thread_len() as u32,
        share_subscribe_resub_num: subscribe_manager.share_follower_resub_len() as u32,
        share_subscribe_follower_thread_num: subscribe_manager.share_follower_resub_thread_len()
            as u32,
        topic_num: cache_manager.topic_info.len() as u32,
        node_list,
        placement_status,
        tcp_connection_num: connection_manager.tcp_write_list.len() as u32,
        tls_connection_num: connection_manager.tcp_tls_write_list.len() as u32,
        websocket_connection_num: connection_manager.websocket_write_list.len() as u32,
        quic_connection_num: connection_manager.quic_write_list.len() as u32,
    };
    let _ = subscribe_manager.snapshot_info();

    Ok(reply)
}
