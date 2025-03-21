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
use crate::storage::auto_subscribe::AutoSubscribeStorage;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use protocol::broker_mqtt::broker_mqtt_admin::{
    DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest, ListAutoSubscribeRuleReply,
    SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest,
};
use protocol::mqtt::common::{qos, retain_forward_rule, Error, QoS, RetainForwardRule};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn set_auto_subscribe_rule(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<SetAutoSubscribeRuleRequest>,
) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();

    let mut _qos: Option<QoS> = None;
    if req.qos <= u8::MAX as u32 {
        _qos = qos(req.qos as u8);
    } else {
        return Err(Status::cancelled(
            Error::InvalidRemainingLength(req.qos as usize).to_string(),
        ));
    };
    let mut _retained_handling: Option<RetainForwardRule> = None;
    if req.retained_handling <= u8::MAX as u32 {
        _retained_handling = retain_forward_rule(req.retained_handling as u8);
    } else {
        return Err(Status::cancelled(
            Error::InvalidRemainingLength(req.retained_handling as usize).to_string(),
        ));
    };

    let auto_subscribe_rule = MqttAutoSubscribeRule {
        cluster: config.cluster_name.clone(),
        topic: req.topic.clone(),
        qos: _qos.ok_or_else(|| Status::cancelled(Error::InvalidQoS(req.qos as u8).to_string()))?,
        no_local: req.no_local,
        retain_as_published: req.retain_as_published,
        retained_handling: _retained_handling.ok_or_else(|| {
            Status::cancelled(Error::InvalidQoS(req.retained_handling as u8).to_string())
        })?,
    };
    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    match auto_subscribe_storage
        .set_auto_subscribe_rule(auto_subscribe_rule.clone())
        .await
    {
        Ok(_) => {
            let key = cache_manager.auto_subscribe_rule_key(&config.cluster_name, &req.topic);
            cache_manager
                .auto_subscribe_rule
                .insert(key, auto_subscribe_rule);
            Ok(Response::new(SetAutoSubscribeRuleReply::default()))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn delete_auto_subscribe_rule(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<DeleteAutoSubscribeRuleRequest>,
) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
    let req = request.into_inner();
    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    match auto_subscribe_storage
        .delete_auto_subscribe_rule(req.topic.clone())
        .await
    {
        Ok(_) => {
            let config = broker_mqtt_conf();
            let key = cache_manager.auto_subscribe_rule_key(&config.cluster_name, &req.topic);
            cache_manager.auto_subscribe_rule.remove(&key);
            Ok(Response::new(DeleteAutoSubscribeRuleReply::default()))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub fn list_auto_subscribe_rule_by_req(
    cache_manager: &Arc<CacheManager>,
) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
    Ok(Response::new(ListAutoSubscribeRuleReply {
        auto_subscribe_rules: cache_manager
            .auto_subscribe_rule
            .iter()
            .map(|entry| entry.value().clone().encode())
            .collect(),
    }))
}
