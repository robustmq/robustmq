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
    core::{
        cache::MQTTCacheManager,
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
use metadata_struct::mqtt::{subscribe::MqttSubscribe, topic::Topic};
use protocol::{
    broker::broker::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType},
    mqtt::common::{Filter, MqttProtocol},
};
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
    pub subscribe: MqttSubscribe,
    pub topic: Topic,
    pub rewrite_sub_path: Option<String>,
}

#[derive(Clone)]
struct AddDirectlyPushContext {
    pub subscribe_manager: Arc<SubscribeManager>,
    pub tenant: String,
    pub topic: String,
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub sub_identifier: Option<usize>,
    pub filter: Filter,
    pub rewrite_sub_path: Option<String>,
}

#[derive(Clone)]
struct AddSharePushContext {
    pub client_pool: Arc<ClientPool>,
    pub tenant: String,
    pub topic_name: String,
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub sub_identifier: Option<usize>,
    pub filter: Filter,
    pub rewrite_sub_path: Option<String>,
}

#[derive(Clone)]
pub struct ParseSubscribeData {
    pub action_type: BrokerUpdateCacheActionType,
    pub resource_type: BrokerUpdateCacheResourceType,
    pub subscribe: Option<MqttSubscribe>,
    pub topic: Option<Topic>,
}

