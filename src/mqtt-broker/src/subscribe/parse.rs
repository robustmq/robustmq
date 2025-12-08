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
    handler::{
        cache::MQTTCacheManager,
        error::MqttBrokerError,
        sub_exclusive::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub},
        sub_share::{
            decode_share_info, full_group_name, is_mqtt_share_subscribe, is_share_sub_leader,
        },
        tool::ResultMqttBrokerError,
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
use metadata_struct::mqtt::{subscribe_data::MqttSubscribe, topic::MQTTTopic};
use protocol::{
    broker::broker_mqtt_inner::{
        MqttBrokerUpdateCacheActionType, MqttBrokerUpdateCacheResourceType,
    },
    mqtt::common::{Filter, MqttProtocol, SubscribeProperties},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    select,
    sync::{broadcast, mpsc::Receiver},
};
use tracing::{debug, error, info};

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
    pub pkid: u16,
}

#[derive(Clone)]
pub struct ParseSubscribeData {
    pub action_type: MqttBrokerUpdateCacheActionType,
    pub resource_type: MqttBrokerUpdateCacheResourceType,
    pub subscribe: Option<MqttSubscribe>,
    pub topic: Option<MQTTTopic>,
}

pub async fn start_update_parse_thread(
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    mut rx: Receiver<ParseSubscribeData>,
    stop_sx: broadcast::Sender<bool>,
) {
    info!("Subscribe parse thread started");
    let mut stop_recv = stop_sx.subscribe();

    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        info!("Subscribe parse thread stopping");
                        break;
                    }
                }
            }

            result = rx.recv() => {
                let Some(data) = result else { continue };

                match (data.resource_type, data.action_type) {
                    (MqttBrokerUpdateCacheResourceType::Topic, MqttBrokerUpdateCacheActionType::Set) => {
                        if let Some(topic) = data.topic {
                            if let Err(e) = parse_subscribe_by_new_topic(&client_pool, &cache_manager, &subscribe_manager, &topic).await {
                                error!("Failed to parse subscriptions for new topic '{}': {}", topic.topic_name, e);
                            }
                        }
                    }
                    (MqttBrokerUpdateCacheResourceType::Topic, MqttBrokerUpdateCacheActionType::Delete) => {
                        if let Some(topic) = data.topic {
                            info!("Removing subscriptions for deleted topic '{}'", topic.topic_name);
                            subscribe_manager.remove_by_topic(&topic.topic_name);
                        }
                    }
                    (MqttBrokerUpdateCacheResourceType::Subscribe, MqttBrokerUpdateCacheActionType::Set) => {
                        if let Some(subscribe) = data.subscribe {
                            if let Err(e) = parse_subscribe_by_new_subscribe(&subscribe_manager, &cache_manager, &client_pool, &subscribe).await {
                                error!("Failed to parse new subscription for client '{}', path '{}': {}",
                                    subscribe.client_id, subscribe.path, e);
                            }
                        }
                    }
                    (MqttBrokerUpdateCacheResourceType::Subscribe, MqttBrokerUpdateCacheActionType::Delete) => {
                        if let Some(subscribe) = data.subscribe {
                            info!("Removing subscription: client='{}', path='{}'", subscribe.client_id, subscribe.path);
                            subscribe_manager.remove_by_sub(&subscribe.client_id, &subscribe.path);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub async fn parse_subscribe_by_new_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    subscribe: &MqttSubscribe,
) -> ResultMqttBrokerError {
    subscribe_manager.add_subscribe(subscribe);
    let rewrite_sub_path = cache_manager.get_new_rewrite_name(&subscribe.filter.path);
    let topic_count = cache_manager.topic_info.len();

    debug!(
        "Matching new subscription: client='{}', path='{}' against {} topics",
        subscribe.client_id, subscribe.filter.path, topic_count
    );

    for row in cache_manager.topic_info.iter() {
        let topic = row.value();
        parse_subscribe(ParseSubscribeContext {
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

/// Parses and matches all existing subscriptions when a new topic is created.
/// This will iterate through all subscriptions to find matches.
pub async fn parse_subscribe_by_new_topic(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    topic: &MQTTTopic,
) -> ResultMqttBrokerError {
    let broker_id = broker_config().broker_id;
    let sub_count = subscribe_manager.subscribe_list.len();

    debug!(
        "Matching new topic '{}' against {} subscriptions",
        topic.topic_name, sub_count
    );

    for row in subscribe_manager.subscribe_list.iter() {
        let subscribe = row.value();
        if subscribe.broker_id != broker_id {
            continue;
        }
        let rewrite_sub_path = cache_manager.get_new_rewrite_name(&subscribe.path);

        parse_subscribe(ParseSubscribeContext {
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

/// Creates a Subscriber instance with common fields.
#[allow(clippy::too_many_arguments)]
fn create_subscriber(
    protocol: MqttProtocol,
    client_id: String,
    topic_name: String,
    group_name: String,
    filter: &Filter,
    sub_identifier: Option<usize>,
    sub_path: String,
    rewrite_sub_path: Option<String>,
) -> Subscriber {
    Subscriber {
        protocol,
        client_id,
        topic_name,
        group_name,
        qos: filter.qos,
        no_local: filter.nolocal,
        preserve_retain: filter.preserve_retain,
        retain_forward_rule: filter.retain_handling.clone(),
        subscription_identifier: sub_identifier,
        sub_path,
        rewrite_sub_path,
        create_time: now_second(),
    }
}

/// Matches a subscription with a topic and adds it to the appropriate push manager.
/// Used both when a new subscription is created and when a new topic is created.
async fn parse_subscribe(context: ParseSubscribeContext) -> ResultMqttBrokerError {
    let sub_identifier = if let Some(properties) = context.subscribe_properties.clone() {
        properties.subscription_identifier
    } else {
        None
    };

    if is_mqtt_share_subscribe(&context.filter.path) {
        add_share_push(
            &context.subscribe_manager,
            &context.client_pool,
            &AddSharePushContext {
                topic_name: context.topic.topic_name.to_owned(),
                client_id: context.client_id.to_owned(),
                protocol: context.protocol.clone(),
                sub_identifier,
                filter: context.filter.clone(),
                pkid: context.pkid,
            },
        )
        .await?;
    } else {
        add_directly_push(AddDirectlyPushContext {
            subscribe_manager: context.subscribe_manager.clone(),
            topic: context.topic.clone(),
            client_id: context.client_id.clone(),
            protocol: context.protocol.clone(),
            sub_identifier,
            filter: context.filter.clone(),
            rewrite_sub_path: context.rewrite_sub_path.clone(),
        })?;
    }
    Ok(())
}

async fn add_share_push(
    subscribe_manager: &Arc<SubscribeManager>,
    client_pool: &Arc<ClientPool>,
    req: &AddSharePushContext,
) -> ResultMqttBrokerError {
    let (group_name, sub_name) = decode_share_info(&req.filter.path);
    let group_name_full = full_group_name(&group_name, &sub_name);

    if is_match_sub_and_topic(&sub_name, &req.topic_name).is_ok() {
        debug!(
            "Adding share subscription: client='{}', group='{}', topic='{}'",
            req.client_id, group_name_full, req.topic_name
        );

        // If the current node is not the Leader, then there is no need to process it and no error needs to be reported.
        if !is_share_sub_leader(client_pool, &group_name_full).await? {
            return Ok(());
        }

        let sub = create_subscriber(
            req.protocol.clone(),
            req.client_id.clone(),
            req.topic_name.clone(),
            group_name_full,
            &req.filter,
            req.sub_identifier,
            req.filter.path.clone(),
            None,
        );

        subscribe_manager.add_share_sub(&req.topic_name, &sub);
    }
    Ok(())
}

fn add_directly_push(context: AddDirectlyPushContext) -> ResultMqttBrokerError {
    let path = if is_exclusive_sub(&context.filter.path) {
        decode_exclusive_sub_path_to_topic_name(&context.filter.path)
    } else {
        &context.filter.path
    };

    let new_path = context.rewrite_sub_path.as_deref().unwrap_or(path);

    if is_match_sub_and_topic(new_path, &context.topic.topic_name).is_ok() {
        debug!(
            "Adding direct subscription: client='{}', path='{}', topic='{}'",
            context.client_id, context.filter.path, context.topic.topic_name
        );

        let group_name = directly_group_name(
            &context.client_id,
            &context.filter.path,
            &context.topic.topic_name,
        );

        let sub = create_subscriber(
            context.protocol,
            context.client_id,
            context.topic.topic_name.clone(),
            group_name,
            &context.filter,
            context.sub_identifier,
            context.filter.path.clone(),
            context.rewrite_sub_path,
        );

        context
            .subscribe_manager
            .add_directly_sub(&context.topic.topic_name, &sub);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::{QoS, RetainHandling};

    fn create_test_filter(path: &str) -> Filter {
        Filter {
            path: path.to_string(),
            qos: QoS::AtMostOnce,
            nolocal: false,
            preserve_retain: true,
            retain_handling: RetainHandling::OnEverySubscribe,
        }
    }

    #[test]
    fn test_create_subscriber() {
        let filter = create_test_filter("test/topic");
        let sub = create_subscriber(
            MqttProtocol::Mqtt5,
            "client1".to_string(),
            "test/topic".to_string(),
            "group1".to_string(),
            &filter,
            Some(123),
            "test/topic".to_string(),
            None,
        );

        assert_eq!(sub.client_id, "client1");
        assert_eq!(sub.topic_name, "test/topic");
        assert_eq!(sub.group_name, "group1");
        assert_eq!(sub.qos, QoS::AtMostOnce);
        assert_eq!(sub.subscription_identifier, Some(123));
        assert!(sub.preserve_retain);
    }

    #[test]
    fn test_add_directly_push() {
        let manager = Arc::new(SubscribeManager::new());
        let filter = create_test_filter("topic");
        let topic = MQTTTopic {
            topic_name: "topic".to_string(),
            create_time: now_second(),
        };
        let context = AddDirectlyPushContext {
            subscribe_manager: manager.clone(),
            topic,
            client_id: "client1".to_string(),
            protocol: MqttProtocol::Mqtt5,
            sub_identifier: None,
            filter,
            rewrite_sub_path: None,
        };

        // Should not panic
        add_directly_push(context);
        assert!(manager.topic_subscribes.get("topic").is_some());
    }

    #[test]
    fn test_add_directly_push_with_wildcard() {
        let manager = Arc::new(SubscribeManager::new());
        let filter = create_test_filter("test/#");
        let topic = MQTTTopic {
            topic_name: "test/topic".to_string(),
            create_time: now_second(),
        };
        let context = AddDirectlyPushContext {
            subscribe_manager: manager.clone(),
            topic,
            client_id: "client1".to_string(),
            protocol: MqttProtocol::Mqtt5,
            sub_identifier: None,
            filter,
            rewrite_sub_path: None,
        };

        // Should not panic and should match wildcard
        add_directly_push(context);
        assert!(manager.topic_subscribes.get("test/topic").is_some());
    }
}
