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

use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    tools::now_second,
    utils::topic_util::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub},
};
use grpc_clients::{placement::mqtt::call::placement_set_subscribe, pool::ClientPool};
use metadata_struct::mqtt::{subscribe_data::MqttSubscribe, topic::MqttTopic};
use protocol::{
    mqtt::common::{Filter, MqttProtocol, Subscribe, SubscribeProperties},
    placement_center::placement_center_mqtt::SetSubscribeRequest,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::subscribe::{
    common::Subscriber,
    common::{
        decode_share_info, get_share_sub_leader, is_match_sub_and_topic, is_queue_sub, is_share_sub,
    },
    manager::{ShareSubShareSub, SubscribeManager},
};

use super::{
    cache::CacheManager, error::MqttBrokerError, topic_rewrite::convert_sub_path_by_rewrite_rule,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct ParseShareQueueSubscribeRequest {
    pub topic_name: String,
    pub topic_id: String,
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub sub_identifier: Option<usize>,
    pub filter: Filter,
    pub sub_name: String,
    pub group_name: String,
    pub pkid: u16,
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
    let filters = &subscribe.filters;
    for filter in filters {
        let subscribe_data = MqttSubscribe {
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
            subscribe: subscribe_data.encode(),
        };

        if let Err(e) = placement_set_subscribe(client_pool, &conf.placement_center, request).await
        {
            error!(
                "Failed to set subscribe to placement center, error message: {}",
                e
            );
            return Err(MqttBrokerError::CommonError(e.to_string()));
        }
        // add subscribe by cache
        subscribe_manager.add_subscribe(subscribe_data);
    }
    // parse subscribe
    let new_client_pool = client_pool.to_owned();
    let new_subscribe_manager = subscribe_manager.clone();
    let new_protocol = protocol.to_owned();
    let new_client_id = client_id.to_owned();
    let new_subscribe = subscribe.to_owned();
    let new_subscribe_properties = subscribe_properties.to_owned();
    let new_filters = filters.to_owned();
    let new_cache_manager = cache_manager.to_owned();
    tokio::spawn(async move {
        for filter in new_filters.clone() {
            let rewrite_sub_path =
                match convert_sub_path_by_rewrite_rule(&new_cache_manager, &filter.path) {
                    Ok(rewrite_sub_path) => rewrite_sub_path,
                    Err(e) => {
                        error!(
                            "Failed to convert sub path by rewrite rule, error message: {}",
                            e
                        );
                        continue;
                    }
                };

            for (_, topic) in new_cache_manager.topic_info.clone() {
                if let Err(e) = parse_subscribe(
                    &new_client_pool,
                    &new_subscribe_manager,
                    &new_client_id,
                    &topic,
                    &new_protocol,
                    new_subscribe.packet_identifier,
                    &filter,
                    &new_subscribe_properties,
                    &rewrite_sub_path,
                )
                .await
                {
                    error!("Failed to parse subscribe, error message: {}", e);
                }
            }
        }
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn parse_subscribe(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
    topic: &MqttTopic,
    protocol: &MqttProtocol,
    pkid: u16,
    filter: &Filter,
    subscribe_properties: &Option<SubscribeProperties>,
    rewrite_sub_path: &Option<String>,
) -> Result<(), MqttBrokerError> {
    let sub_identifier = if let Some(properties) = subscribe_properties.clone() {
        properties.subscription_identifier
    } else {
        None
    };

    // share sub
    if is_share_sub(&filter.path) || is_queue_sub(&filter.path) {
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
        .await
    } else {
        add_exclusive_push(
            subscribe_manager,
            topic,
            client_id,
            protocol,
            &sub_identifier,
            filter,
            rewrite_sub_path,
        )
    }
}

async fn parse_share_subscribe(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &mut ParseShareQueueSubscribeRequest,
) -> Result<(), MqttBrokerError> {
    let (group_name, sub_name) = decode_share_info(&req.filter.path);

    req.group_name = if is_queue_sub(&req.filter.path) {
        format!("$queue_{}", sub_name)
    } else {
        format!("{}_{}", group_name, sub_name)
    };

    req.sub_name = sub_name;

    let conf = broker_mqtt_conf();
    if is_match_sub_and_topic(&req.sub_name, &req.topic_name).is_ok() {
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
    Ok(())
}

pub async fn add_share_push_leader(
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
        retain_forward_rule: req.filter.retain_handling.clone(),
        subscription_identifier: req.sub_identifier,
        sub_path: req.filter.path.clone(),
        rewrite_sub_path: None,
        create_time: now_second(),
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
        topic_id: req.topic_id.clone(),
        topic_name: req.topic_name.clone(),
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
    rewrite_sub_path: &Option<String>,
) -> Result<(), MqttBrokerError> {
    let path = if is_exclusive_sub(&filter.path) {
        decode_exclusive_sub_path_to_topic_name(&filter.path).to_owned()
    } else {
        filter.path.to_owned()
    };

    let new_path = if let Some(sub_path) = rewrite_sub_path.clone() {
        sub_path
    } else {
        path
    };

    if is_match_sub_and_topic(&new_path, &topic.topic_name).is_ok() {
        let sub = Subscriber {
            protocol: protocol.to_owned(),
            client_id: client_id.to_owned(),
            topic_name: topic.topic_name.to_owned(),
            group_name: None,
            topic_id: topic.topic_id.to_owned(),
            qos: filter.qos,
            nolocal: filter.nolocal,
            preserve_retain: filter.preserve_retain,
            retain_forward_rule: filter.retain_handling.to_owned(),
            subscription_identifier: sub_identifier.to_owned(),
            sub_path: filter.path.clone(),
            rewrite_sub_path: rewrite_sub_path.clone(),
            create_time: now_second(),
        };
        subscribe_manager.add_topic_subscribe(&topic.topic_name, client_id, &filter.path);
        subscribe_manager.add_exclusive_push(client_id, &filter.path, &topic.topic_id, sub);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::add_exclusive_push;
    use crate::subscribe::manager::SubscribeManager;
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::topic::MqttTopic;
    use protocol::mqtt::common::{Filter, MqttProtocol};
    use std::sync::Arc;

    #[test]
    fn add_exclusive_push_test() {
        let ex_path = "$exclusive/topic/1/2";
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let topic = MqttTopic {
            topic_name: "/topic/1/2".to_string(),
            topic_id: "test-id".to_string(),
            ..Default::default()
        };
        let client_id = unique_id();
        let filter = Filter {
            path: ex_path.to_owned(),
            ..Default::default()
        };
        let res = add_exclusive_push(
            &subscribe_manager,
            &topic,
            &client_id,
            &MqttProtocol::Mqtt3,
            &Some(1),
            &filter,
            &None,
        );
        assert!(res.is_ok());
        println!("{:?}", subscribe_manager.topic_subscribe_list);
        println!("{:?}", subscribe_manager.exclusive_push);
        assert_eq!(subscribe_manager.topic_subscribe_list.len(), 1);
        assert_eq!(subscribe_manager.exclusive_push.len(), 1);

        for (_, sub) in subscribe_manager.exclusive_push.clone() {
            assert_eq!(sub.client_id, client_id);
            assert_eq!(sub.sub_path, ex_path.to_owned());
            assert_eq!(sub.topic_name, topic.topic_name.to_owned());
        }

        for (to, sub) in subscribe_manager.topic_subscribe_list.clone() {
            for raw in sub {
                assert_eq!(raw.client_id, client_id);
                assert_eq!(raw.path, ex_path.to_owned());
                assert_eq!(to, topic.topic_name.to_owned());
            }
        }
    }
}
