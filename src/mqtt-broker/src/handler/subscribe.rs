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

use std::sync::Arc;

use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::{placement::mqtt::call::placement_set_subscribe, pool::ClientPool};
use log::error;
use metadata_struct::mqtt::{
    cluster::AvailableFlag, subscribe_data::MqttSubscribe, topic::MqttTopic,
};
use protocol::{
    mqtt::common::{Filter, MqttProtocol, Subscribe, SubscribeProperties},
    placement_center::placement_center_mqtt::SetSubscribeRequest,
};
use serde::{Deserialize, Serialize};

use crate::subscribe::{
    sub_common::{
        decode_queue_info, decode_share_info, get_share_sub_leader, is_queue_sub, is_share_sub,
        path_regex_match,
    },
    subscribe_manager::{ShareSubShareSub, SubscribeManager},
    subscriber::Subscriber,
};

use super::{cache::CacheManager, error::MqttBrokerError, sub_exclusive::add_exclusive_subscribe};

#[derive(Clone, Deserialize, Serialize)]
struct ParseShareQueueSubscribeRequest {
    topic_name: String,
    topic_id: String,
    client_id: String,
    protocol: MqttProtocol,
    sub_identifier: Option<usize>,
    filter: Filter,
    sub_name: String,
    group_name: String,
    pkid: u16,
}