pub async fn start_update_parse_thread(
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    mut rx: Receiver<ParseSubscribeData>,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut stop_recv = stop_sx.subscribe();

    loop {
        select! {
            val = stop_recv.recv() => {
                match val {
                    Ok(true) => {
                        info!("Subscribe parse thread stopping");
                        break;
                    }
                    Ok(false) => {}
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Subscribe parse thread stop channel closed, exiting.");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        debug!(
                            "Subscribe parse thread stop channel lagged, skipped {} messages.",
                            skipped
                        );
                    }
                }
            }

            result = rx.recv() => {
                let Some(data) = result else {
                    info!("Subscribe parse thread request channel closed, exiting.");
                    break;
                };

                match (data.resource_type, data.action_type) {
                    (BrokerUpdateCacheResourceType::Topic, BrokerUpdateCacheActionType::Create) => {
                        if let Some(topic) = data.topic {
                            if let Err(e) = parse_subscribe_by_new_topic(&client_pool, &cache_manager, &subscribe_manager, &topic).await {
                                error!("Failed to parse subscriptions for new topic '{}': {}", topic.topic_name, e);
                            }
                        }
                    }
                    (BrokerUpdateCacheResourceType::Topic, BrokerUpdateCacheActionType::Delete) => {
                        if let Some(topic) = data.topic {
                            info!("Removing subscriptions for deleted topic '{}'", topic.topic_name);
                            subscribe_manager.remove_by_topic(&topic.tenant, &topic.topic_name);
                        }
                    }
                    (BrokerUpdateCacheResourceType::Subscribe, BrokerUpdateCacheActionType::Create) => {
                        if let Some(subscribe) = data.subscribe {
                            if let Err(e) = parse_subscribe_by_new_subscribe(&subscribe_manager, &cache_manager, &client_pool, &subscribe).await {
                                error!("Failed to parse new subscription for client '{}', path '{}': {}",
                                    subscribe.client_id, subscribe.path, e);
                            }
                        }
                    }
                    (BrokerUpdateCacheResourceType::Subscribe, BrokerUpdateCacheActionType::Delete) => {
                        if let Some(subscribe) = data.subscribe {
                            info!("Removing subscription: client='{}', path='{}'", subscribe.client_id, subscribe.path);
                            subscribe_manager.remove_by_sub(&subscribe.tenant, &subscribe.client_id, &subscribe.path);
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
    debug!(
        "Matching new subscription: client='{}', path='{}' under tenant '{}'",
        subscribe.client_id, subscribe.filter.path, subscribe.tenant
    );

    let rewrite_sub_path =
        cache_manager.get_new_rewrite_name(&subscribe.tenant, &subscribe.filter.path);

    // Collect topics first to release DashMap shard locks before any .await
    let topics: Vec<_> = cache_manager
        .node_cache
        .list_topics_by_tenant(&subscribe.tenant);

    for topic in topics {
        parse_subscribe(
            cache_manager,
            ParseSubscribeContext {
                client_pool: client_pool.clone(),
                subscribe_manager: subscribe_manager.clone(),
                subscribe: subscribe.clone(),
                topic,
                rewrite_sub_path: rewrite_sub_path.clone(),
            },
        )
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
    topic: &Topic,
) -> ResultMqttBrokerError {
    let broker_id = broker_config().broker_id;

    // Collect subscriptions first to release DashMap shard locks before any .await
    let subscribes: Vec<_> = subscribe_manager
        .subscribe_list
        .get(&topic.tenant)
        .map(|tenant_subs| {
            debug!(
                "Matching new topic '{}' against {} subscriptions under tenant '{}'",
                topic.topic_name,
                tenant_subs.len(),
                topic.tenant
            );
            tenant_subs
                .value()
                .iter()
                .filter(|row| row.value().broker_id == broker_id)
                .map(|row| row.value().clone())
                .collect()
        })
        .unwrap_or_default();

    let rewrite_sub_path = cache_manager.get_new_rewrite_name(&topic.tenant, &topic.topic_name);
    for subscribe in subscribes {
        parse_subscribe(
            cache_manager,
            ParseSubscribeContext {
                client_pool: client_pool.clone(),
                subscribe_manager: subscribe_manager.clone(),
                subscribe,
                topic: topic.clone(),
                rewrite_sub_path: rewrite_sub_path.clone(),
            },
        )
        .await?;
    }
    Ok(())
}

/// Creates a Subscriber instance with common fields.
#[allow(clippy::too_many_arguments)]
fn create_subscriber(
    protocol: MqttProtocol,
    client_id: String,
    tenant: String,
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
        tenant,
        topic_name,
        group_name,
        qos: filter.qos,
        no_local: filter.no_local,
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
async fn parse_subscribe(
    cache_manager: &Arc<MQTTCacheManager>,
    context: ParseSubscribeContext,
) -> ResultMqttBrokerError {
    let sub = &context.subscribe;
    let sub_identifier = sub
        .subscribe_properties
        .as_ref()
        .and_then(|p| p.subscription_identifier);

    let new_topic_name = cache_manager
        .get_new_rewrite_name(&sub.tenant, &context.topic.topic_name)
        .unwrap_or_else(|| context.topic.topic_name.clone());

    if is_mqtt_share_subscribe(&sub.filter.path) {
        add_share_push(
            cache_manager,
            &context.subscribe_manager,
            &AddSharePushContext {
                client_pool: context.client_pool.clone(),
                tenant: sub.tenant.clone(),
                topic_name: new_topic_name,
                client_id: sub.client_id.clone(),
                protocol: sub.protocol.clone(),
                sub_identifier,
                filter: sub.filter.clone(),
                rewrite_sub_path: context.rewrite_sub_path.clone(),
            },
        )
        .await?;
    } else {
        add_directly_push(AddDirectlyPushContext {
            subscribe_manager: context.subscribe_manager.clone(),
            tenant: sub.tenant.clone(),
            topic: new_topic_name,
            client_id: sub.client_id.clone(),
            protocol: sub.protocol.clone(),
            sub_identifier,
            filter: sub.filter.clone(),
            rewrite_sub_path: context.rewrite_sub_path.clone(),
        })?;
    }
    Ok(())
}

async fn add_share_push(
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &AddSharePushContext,
) -> ResultMqttBrokerError {
    let (group_name, sub_name) = decode_share_info(&req.filter.path);
    let group_name_full = full_group_name(&group_name, &sub_name);

    // Use the rewritten sub path for topic matching when a rewrite rule applies.
    let match_path = req.rewrite_sub_path.as_deref().unwrap_or(&sub_name);

    if is_match_sub_and_topic(match_path, &req.topic_name).is_ok() {
        debug!(
            "Adding share subscription: client='{}', group='{}', topic='{}'",
            req.client_id, group_name_full, req.topic_name
        );

        if !is_share_sub_leader(
            cache_manager,
            &req.client_pool,
            &req.tenant,
            &group_name_full,
        )
        .await?
        {
            return Ok(());
        }

        let sub = create_subscriber(
            req.protocol.clone(),
            req.client_id.clone(),
            req.tenant.clone(),
            req.topic_name.clone(),
            group_name_full,
            &req.filter,
            req.sub_identifier,
            req.filter.path.clone(),
            req.rewrite_sub_path.clone(),
        );

        subscribe_manager.add_share_sub(&sub);
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

    if is_match_sub_and_topic(new_path, &context.topic).is_ok() {
        debug!(
            "Adding direct subscription: client='{}', path='{}', topic='{}'",
            context.client_id, context.filter.path, context.topic
        );

        let group_name =
            directly_group_name(&context.client_id, &context.filter.path, &context.topic);

        let sub = create_subscriber(
            context.protocol,
            context.client_id,
            context.tenant.clone(),
            context.topic.clone(),
            group_name,
            &context.filter,
            context.sub_identifier,
            context.filter.path.clone(),
            context.rewrite_sub_path,
        );

        context.subscribe_manager.add_directly_sub(&sub);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use protocol::mqtt::common::{QoS, RetainHandling};

    fn create_test_filter(path: &str) -> Filter {
        Filter {
            path: path.to_string(),
            qos: QoS::AtMostOnce,
            no_local: false,
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
            DEFAULT_TENANT.to_string(),
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
        let topic = "topic".to_string();
        let context = AddDirectlyPushContext {
            subscribe_manager: manager.clone(),
            tenant: DEFAULT_TENANT.to_string(),
            topic,
            client_id: "client1".to_string(),
            protocol: MqttProtocol::Mqtt5,
            sub_identifier: None,
            filter,
            rewrite_sub_path: None,
        };

        // Should not panic
        add_directly_push(context).unwrap();
        assert!(manager
            .topic_subscribes
            .get(DEFAULT_TENANT)
            .map(|t| t.contains_key("topic"))
            .unwrap_or(false));
    }

    #[test]
    fn test_add_directly_push_with_wildcard() {
        let manager = Arc::new(SubscribeManager::new());
        let filter = create_test_filter("test/#");
        let topic = "test/topic".to_string();
        let context = AddDirectlyPushContext {
            subscribe_manager: manager.clone(),
            tenant: DEFAULT_TENANT.to_string(),
            topic,
            client_id: "client1".to_string(),
            protocol: MqttProtocol::Mqtt5,
            sub_identifier: None,
            filter,
            rewrite_sub_path: None,
        };

        // Should not panic and should match wildcard
        add_directly_push(context).unwrap();
        assert!(manager
            .topic_subscribes
            .get(DEFAULT_TENANT)
            .map(|t| t.contains_key("test/topic"))
            .unwrap_or(false));
    }
}
