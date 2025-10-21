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
    extractor::ValidatedJson,
    state::HttpState,
    tool::{
        query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::{extract::State, Json};
use mqtt_broker::subscribe::{common::Subscriber, manager::ShareLeaderSubscribeData};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeListReq {
    pub client_id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SubscribeDetailReq {
    pub client_id: String,
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct ShareSubscribeDetailReq {
    pub client_id: String,
    pub group_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeDetailRep {
    pub share_sub: bool,
    pub group_leader_info: Option<SubGroupLeaderRaw>,
    pub topic_list: Vec<SubTopicRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubTopicRaw {
    pub client_id: String,
    pub path: String,
    pub topic_name: String,
    pub exclusive_push_data: Option<Subscriber>,
    pub share_push_data: Option<ShareLeaderSubscribeData>,
    pub push_thread: Option<SubPushThreadDataRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubGroupLeaderRaw {
    pub broker_id: u64,
    pub broker_addr: String,
    pub extend_info: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubPushThreadDataRaw {
    pub push_success_record_num: u64,
    pub push_error_record_num: u64,
    pub last_push_time: u64,
    pub last_run_time: u64,
    pub create_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubPushThreadRaw {}

#[derive(Serialize, Deserialize, Debug)]
pub struct AutoSubscribeListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateAutoSubscribeReq {
    #[validate(length(min = 1, max = 256, message = "Topic length must be between 1-256"))]
    pub topic: String,

    #[validate(range(max = 2, message = "QoS must be 0, 1 or 2"))]
    pub qos: u32,

    pub no_local: bool,
    pub retain_as_published: bool,

    #[validate(range(max = 2, message = "Retained handling must be 0, 1 or 2"))]
    pub retained_handling: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteAutoSubscribeReq {
    #[validate(length(
        min = 1,
        max = 256,
        message = "Topic name length must be between 1-256"
    ))]
    pub topic_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubscribeListRow {
    pub client_id: String,
    pub path: String,
    pub broker_id: u64,
    pub protocol: String,
    pub qos: String,
    pub no_local: u32,
    pub preserve_retain: u32,
    pub retain_handling: String,
    pub create_time: String,
    pub pk_id: u32,
    pub properties: String,
    pub is_share_sub: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AutoSubscribeListRow {
    pub topic: String,
    pub qos: String,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub retained_handling: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SlowSubscribeListRow {
    pub client_id: String,
    pub topic_name: String,
    pub time_span: u64,
    pub node_info: String,
    pub create_time: String,
    pub subscribe_name: String,
}

use common_base::{
    error::common::CommonError,
    http_response::{error_response, success_response},
    utils::time_util::timestamp_to_local_datetime,
};
use metadata_struct::mqtt::{
    auto_subscribe_rule::MqttAutoSubscribeRule, subscribe_data::is_mqtt_share_subscribe,
};
use mqtt_broker::{
    handler::error::MqttBrokerError,
    storage::{auto_subscribe::AutoSubscribeStorage, local::LocalStorage},
    subscribe::common::{decode_share_group_and_path, get_share_sub_leader, is_share_sub_leader},
};
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
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SubscribeDetailReq>,
) -> String {
    if is_mqtt_share_subscribe(&params.path) {
        let (group, _) = decode_share_group_and_path(&params.path);
        let leader = match is_share_sub_leader(&state.client_pool, &group).await {
            Ok(data) => data,
            Err(e) => {
                return error_response(e.to_string());
            }
        };

        // Forward the request to the leader of the group
        if leader {
            //
            return "".to_string();
        }

        // current node is leader
        let (topic_list, group_leader_info) = match share_sub_detail(&state, &params).await {
            Ok(data) => data,
            Err(e) => {
                return error_response(e.to_string());
            }
        };
        return success_response(SubscribeDetailRep {
            share_sub: true,
            topic_list,
            group_leader_info: Some(group_leader_info),
        });
    }

    let topic_list = match exclusive_sub_detail(&state, &params).await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };
    success_response(SubscribeDetailRep {
        share_sub: false,
        topic_list,
        group_leader_info: None,
    })
}

pub async fn share_subscribe_detail(
    State(_state): State<Arc<HttpState>>,
    Json(_params): Json<ShareSubscribeDetailReq>,
) -> String {
    success_response("".to_string())
}

async fn exclusive_sub_detail(
    state: &Arc<HttpState>,
    params: &SubscribeDetailReq,
) -> Result<Vec<SubTopicRaw>, MqttBrokerError> {
    let mut topic_list: Vec<SubTopicRaw> = Vec::new();
    for topic_name in state
        .mqtt_context
        .subscribe_manager
        .get_subscribe_topics_by_client_id_path(&params.client_id, &params.path)
    {
        let key = state.mqtt_context.subscribe_manager.exclusive_key(
            &params.client_id,
            &params.path,
            &topic_name,
        );

        let push_data = state
            .mqtt_context
            .subscribe_manager
            .get_exclusive_push(&key);

        let push_thread = if let Some(data) = state
            .mqtt_context
            .subscribe_manager
            .get_exclusive_push_thread(&key)
        {
            Some(SubPushThreadDataRaw {
                push_error_record_num: data.push_error_record_num,
                push_success_record_num: data.push_success_record_num,
                last_push_time: data.last_push_time,
                last_run_time: data.last_run_time,
                create_time: data.create_time,
            })
        } else {
            None
        };

        topic_list.push(SubTopicRaw {
            client_id: params.client_id.clone(),
            path: params.path.clone(),
            topic_name,
            exclusive_push_data: push_data,
            share_push_data: None,
            push_thread,
        });
    }

    Ok(topic_list)
}

async fn share_sub_detail(
    state: &Arc<HttpState>,
    params: &SubscribeDetailReq,
) -> Result<(Vec<SubTopicRaw>, SubGroupLeaderRaw), CommonError> {
    let (group, sub_name) = decode_share_group_and_path(&params.path);
    let mut topic_list = Vec::new();

    // topic list
    for topic_name in state
        .mqtt_context
        .subscribe_manager
        .get_subscribe_topics_by_client_id_path(&params.client_id, &params.path)
    {
        let key =
            state
                .mqtt_context
                .subscribe_manager
                .share_leader_key(&group, &sub_name, &topic_name);

        let push_thread = if let Some(data) = state
            .mqtt_context
            .subscribe_manager
            .get_share_leader_push_thread(&key)
        {
            Some(SubPushThreadDataRaw {
                push_error_record_num: data.push_error_record_num,
                push_success_record_num: data.push_success_record_num,
                last_push_time: data.last_push_time,
                last_run_time: data.last_run_time,
                create_time: data.create_time,
            })
        } else {
            None
        };

        let leader_push_data = state
            .mqtt_context
            .subscribe_manager
            .get_share_leader_push(&key);

        topic_list.push(SubTopicRaw {
            client_id: params.client_id.clone(),
            path: params.path.clone(),
            topic_name,
            exclusive_push_data: None,
            share_push_data: leader_push_data,
            push_thread,
        });
    }

    // group info
    let (group, _) = decode_share_group_and_path(&params.path);
    let reply = get_share_sub_leader(&state.client_pool, &group).await?;
    let group_leader_info: SubGroupLeaderRaw = SubGroupLeaderRaw {
        broker_addr: reply.broker_addr.clone(),
        broker_id: reply.broker_id,
        extend_info: reply.extend_info.clone(),
    };

    Ok((topic_list, group_leader_info))
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
    ValidatedJson(params): ValidatedJson<CreateAutoSubscribeReq>,
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
    ValidatedJson(params): ValidatedJson<DeleteAutoSubscribeReq>,
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
        .delete_auto_subscribe_rule( &params.topic_name);

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
