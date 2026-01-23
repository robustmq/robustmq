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
use super::dynamic_config::build_cluster_config;
use crate::bridge::manager::ConnectorManager;
use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;
use crate::core::topic::delete_topic;
use crate::storage::auto_subscribe::AutoSubscribeStorage;
use crate::storage::connector::ConnectorStorage;
use crate::storage::schema::SchemaStorage;
use crate::storage::topic::TopicStorage;
use crate::subscribe::parse::ParseSubscribeData;
use crate::{security::AuthDriver, subscribe::manager::SubscribeManager};
use common_base::utils::serialize;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tracing::info;

pub async fn load_metadata_cache(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    auth_driver: &Arc<AuthDriver>,
    connector_manager: &Arc<ConnectorManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
) -> ResultMqttBrokerError {
    let cluster = build_cluster_config(client_pool).await.map_err(|e| {
        MqttBrokerError::CommonError(format!("Failed to load cluster config: {}", e))
    })?;
    cache_manager.broker_cache.set_cluster_config(cluster).await;

    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_list = topic_storage
        .all()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load topics: {}", e)))?;
    for topic in topic_list.iter() {
        cache_manager
            .broker_cache
            .add_topic(&topic.topic_name, &topic.clone());
    }

    let user_list = auth_driver
        .read_all_user()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load users: {}", e)))?;
    for user in user_list.iter() {
        cache_manager.add_user(user.clone());
    }

    let acl_list = auth_driver
        .read_all_acl()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load ACLs: {}", e)))?;
    for acl in acl_list.iter() {
        cache_manager.add_acl(acl.clone());
    }

    let blacklist_list = auth_driver
        .read_all_blacklist()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load blacklist: {}", e)))?;
    for blacklist in blacklist_list.iter() {
        cache_manager.add_blacklist(blacklist.clone());
    }

    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_rewrite_rules = topic_storage.all_topic_rewrite_rule().await.map_err(|e| {
        MqttBrokerError::CommonError(format!("Failed to load topic rewrite rules: {}", e))
    })?;
    for topic_rewrite_rule in topic_rewrite_rules.iter() {
        cache_manager.add_topic_rewrite_rule(topic_rewrite_rule.clone());
    }

    let connector_storage = ConnectorStorage::new(client_pool.clone());
    let connectors = connector_storage
        .list_all_connectors()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load connectors: {}", e)))?;
    for connector in connectors.iter() {
        connector_manager.add_connector(connector);
    }

    let schema_storage = SchemaStorage::new(client_pool.clone());
    let schemas = schema_storage
        .list("".to_string())
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load schemas: {}", e)))?;
    for schema in schemas.iter() {
        schema_manager.add_schema(schema.clone());
    }

    let schema_storage = SchemaStorage::new(client_pool.clone());
    let schema_binds = schema_storage
        .list_bind()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load schema binds: {}", e)))?;
    for schema in schema_binds.iter() {
        schema_manager.add_bind(schema);
    }

    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    let auto_subscribe_rules = auto_subscribe_storage
        .list_auto_subscribe_rule()
        .await
        .map_err(|e| {
            MqttBrokerError::CommonError(format!("Failed to load auto subscribe rules: {}", e))
        })?;
    for auto_subscribe_rule in auto_subscribe_rules.iter() {
        cache_manager.add_auto_subscribe_rule(auto_subscribe_rule.clone());
    }

    info!(
        "Cache loading successful: topics={}, users={}, acls={}, blacklist={}, topic_rewrite_rules={}, connectors={}, schemas={}, auto_subscribe_rules={}",
        topic_list.len(),
        user_list.len(),
        acl_list.len(),
        blacklist_list.len(),
        topic_rewrite_rules.len(),
        connectors.len(),
        schemas.len(),
        auto_subscribe_rules.len(),
    );

    Ok(())
}

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
                cache_manager
                    .broker_cache
                    .add_topic(&topic.topic_name, &topic);
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
        _ => {}
    }
    Ok(())
}
