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

use super::cache::MQTTCacheManager;
use crate::core::session::delete_session_by_local;
use crate::core::tool::ResultMqttBrokerError;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::parse::ParseSubscribeData;
use common_base::utils::serialize;
use metadata_struct::mqtt::auto_subscribe::MqttAutoSubscribeRule;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe::MqttSubscribe;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::broker::broker::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use std::sync::Arc;

pub async fn update_mqtt_cache_metadata(
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    record: &UpdateCacheRecord,
) -> ResultMqttBrokerError {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::Session => match record.action_type() {
            BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                let session = serialize::deserialize::<MqttSession>(&record.data)?;
                cache_manager.add_session(&session.client_id, &session);
            }
            BrokerUpdateCacheActionType::Delete => {
                let session = serialize::deserialize::<MqttSession>(&record.data)?;
                delete_session_by_local(
                    cache_manager,
                    subscribe_manager,
                    &session.tenant,
                    &session.client_id,
                );
            }
        },
        BrokerUpdateCacheResourceType::Subscribe => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let subscribe = serialize::deserialize::<MqttSubscribe>(&record.data)?;
                subscribe_manager.add_subscribe(&subscribe);

                subscribe_manager
                    .add_wait_parse_data(ParseSubscribeData {
                        action_type: BrokerUpdateCacheActionType::Create,
                        resource_type: BrokerUpdateCacheResourceType::Subscribe,
                        subscribe: Some(subscribe),
                        topic: None,
                    })
                    .await;
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let subscribe = serialize::deserialize::<MqttSubscribe>(&record.data)?;
                subscribe_manager.remove_by_sub(
                    &subscribe.tenant,
                    &subscribe.client_id,
                    &subscribe.path,
                );
                subscribe_manager
                    .add_wait_parse_data(ParseSubscribeData {
                        action_type: BrokerUpdateCacheActionType::Delete,
                        resource_type: BrokerUpdateCacheResourceType::Subscribe,
                        subscribe: Some(subscribe),
                        topic: None,
                    })
                    .await;
            }
        },

        BrokerUpdateCacheResourceType::AutoSubscribeRule => match record.action_type() {
            BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                let rule = MqttAutoSubscribeRule::decode(&record.data)
                    .map_err(|e| crate::core::error::MqttBrokerError::CommonError(e.to_string()))?;
                cache_manager.add_auto_subscribe_rule(rule);
            }
            BrokerUpdateCacheActionType::Delete => {
                let rule = MqttAutoSubscribeRule::decode(&record.data)
                    .map_err(|e| crate::core::error::MqttBrokerError::CommonError(e.to_string()))?;
                cache_manager.delete_auto_subscribe_rule(&rule.tenant, &rule.name);
            }
        },
        BrokerUpdateCacheResourceType::TopicRewriteRule => match record.action_type() {
            BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                let rule = MqttTopicRewriteRule::decode(&record.data)
                    .map_err(|e| crate::core::error::MqttBrokerError::CommonError(e.to_string()))?;
                cache_manager.add_topic_rewrite_rule(rule);
                cache_manager.set_re_calc_topic_rewrite(true).await;
            }
            BrokerUpdateCacheActionType::Delete => {
                let rule = MqttTopicRewriteRule::decode(&record.data)
                    .map_err(|e| crate::core::error::MqttBrokerError::CommonError(e.to_string()))?;
                cache_manager.delete_topic_rewrite_rule(&rule.tenant, &rule.name);
                cache_manager.set_re_calc_topic_rewrite(true).await;
            }
        },
        _ => {}
    }
    Ok(())
}