pub async fn save_subscribe(
    client_id: &str,
    protocol: &MqttProtocol,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    subscribe: &Subscribe,
    subscribe_properties: &Option<SubscribeProperties>,
) -> Result<(), MqttBrokerError> {
    let conf = broker_mqtt_conf();

    for filter in subscribe.filters.clone() {
        let sucscribe_data = MqttSubscribe {
            client_id: client_id.to_owned(),
            path: filter.path.clone(),
            cluster_name: conf.cluster_name.to_owned(),
            broker_id: conf.broker_id,
            filter: filter.clone(),
            pkid: subscribe.packet_identifier,
            subscribe_properties: subscribe_properties.to_owned(),
            protocol: protocol.to_owned(),
        };

        // save subscribe
        let request = SetSubscribeRequest {
            cluster_name: conf.cluster_name.to_owned(),
            client_id: client_id.to_owned(),
            path: filter.path.clone(),
            subscribe: sucscribe_data.encode(),
        };
        placement_set_subscribe(client_pool, &conf.placement_center, request).await?;

        // add susribe by cache
        subscribe_manager.add_subscribe(sucscribe_data.clone());
    }

    // parse subscribe
    for (_, topic) in cache_manager.topic_info.clone() {
        for filter in subscribe.filters.clone() {
            parse_subscribe(
                client_pool,
                cache_manager,
                subscribe_manager,
                client_id,
                &topic,
                protocol,
                subscribe.packet_identifier,
                &filter,
                subscribe_properties,
            )
            .await;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn parse_subscribe(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
    topic: &MqttTopic,
    protocol: &MqttProtocol,
    pkid: u16,
    filter: &Filter,
    subscribe_properties: &Option<SubscribeProperties>,
) {
    let sub_identifier = if let Some(properties) = subscribe_properties.clone() {
        properties.subscription_identifier
    } else {
        None
    };

    let enable_exclusive_sub = metadata_cache
        .get_cluster_info()
        .feature
        .exclusive_subscription_available
        == AvailableFlag::Disable;

    if enable_exclusive_sub {
        add_exclusive_subscribe(subscribe_manager, &filter.path, client_id);
    }

    if is_share_sub(&filter.path) {
        parse_share_subscribe(
            client_pool,
            subscribe_manager,
            &mut ParseShareQueueSubscribeRequest {
                topic_name: topic.topic_name.to_owned(),
                topic_id: topic.topic_id.to_owned(),
                client_id: client_id.to_owned(),
                protocol: protocol.clone(),
                sub_identifier,
                filter: filter.clone(),
                pkid,
                sub_name: "".to_string(),
                group_name: "".to_string(),
            },
        )
        .await;
    } else if is_queue_sub(&filter.path) {
        parse_queue_subscribe(
            client_pool,
            subscribe_manager,
            &mut ParseShareQueueSubscribeRequest {
                topic_name: topic.topic_name.to_owned(),
                topic_id: topic.topic_id.to_owned(),
                client_id: client_id.to_owned(),
                protocol: protocol.clone(),
                pkid,
                sub_identifier,
                filter: filter.clone(),
                sub_name: "".to_string(),
                group_name: "".to_string(),
            },
        )
        .await;
    } else {
        add_exclusive_push(
            subscribe_manager,
            topic,
            client_id,
            protocol,
            &sub_identifier,
            filter,
        );
    }
}

async fn parse_share_subscribe(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &mut ParseShareQueueSubscribeRequest,
) {
    let (group_name, sub_name) = decode_share_info(&req.filter.path);
    req.group_name = format!("{}_{}", group_name, sub_name);
    req.sub_name = sub_name;
    parse_share_queue_subscribe_common(client_pool, subscribe_manager, req).await;
}

async fn parse_queue_subscribe(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &mut ParseShareQueueSubscribeRequest,
) {
    let sub_name = decode_queue_info(&req.filter.path);
    // queueSub is a special shareSub
    let group_name = format!("$queue_{}", sub_name);
    req.group_name = group_name;
    req.sub_name = sub_name;
    parse_share_queue_subscribe_common(client_pool, subscribe_manager, req).await;
}

async fn parse_share_queue_subscribe_common(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &ParseShareQueueSubscribeRequest,
) {
    let conf = broker_mqtt_conf();
    if path_regex_match(&req.topic_name, &req.sub_name) {
        match get_share_sub_leader(client_pool, &req.group_name).await {
            Ok(reply) => {
                if reply.broker_id == conf.broker_id {
                    add_share_push_leader(subscribe_manager, req).await;
                } else {
                    add_share_push_follower(subscribe_manager, req).await;
                }
            }
            Err(e) => {
                error!(
                    "Failed to get Leader for shared subscription, error message: {}",
                    e
                );
            }
        }
    }
}

async fn add_share_push_leader(
    subscribe_manager: &Arc<SubscribeManager>,
    req: &ParseShareQueueSubscribeRequest,
) {
    let sub = Subscriber {
        protocol: req.protocol.clone(),
        client_id: req.client_id.clone(),
        topic_name: req.topic_name.clone(),
        group_name: Some(req.group_name.clone()),
        topic_id: req.topic_id.clone(),
        qos: req.filter.qos,
        nolocal: req.filter.nolocal,
        preserve_retain: req.filter.preserve_retain,
        retain_forward_rule: req.filter.retain_forward_rule.clone(),
        subscription_identifier: req.sub_identifier,
        sub_path: req.filter.path.clone(),
    };

    subscribe_manager.add_topic_subscribe(&req.topic_name, &req.client_id, &req.filter.path);
    subscribe_manager.add_share_subscribe_leader(&req.sub_name, sub);
}

async fn add_share_push_follower(
    subscribe_manager: &Arc<SubscribeManager>,
    req: &ParseShareQueueSubscribeRequest,
) {
    let share_sub = ShareSubShareSub {
        client_id: req.client_id.clone(),
        protocol: req.protocol.clone(),
        packet_identifier: req.pkid,
        filter: req.filter.clone(),
        group_name: req.group_name.clone(),
        sub_name: req.sub_name.clone(),
        subscription_identifier: req.sub_identifier,
    };

    subscribe_manager.add_share_subscribe_follower(
        &req.client_id,
        &req.group_name,
        &req.topic_id,
        share_sub,
    );
}

fn add_exclusive_push(
    subscribe_manager: &Arc<SubscribeManager>,
    topic: &MqttTopic,
    client_id: &str,
    protocol: &MqttProtocol,
    sub_identifier: &Option<usize>,
    filter: &Filter,
) {
    if path_regex_match(&topic.topic_name, &filter.path) {
        let sub = Subscriber {
            protocol: protocol.to_owned(),
            client_id: client_id.to_owned(),
            topic_name: topic.topic_name.to_owned(),
            group_name: None,
            topic_id: topic.topic_id.to_owned(),
            qos: filter.qos,
            nolocal: filter.nolocal,
            preserve_retain: filter.preserve_retain,
            retain_forward_rule: filter.retain_forward_rule.to_owned(),
            subscription_identifier: sub_identifier.to_owned(),
            sub_path: filter.path.to_owned(),
        };
        subscribe_manager.add_topic_subscribe(&topic.topic_name, client_id, &filter.path);
        subscribe_manager.add_exclusive_push(client_id, &filter.path, &topic.topic_id, sub);
    }
}
