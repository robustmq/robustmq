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
use crate::core::tool::ResultMqttBrokerError;
use crate::core::topic::delete_topic;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::parse::ParseSubscribeData;
use common_base::utils::serialize;
use connector::manager::ConnectorManager;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::connector::MQTTConnector;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use metadata_struct::tenant::Tenant;
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tracing::info;

pub async fn update_mqtt_cache_metadata(
    cache_manager: &Arc<MQTTCacheManager>,
    connector_manager: &Arc<ConnectorManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    metrics_manager: &Arc<MQTTMetricsCache>,
    record: &UpdateCacheRecord,
) -> ResultMqttBrokerError {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::Node => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let node = serialize::deserialize::<BrokerNode>(&record.data)?;
                info!(
                    "Node {} is online. Node information: {:?}",
                    node.node_id, node
                );
                cache_manager.broker_cache.add_node(node);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let node = serialize::deserialize::<BrokerNode>(&record.data)?;
                info!(
                    "Node {} has been taken offline. Node information: {:?}",
                    node.node_id, node
                );
                cache_manager.broker_cache.remove_node(node);
            }
        },

        BrokerUpdateCacheResourceType::Session => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let session = serialize::deserialize::<MqttSession>(&record.data)?;
                cache_manager.add_session(&session.client_id, &session);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let session = serialize::deserialize::<MqttSession>(&record.data)?;
                cache_manager.remove_session(&session.client_id);
            }
        },
        BrokerUpdateCacheResourceType::User => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let user = serialize::deserialize::<MqttUser>(&record.data)?;
                cache_manager.add_user(user);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let user = serialize::deserialize::<MqttUser>(&record.data)?;
                cache_manager.del_user(user.username);
            }
        },
        BrokerUpdateCacheResourceType::Acl => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let acl = serialize::deserialize::<MqttAcl>(&record.data)?;
                cache_manager.add_acl(acl);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let acl = serialize::deserialize::<MqttAcl>(&record.data)?;
                cache_manager.remove_acl(acl);
            }
        },
        BrokerUpdateCacheResourceType::Blacklist => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let blacklist = serialize::deserialize::<MqttAclBlackList>(&record.data)?;
                cache_manager.add_blacklist(blacklist);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let blacklist = serialize::deserialize::<MqttAclBlackList>(&record.data)?;
                cache_manager.remove_blacklist(blacklist);
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
                subscribe_manager.remove_by_sub(&subscribe.client_id, &subscribe.path);
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

        BrokerUpdateCacheResourceType::Topic => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let topic = serialize::deserialize::<Topic>(&record.data)?;
                cache_manager.broker_cache.add_topic(&topic);
                if !cache_manager.topic_rewrite_rule.is_empty() {
                    cache_manager.set_re_calc_topic_rewrite(true).await;
                }
                subscribe_manager
                    .add_wait_parse_data(ParseSubscribeData {
                        action_type: BrokerUpdateCacheActionType::Create,
                        resource_type: BrokerUpdateCacheResourceType::Topic,
                        subscribe: None,
                        topic: Some(topic),
                    })
                    .await;
            }

            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let topic = serialize::deserialize::<Topic>(&record.data)?;
                delete_topic(
                    cache_manager,
                    &topic.tenant,
                    &topic.topic_name,
                    storage_driver_manager,
                    subscribe_manager,
                    metrics_manager,
                )
                .await?;
                subscribe_manager
                    .add_wait_parse_data(ParseSubscribeData {
                        action_type: BrokerUpdateCacheActionType::Delete,
                        resource_type: BrokerUpdateCacheResourceType::Topic,
                        subscribe: None,
                        topic: Some(topic),
                    })
                    .await;
            }
        },
        BrokerUpdateCacheResourceType::Connector => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let connector = serialize::deserialize::<MQTTConnector>(&record.data)?;
                connector_manager.add_connector(&connector);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let connector = serialize::deserialize::<MQTTConnector>(&record.data)?;
                connector_manager.remove_connector(&connector.connector_name);
            }
        },
        BrokerUpdateCacheResourceType::Schema => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let schema = serialize::deserialize::<SchemaData>(&record.data)?;
                schema_manager.add_schema(schema);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let schema = serialize::deserialize::<SchemaData>(&record.data)?;
                schema_manager.remove_schema(&schema.name);
            }
        },
        BrokerUpdateCacheResourceType::SchemaResource => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let schema_resource = serialize::deserialize::<SchemaResourceBind>(&record.data)?;
                schema_manager.add_bind(&schema_resource);
            }

            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let schema_resource = serialize::deserialize::<SchemaResourceBind>(&record.data)?;
                schema_manager.remove_bind(&schema_resource);
            }
        },
        BrokerUpdateCacheResourceType::Tenant => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                let tenant = serialize::deserialize::<Tenant>(&record.data)?;
                cache_manager.broker_cache.add_tenant(tenant);
            }
            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                let tenant = serialize::deserialize::<Tenant>(&record.data)?;
                cache_manager
                    .broker_cache
                    .remove_tenant(&tenant.tenant_name);
            }
        },
        _ => {}
    }
    Ok(())
}
