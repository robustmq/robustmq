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

use crate::admin::query::{apply_filters, apply_pagination, apply_sorting, Queryable};
use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::auto_subscribe::AutoSubscribeStorage;
use crate::subscribe::common::{decode_share_group_and_path, get_share_sub_leader, Subscriber};
use crate::subscribe::manager::SubscribeManager;
use common_config::broker::broker_config;
use grpc_clients::mqtt::admin::call::mqtt_broker_subscribe_detail;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use metadata_struct::mqtt::node_extend::MqttNodeExtend;
use metadata_struct::mqtt::subscribe_data::is_mqtt_share_subscribe;
use protocol::broker_mqtt::broker_mqtt_admin::{
    DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest, ListAutoSubscribeRuleReply,
    ListSubscribeReply, ListSubscribeRequest, MqttSubscribeRaw, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SubscribeDetailRaw, SubscribeDetailReply, SubscribeDetailRequest,
};
use protocol::mqtt::common::{qos, retain_forward_rule, Error};
use std::sync::Arc;

pub async fn set_auto_subscribe_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: &SetAutoSubscribeRuleRequest,
) -> Result<SetAutoSubscribeRuleReply, MqttBrokerError> {
    let config = broker_config();

    // Validate and convert QoS
    let _qos = if request.qos <= u8::MAX as u32 {
        qos(request.qos as u8)
    } else {
        return Err(MqttBrokerError::CommonError(
            Error::InvalidRemainingLength(request.qos as usize).to_string(),
        ));
    };

    // Validate and convert RetainHandling
    let _retained_handling = if request.retained_handling <= u8::MAX as u32 {
        retain_forward_rule(request.retained_handling as u8)
    } else {
        return Err(MqttBrokerError::CommonError(
            Error::InvalidRemainingLength(request.retained_handling as usize).to_string(),
        ));
    };

    let auto_subscribe_rule = MqttAutoSubscribeRule {
        cluster: config.cluster_name.clone(),
        topic: request.topic.clone(),
        qos: _qos.ok_or_else(|| {
            MqttBrokerError::CommonError(Error::InvalidQoS(request.qos as u8).to_string())
        })?,
        no_local: request.no_local,
        retain_as_published: request.retain_as_published,
        retained_handling: _retained_handling.ok_or_else(|| {
            MqttBrokerError::CommonError(
                Error::InvalidQoS(request.retained_handling as u8).to_string(),
            )
        })?,
    };

    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    auto_subscribe_storage
        .set_auto_subscribe_rule(auto_subscribe_rule.clone())
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let key = cache_manager.auto_subscribe_rule_key(&config.cluster_name, &request.topic);
    cache_manager
        .auto_subscribe_rule
        .insert(key, auto_subscribe_rule);

    Ok(SetAutoSubscribeRuleReply {})
}

// Delete auto subscribe rule
pub async fn delete_auto_subscribe_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: &DeleteAutoSubscribeRuleRequest,
) -> Result<DeleteAutoSubscribeRuleReply, MqttBrokerError> {
    let config = broker_config();

    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    auto_subscribe_storage
        .delete_auto_subscribe_rule(request.topic.clone())
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let key = cache_manager.auto_subscribe_rule_key(&config.cluster_name, &request.topic);
    cache_manager.auto_subscribe_rule.remove(&key);

    Ok(DeleteAutoSubscribeRuleReply {})
}

// List all auto subscribe rules
pub async fn list_auto_subscribe_rule_by_req(
    cache_manager: &Arc<CacheManager>,
) -> Result<ListAutoSubscribeRuleReply, MqttBrokerError> {
    let auto_subscribe_rules = cache_manager
        .auto_subscribe_rule
        .iter()
        .map(|entry| entry.value().clone().encode())
        .collect();

    Ok(ListAutoSubscribeRuleReply {
        auto_subscribe_rules,
    })
}

pub async fn list_subscribe_by_req(
    subscribe_manager: &Arc<SubscribeManager>,
    request: ListSubscribeRequest,
) -> Result<ListSubscribeReply, MqttBrokerError> {
    let mut subscriptions = Vec::new();
    for (_, raw) in subscribe_manager.subscribe_list.clone() {
        subscriptions.push(MqttSubscribeRaw::from(raw));
    }

    let filtered = apply_filters(subscriptions, &request.options);
    let sorted = apply_sorting(filtered, &request.options);
    let pagination = apply_pagination(sorted, &request.options);

    Ok(ListSubscribeReply {
        subscriptions: pagination.0,
        total_count: pagination.1 as u32,
    })
}

pub async fn subscribe_detail_by_req(
    subscribe_manager: &Arc<SubscribeManager>,
    client_pool: &Arc<ClientPool>,
    request: SubscribeDetailRequest,
) -> Result<SubscribeDetailReply, MqttBrokerError> {
    let subscribe = if let Some(subscribe) =
        subscribe_manager.get_subscribe(&request.client_id, &request.path)
    {
        subscribe
    } else {
        return Ok(SubscribeDetailReply::default());
    };

    if !is_mqtt_share_subscribe(&subscribe.path) {
        let mut details = vec![];
        for (key, value) in subscribe_manager.exclusive_push.clone() {
            if value.client_id == request.client_id && value.sub_path == request.path {
                let thread_data =
                    if let Some(thread) = subscribe_manager.exclusive_push_thread.get(&key) {
                        serde_json::to_string(&thread.clone())?
                    } else {
                        "{}".to_string()
                    };
                let sub_data = serde_json::to_string(&value)?;

                details.push(SubscribeDetailRaw {
                    sub: sub_data,
                    thread: thread_data,
                });
            }
        }

        return Ok(SubscribeDetailReply {
            sub_info: serde_json::to_string(&subscribe)?,
            details,
        });
    }

    let conf = broker_config();
    let (group, _) = decode_share_group_and_path(&subscribe.path);
    let reply = get_share_sub_leader(client_pool, &group).await?;

    if conf.broker_id == reply.broker_id {
        let mut raw_details = vec![];
        for (key, value) in subscribe_manager.share_leader_push.clone() {
            if value.path == request.path {
                let data: Vec<Subscriber> = value
                    .sub_list
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect();
                let thread_data =
                    if let Some(thread) = subscribe_manager.share_leader_push_thread.get(&key) {
                        serde_json::to_string(&thread.clone())?
                    } else {
                        "{}".to_string()
                    };
                let sub_data = serde_json::to_string(&data)?;
                raw_details.push(SubscribeDetailRaw {
                    sub: sub_data,
                    thread: thread_data,
                });
            }
        }
        return Ok(SubscribeDetailReply {
            sub_info: serde_json::to_string(&subscribe)?,
            details: raw_details,
        });
    }

    let extend_info: MqttNodeExtend = serde_json::from_str::<MqttNodeExtend>(&reply.extend_info)?;
    let req = SubscribeDetailRequest {
        client_id: request.client_id.clone(),
        path: request.path.clone(),
    };
    let reply = mqtt_broker_subscribe_detail(client_pool, &[extend_info.grpc_addr], req).await?;
    Ok(reply)
}

impl Queryable for MqttSubscribeRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            "path" => Some(self.path.clone()),
            _ => None,
        }
    }
}
