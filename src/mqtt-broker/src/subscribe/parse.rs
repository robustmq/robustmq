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
    common::types::ResultMqttBrokerError,
    handler::{
        cache::MQTTCacheManager,
        sub_exclusive::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub},
        sub_share::{decode_share_info, is_mqtt_share_subscribe},
    },
    subscribe::{
        common::{is_match_sub_and_topic, Subscriber},
        directly_push::directly_group_name,
        manager::SubscribeManager,
    },
};
use common_base::tools::now_second;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::MQTTTopic;
use protocol::mqtt::common::{Filter, MqttProtocol, SubscribeProperties};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct ParseSubscribeContext {
    pub client_pool: Arc<ClientPool>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub client_id: String,
    pub topic: MQTTTopic,
    pub protocol: MqttProtocol,
    pub pkid: u16,
    pub filter: Filter,
    pub subscribe_properties: Option<SubscribeProperties>,
    pub rewrite_sub_path: Option<String>,
}

#[derive(Clone)]
struct AddDirectlyPushContext {
    pub subscribe_manager: Arc<SubscribeManager>,
    pub topic: MQTTTopic,
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub sub_identifier: Option<usize>,
    pub filter: Filter,
    pub rewrite_sub_path: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct AddSharePushContext {
    pub topic_name: String,
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub sub_identifier: Option<usize>,
    pub filter: Filter,
    pub sub_path: String,
    pub group_name: String,
    pub pkid: u16,
}

pub async fn parse_subscribe_by_new_topic(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    topic: &MQTTTopic,
) -> ResultMqttBrokerError {
    let conf = broker_config();

    for row in subscribe_manager.subscribe_list.iter() {
        let subscribe = row.value();
        if subscribe.broker_id != conf.broker_id {
            continue;
        }
        let rewrite_sub_path = cache_manager.get_new_rewrite_name(&subscribe.path);

        parse_subscribe_by_new_subscribe(ParseSubscribeContext {
            client_pool: client_pool.clone(),
            subscribe_manager: subscribe_manager.clone(),
            client_id: subscribe.client_id.clone(),
            topic: topic.clone(),
            protocol: subscribe.protocol.clone(),
            pkid: subscribe.pkid,
            filter: subscribe.filter.clone(),
            subscribe_properties: subscribe.subscribe_properties.clone(),
            rewrite_sub_path: rewrite_sub_path.clone(),
        })
        .await?;
    }
    Ok(())
}

pub async fn parse_subscribe_by_new_subscribe(
    context: ParseSubscribeContext,
) -> ResultMqttBrokerError {
    let sub_identifier = if let Some(properties) = context.subscribe_properties.clone() {
        properties.subscription_identifier
    } else {
        None
    };

    // share sub
    if is_mqtt_share_subscribe(&context.filter.path) {
        add_share_push(
            &context.subscribe_manager,
            &mut AddSharePushContext {
                topic_name: context.topic.topic_name.to_owned(),
                client_id: context.client_id.to_owned(),
                protocol: context.protocol.clone(),
                sub_identifier,
                filter: context.filter.clone(),
                pkid: context.pkid,
                sub_path: "".to_string(),
                group_name: "".to_string(),
            },
        )
        .await
    } else {
        add_directly_push(AddDirectlyPushContext {
            subscribe_manager: context.subscribe_manager.clone(),
            topic: context.topic.clone(),
            client_id: context.client_id.clone(),
            protocol: context.protocol.clone(),
            sub_identifier,
            filter: context.filter.clone(),
            rewrite_sub_path: context.rewrite_sub_path.clone(),
        })
        .await
    }
}

async fn add_share_push(
    _subscribe_manager: &Arc<SubscribeManager>,
    req: &mut AddSharePushContext,
) -> ResultMqttBrokerError {
    let (group_name, sub_name) = decode_share_info(&req.filter.path);
    req.group_name = format!("{group_name}_{sub_name}");
    req.sub_path = sub_name;

    if is_match_sub_and_topic(&req.sub_path, &req.topic_name).is_ok() {
        let _sub = Subscriber {
            protocol: req.protocol.clone(),
            client_id: req.client_id.clone(),
            topic_name: req.topic_name.clone(),
            group_name: req.group_name.clone(),
            qos: req.filter.qos,
            no_local: req.filter.nolocal,
            preserve_retain: req.filter.preserve_retain,
            retain_forward_rule: req.filter.retain_handling.clone(),
            subscription_identifier: req.sub_identifier,
            sub_path: req.filter.path.clone(),
            rewrite_sub_path: None,
            create_time: now_second(),
        };

        // subscribe_manager.directly_sub_manager.add_sub(sub);
    }
    Ok(())
}

async fn add_directly_push(context: AddDirectlyPushContext) -> ResultMqttBrokerError {
    let path = if is_exclusive_sub(&context.filter.path) {
        decode_exclusive_sub_path_to_topic_name(&context.filter.path).to_owned()
    } else {
        context.filter.path.to_owned()
    };

    let new_path = if let Some(sub_path) = context.rewrite_sub_path.clone() {
        sub_path
    } else {
        path.clone()
    };

    if is_match_sub_and_topic(&new_path, &context.topic.topic_name).is_ok() {
        let sub = Subscriber {
            protocol: context.protocol.to_owned(),
            client_id: context.client_id.to_owned(),
            topic_name: context.topic.topic_name.to_owned(),
            group_name: directly_group_name(
                &context.client_id,
                &context.filter.path,
                &context.topic.topic_name,
            ),
            qos: context.filter.qos,
            no_local: context.filter.nolocal,
            preserve_retain: context.filter.preserve_retain,
            retain_forward_rule: context.filter.retain_handling.to_owned(),
            subscription_identifier: context.sub_identifier.to_owned(),
            sub_path: context.filter.path.clone(),
            rewrite_sub_path: context.rewrite_sub_path.clone(),
            create_time: now_second(),
        };

        context
            .subscribe_manager
            .add_directly_sub(&context.topic.topic_name, &sub)
            .await;
    }
    Ok(())
}
