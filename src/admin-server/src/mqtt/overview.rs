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

use crate::{
    request::mqtt::OverviewMetricsReq,
    response::mqtt::{OverViewMetricsResp, OverViewResp},
    state::HttpState,
};
use axum::{extract::State, Json};
use broker_core::{cache::BrokerCacheManager, cluster::ClusterStorage};
use common_base::{
    error::common::CommonError,
    http_response::{error_response, success_response},
    tools::now_second,
};
use common_config::broker::broker_config;
use common_metrics::mqtt::publish::{
    record_mqtt_messages_received_get, record_mqtt_messages_sent_get,
};
use grpc_clients::pool::ClientPool;
use mqtt_broker::{
    common::metrics_cache::MetricsCacheManager, handler::cache::MQTTCacheManager,
    subscribe::manager::SubscribeManager,
};
use network_server::common::connection_manager::ConnectionManager;
use std::sync::Arc;

pub async fn overview(State(state): State<Arc<HttpState>>) -> String {
    match cluster_overview_by_req(
        &state.client_pool,
        &state.mqtt_context.subscribe_manager,
        &state.connection_manager,
        &state.mqtt_context.cache_manager,
        &state.broker_cache,
    )
    .await
    {
        Ok(data) => success_response(data),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn overview_metrics(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<OverviewMetricsReq>,
) -> String {
    match cluster_overview_metrics_by_req(&state.mqtt_context.metrics_manager, &params).await {
        Ok(data) => success_response(data),
        Err(e) => error_response(e.to_string()),
    }
}

async fn cluster_overview_metrics_by_req(
    metrics_cache_manager: &Arc<MetricsCacheManager>,
    request: &OverviewMetricsReq,
) -> Result<OverViewMetricsResp, CommonError> {
    let start_time = if request.start_time == 0 {
        now_second() - 3600
    } else {
        request.start_time
    };
    let end_time = if request.end_time == 0 {
        now_second()
    } else {
        request.end_time
    };
    let reply = OverViewMetricsResp {
        connection_num: metrics_cache_manager.get_connection_num_by_time(start_time, end_time),
        topic_num: metrics_cache_manager.get_topic_num_by_time(start_time, end_time),
        subscribe_num: metrics_cache_manager.get_subscribe_num_by_time(start_time, end_time),
        message_in_num: metrics_cache_manager.get_message_in_num_by_time(start_time, end_time),
        message_out_num: metrics_cache_manager.get_message_out_num_by_time(start_time, end_time),
        message_drop_num: metrics_cache_manager.get_message_drop_num_by_time(start_time, end_time),
    };

    Ok(reply)
}

async fn cluster_overview_by_req(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    broker_cache: &Arc<BrokerCacheManager>,
) -> Result<OverViewResp, CommonError> {
    let config = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let placement_status = cluster_storage.place_cluster_status().await?;
    let node_list = broker_cache.node_list();

    let reply = OverViewResp {
        cluster_name: config.cluster_name.clone(),
        message_in_rate: record_mqtt_messages_received_get(),
        message_out_rate: record_mqtt_messages_sent_get(),
        connection_num: connection_manager.connections.len() as u32,
        session_num: cache_manager.session_info.len() as u32,
        subscribe_num: subscribe_manager.subscribe_list.len() as u32,
        exclusive_subscribe_num: subscribe_manager.exclusive_push.len() as u32,
        exclusive_subscribe_thread_num: subscribe_manager.exclusive_push_thread.len() as u32,
        share_subscribe_leader_num: subscribe_manager.share_leader_push.len() as u32,
        share_subscribe_leader_thread_num: subscribe_manager.share_leader_push_thread.len() as u32,
        share_subscribe_resub_num: subscribe_manager.share_follower_resub.len() as u32,
        share_subscribe_follower_thread_num: subscribe_manager.share_follower_resub_thread.len()
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
