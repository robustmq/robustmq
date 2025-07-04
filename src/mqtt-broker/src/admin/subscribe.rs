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

use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::auto_subscribe::AutoSubscribeStorage;

use crate::subscribe::manager::SubscribeManager;
use common_base::utils::time_util::timestamp_to_local_datetime;
use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use protocol::broker_mqtt::broker_mqtt_admin::{
    DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest, ListAutoSubscribeRuleReply,
    ListSubscribeReply, MqttSubscribeRaw, SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest,
    SubscribeDetailReply,
};
use protocol::mqtt::common::{qos, retain_forward_rule, Error};
use std::sync::Arc;

pub async fn set_auto_subscribe_rule(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: &SetAutoSubscribeRuleRequest,
) -> Result<SetAutoSubscribeRuleReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

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
pub async fn delete_auto_subscribe_rule(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: &DeleteAutoSubscribeRuleRequest,
) -> Result<DeleteAutoSubscribeRuleReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

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

pub async fn list_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
) -> Result<ListSubscribeReply, MqttBrokerError> {
    let mut results = Vec::new();
    for (_, raw) in subscribe_manager.subscribe_list.clone() {
        results.push(MqttSubscribeRaw {
            broker_id: raw.broker_id,
            client_id: raw.client_id,
            create_time: timestamp_to_local_datetime(raw.create_time as i64),
            no_local: if raw.filter.nolocal { 1 } else { 0 },
            path: raw.path,
            pk_id: raw.pkid as u32,
            preserve_retain: if raw.filter.preserve_retain { 1 } else { 0 },
            properties: serde_json::to_string(&raw.subscribe_properties)?,
            protocol: format!("{:?}", raw.protocol),
            qos: format!("{:?}", raw.filter.qos),
            retain_handling: format!("{:?}", raw.filter.retain_handling),
        });
    }

    Ok(ListSubscribeReply {
        subscriptions: results,
    })
}

pub async fn subscribe_detail(
    _subscribe_manager: &Arc<SubscribeManager>,
) -> Result<SubscribeDetailReply, MqttBrokerError> {
    Ok(SubscribeDetailReply::default())
}
