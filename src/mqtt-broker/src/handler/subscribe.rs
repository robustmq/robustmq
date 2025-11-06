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
    tools::now_second,
    utils::topic_util::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub},
};
use common_config::broker::broker_config;
use grpc_clients::{meta::mqtt::call::placement_set_subscribe, pool::ClientPool};
use metadata_struct::mqtt::{
    subscribe_data::{is_mqtt_share_subscribe, MqttSubscribe},
    topic::MQTTTopic,
};
use protocol::{
    meta::meta_service_mqtt::SetSubscribeRequest,
    mqtt::common::{Filter, MqttProtocol, Subscribe, SubscribeProperties},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::subscribe::{
    common::{
        decode_share_group_and_path, is_match_sub_and_topic, is_share_sub_leader, Subscriber,
    },
    manager::{ShareSubShareSub, SubscribeManager},
};

use super::{cache::MQTTCacheManager, error::MqttBrokerError};
use crate::common::types::ResultMqttBrokerError;

#[derive(Clone)]
pub struct SaveSubscribeContext {
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub client_pool: Arc<ClientPool>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
}

#[derive(Clone)]
pub struct AddExclusivePushContext {
    pub subscribe_manager: Arc<SubscribeManager>,
    pub topic: MQTTTopic,
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub sub_identifier: Option<usize>,
    pub filter: Filter,
    pub rewrite_sub_path: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ParseShareQueueSubscribeRequest {
    pub topic_name: String,
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub sub_identifier: Option<usize>,
    pub filter: Filter,
    pub sub_path: String,
    pub group_name: String,
    pub pkid: u16,
}

pub async fn save_subscribe(context: SaveSubscribeContext) -> ResultMqttBrokerError {
    let conf = broker_config();
    let filters = &context.subscribe.filters;
    for filter in filters {
        let subscribe_data = MqttSubscribe {
            client_id: context.client_id.to_owned(),
            path: filter.path.clone(),
            cluster_name: conf.cluster_name.to_owned(),
            broker_id: conf.broker_id,
            filter: filter.clone(),
            pkid: context.subscribe.packet_identifier,
            subscribe_properties: context.subscribe_properties.to_owned(),
            protocol: context.protocol.to_owned(),
            create_time: now_second(),
        };

        // save subscribe
        let request = SetSubscribeRequest {
            cluster_name: conf.cluster_name.to_owned(),
            client_id: context.client_id.to_owned(),
            path: filter.path.clone(),
            subscribe: subscribe_data.encode()?,
        };

        if let Err(e) =
            placement_set_subscribe(&context.client_pool, &conf.get_meta_service_addr(), request)
                .await
        {
            error!(
                "Failed to set subscribe to meta service, error message: {}",
                e
            );
            return Err(MqttBrokerError::CommonError(e.to_string()));
        }
        // add subscribe by cache
        context
            .subscribe_manager
            .add_subscribe(subscribe_data.clone());
    }
    // parse subscribe
    let new_client_pool = context.client_pool.to_owned();
    let new_subscribe_manager = context.subscribe_manager.clone();
    let new_protocol = context.protocol.to_owned();
    let new_client_id = context.client_id.to_owned();
    let new_subscribe = context.subscribe.clone();
    let new_subscribe_properties = context.subscribe_properties.to_owned();
    let new_filters = context.subscribe.filters.to_owned();
    let new_cache_manager = context.cache_manager.to_owned();
    tokio::spawn(async move {
        for filter in new_filters.clone() {
            let rewrite_sub_path = new_cache_manager.get_new_rewrite_name(&filter.path);
            for (_, topic) in new_cache_manager.topic_info.clone() {
                if let Err(e) = parse_subscribe(ParseSubscribeContext {
                    client_pool: new_client_pool.clone(),
                    subscribe_manager: new_subscribe_manager.clone(),
                    client_id: new_client_id.clone(),
                    topic: topic.clone(),
                    protocol: new_protocol.clone(),
                    pkid: new_subscribe.packet_identifier,
                    filter: filter.clone(),
                    subscribe_properties: new_subscribe_properties.clone(),
                    rewrite_sub_path: rewrite_sub_path.clone(),
                })
                .await
                {
                    error!("Failed to parse subscribe, error message: {}", e);
                }
            }
        }
    });
    Ok(())
}

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

pub async fn parse_subscribe(context: ParseSubscribeContext) -> ResultMqttBrokerError {
    let sub_identifier = if let Some(properties) = context.subscribe_properties.clone() {
        properties.subscription_identifier
    } else {
        None
    };

    // share sub
    if is_mqtt_share_subscribe(&context.filter.path) {
        parse_share_subscribe(
            &context.client_pool,
            &context.subscribe_manager,
            &mut ParseShareQueueSubscribeRequest {
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
        add_exclusive_push(AddExclusivePushContext {
            subscribe_manager: context.subscribe_manager.clone(),
            topic: context.topic.clone(),
            client_id: context.client_id.clone(),
            protocol: context.protocol.clone(),
            sub_identifier,
            filter: context.filter.clone(),
            rewrite_sub_path: context.rewrite_sub_path.clone(),
        })
    }
}

async fn parse_share_subscribe(
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &mut ParseShareQueueSubscribeRequest,
) -> ResultMqttBrokerError {
    let (group_name, sub_name) = decode_share_group_and_path(&req.filter.path);
    req.group_name = format!("{group_name}_{sub_name}");
    req.sub_path = sub_name;

    if is_match_sub_and_topic(&req.sub_path, &req.topic_name).is_ok() {
        if is_share_sub_leader(client_pool, &req.group_name).await? {
            add_share_push_leader(subscribe_manager, req).await;
        } else {
            add_share_push_follower(subscribe_manager, req).await;
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
        qos: req.filter.qos,
        nolocal: req.filter.nolocal,
        preserve_retain: req.filter.preserve_retain,
        retain_forward_rule: req.filter.retain_handling.clone(),
        subscription_identifier: req.sub_identifier,
        sub_path: req.filter.path.clone(),
        rewrite_sub_path: None,
        create_time: now_second(),
    };
    subscribe_manager.add_share_subscribe_leader(&req.sub_path, sub);
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
        sub_path: req.sub_path.clone(),
        subscription_identifier: req.sub_identifier,
        topic_name: req.topic_name.clone(),
    };

    subscribe_manager.add_share_subscribe_follower(
        &req.client_id,
        &req.group_name,
        &req.topic_name,
        share_sub,
    );
}

fn add_exclusive_push(context: AddExclusivePushContext) -> ResultMqttBrokerError {
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
            group_name: None,
            qos: context.filter.qos,
            nolocal: context.filter.nolocal,
            preserve_retain: context.filter.preserve_retain,
            retain_forward_rule: context.filter.retain_handling.to_owned(),
            subscription_identifier: context.sub_identifier.to_owned(),
            sub_path: context.filter.path.clone(),
            rewrite_sub_path: context.rewrite_sub_path.clone(),
            create_time: now_second(),
        };
        context.subscribe_manager.add_topic_subscribe(
            &context.topic.topic_name,
            &context.client_id,
            &context.filter.path,
        );
        context.subscribe_manager.add_exclusive_push(
            &context.client_id,
            &context.filter.path,
            &context.topic.topic_name,
            sub,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{add_exclusive_push, AddExclusivePushContext};
    use crate::subscribe::manager::SubscribeManager;
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::{Filter, MqttProtocol};
    use std::sync::Arc;

    #[test]
    fn add_exclusive_push_test() {
        let ex_path = "$exclusive/topic/1/2";
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let topic = MQTTTopic {
            topic_name: "/topic/1/2".to_string(),
            ..Default::default()
        };
        let client_id = unique_id();
        let filter = Filter {
            path: ex_path.to_owned(),
            ..Default::default()
        };
        let res = add_exclusive_push(AddExclusivePushContext {
            subscribe_manager: subscribe_manager.clone(),
            topic: topic.clone(),
            client_id: client_id.clone(),
            protocol: MqttProtocol::Mqtt3,
            sub_identifier: Some(1),
            filter: filter.clone(),
            rewrite_sub_path: None,
        });
        assert!(res.is_ok());
        assert_eq!(subscribe_manager.topic_subscribe_list().len(), 1);
        assert_eq!(subscribe_manager.exclusive_push_list().len(), 1);

        for (_, sub) in subscribe_manager.exclusive_push_list() {
            assert_eq!(sub.client_id, client_id);
            assert_eq!(sub.sub_path, ex_path.to_owned());
            assert_eq!(sub.topic_name, topic.topic_name.to_owned());
        }

        for (to, sub) in subscribe_manager.topic_subscribe_list() {
            for raw in sub {
                assert_eq!(raw.client_id, client_id);
                assert_eq!(raw.path, ex_path.to_owned());
                assert_eq!(to, topic.topic_name.to_owned());
            }
        }
    }
}
