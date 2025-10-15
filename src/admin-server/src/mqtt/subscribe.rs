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
    request::mqtt::{
        AutoSubscribeListReq, CreateAutoSubscribeReq, DeleteAutoSubscribeReq, SubscribeDetailReq,
        SubscribeListReq,
    },
    response::{
        mqtt::{AutoSubscribeListRow, SlowSubscribeListRow, SubscribeListRow},
        PageReplyData,
    },
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::{
    http_response::{error_response, success_response},
    utils::time_util::timestamp_to_local_datetime,
};
use metadata_struct::mqtt::{
    auto_subscribe_rule::MqttAutoSubscribeRule, subscribe_data::is_mqtt_share_subscribe,
};
use mqtt_broker::storage::{auto_subscribe::AutoSubscribeStorage, local::LocalStorage};
use protocol::mqtt::common::{qos, retain_forward_rule};
use std::sync::Arc;

pub async fn subscribe_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SubscribeListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let mut subscribes = Vec::new();
    for (_, sub) in state.mqtt_context.subscribe_manager.list_subscribe() {
        subscribes.push(SubscribeListRow {
            broker_id: sub.broker_id,
            client_id: sub.client_id,
            create_time: timestamp_to_local_datetime(sub.create_time as i64),
            no_local: if sub.filter.nolocal { 1 } else { 0 },
            path: sub.path.clone(),
            pk_id: sub.pkid as u32,
            preserve_retain: if sub.filter.preserve_retain { 1 } else { 0 },
            properties: serde_json::to_string(&sub.subscribe_properties).unwrap(),
            protocol: format!("{:?}", sub.protocol),
            qos: format!("{:?}", sub.filter.qos),
            retain_handling: format!("{:?}", sub.filter.retain_handling),
            is_share_sub: is_mqtt_share_subscribe(&sub.path),
        });
    }
    let filtered = apply_filters(subscribes, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SubscribeListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            _ => None,
        }
    }
}

pub async fn subscribe_detail(
    State(_state): State<Arc<HttpState>>,
    Json(_params): Json<SubscribeDetailReq>,
) -> String {
    success_response("")
}

pub async fn auto_subscribe_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<AutoSubscribeListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );
    let mut subscriptions = Vec::new();
    for (_, raw) in state.mqtt_context.cache_manager.auto_subscribe_rule.clone() {
        subscriptions.push(AutoSubscribeListRow {
            topic: raw.topic.clone(),
            qos: format!("{:?}", raw.topic),
            no_local: raw.no_local,
            retain_as_published: raw.retain_as_published,
            retained_handling: format!("{:?}", raw.retained_handling),
        });
    }

    let filtered = apply_filters(subscriptions, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for AutoSubscribeListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "topic" => Some(self.topic.clone()),
            _ => None,
        }
    }
}

pub async fn auto_subscribe_create(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<CreateAutoSubscribeReq>,
) -> String {
    let qos_new = if let Some(qos) = qos(params.qos as u8) {
        qos
    } else {
        return error_response("Inconsistent QoS format".to_string());
    };

    let handing = if let Some(handing) = retain_forward_rule(params.retained_handling as u8) {
        handing
    } else {
        return error_response("Inconsistent RetainHandling format".to_string());
    };

    let auto_subscribe_rule = MqttAutoSubscribeRule {
        cluster: state.broker_cache.cluster_name.clone(),
        topic: params.topic.clone(),
        qos: qos_new,
        no_local: params.no_local,
        retain_as_published: params.retain_as_published,
        retained_handling: handing,
    };

    let auto_subscribe_storage = AutoSubscribeStorage::new(state.client_pool.clone());
    if let Err(e) = auto_subscribe_storage
        .set_auto_subscribe_rule(auto_subscribe_rule.clone())
        .await
    {
        return error_response(e.to_string());
    }

    state
        .mqtt_context
        .cache_manager
        .add_auto_subscribe_rule(auto_subscribe_rule);

    success_response("success")
}

pub async fn auto_subscribe_delete(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<DeleteAutoSubscribeReq>,
) -> String {
    let auto_subscribe_storage = AutoSubscribeStorage::new(state.client_pool.clone());
    if let Err(e) = auto_subscribe_storage
        .delete_auto_subscribe_rule(params.topic_name.clone())
        .await
    {
        return error_response(e.to_string());
    }

    state
        .mqtt_context
        .cache_manager
        .delete_auto_subscribe_rule(&state.broker_cache.cluster_name, &params.topic_name);

    success_response("success")
}

pub async fn slow_subscribe_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<AutoSubscribeListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );
    let mut list_slow_subscribes = Vec::new();

    let local_storage = LocalStorage::new(state.rocksdb_engine_handler.clone());
    let data_list = match local_storage.list_slow_sub_log().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    for slow_data in data_list {
        list_slow_subscribes.push(SlowSubscribeListRow {
            client_id: slow_data.client_id.clone(),
            topic_name: slow_data.topic_name.clone(),
            time_span: slow_data.time_span,
            node_info: slow_data.node_info.clone(),
            create_time: timestamp_to_local_datetime(slow_data.create_time as i64),
            subscribe_name: slow_data.subscribe_name.clone(),
        });
    }

    let filtered = apply_filters(list_slow_subscribes, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SlowSubscribeListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            "topic_name" => Some(self.topic_name.clone()),
            _ => None,
        }
    }
}
