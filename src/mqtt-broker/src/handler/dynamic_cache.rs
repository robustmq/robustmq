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
use crate::handler::tool::ResultMqttBrokerError;
use crate::handler::topic::delete_topic;
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
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tracing::info;

pub async fn load_metadata_cache(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    auth_driver: &Arc<AuthDriver>,
    connector_manager: &Arc<ConnectorManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
) -> ResultMqttBrokerError {
    // load cluster config
    let cluster = build_cluster_config(client_pool).await?;
    cache_manager.broker_cache.set_cluster_config(cluster).await;

    // load all topic
    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_list = topic_storage.all().await?;
    for topic in topic_list.iter() {
        cache_manager.add_topic(&topic.topic_name, &topic.clone());
    }

    // load all user
    let user_list = auth_driver.read_all_user().await?;
    for user in user_list.iter() {
        cache_manager.add_user(user.clone());
    }

    // load all acl
    let acl_list = auth_driver.read_all_acl().await?;
    for acl in acl_list.iter() {
        cache_manager.add_acl(acl.clone());
    }

    // load all blacklist
    let blacklist_list = auth_driver.read_all_blacklist().await?;
    for blacklist in blacklist_list.iter() {
        cache_manager.add_blacklist(blacklist.clone());
    }

    // load All topic_rewrite rule
    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_rewrite_rules = topic_storage.all_topic_rewrite_rule().await?;
    for topic_rewrite_rule in topic_rewrite_rules.iter() {
        cache_manager.add_topic_rewrite_rule(topic_rewrite_rule.clone());
    }

    // load all connectors
    let connector_storage = ConnectorStorage::new(client_pool.clone());
    let connectors = connector_storage.list_all_connectors().await?;
    for connector in connectors.iter() {
        connector_manager.add_connector(connector);
    }

    // load all schemas
    let schema_storage = SchemaStorage::new(client_pool.clone());
    let schemas = schema_storage.list("".to_string()).await?;
    for schema in schemas.iter() {
        schema_manager.add_schema(schema.clone());
    }

    // load all schema binds
    let schema_storage = SchemaStorage::new(client_pool.clone());
    let schemas = schema_storage.list_bind().await?;
    for schema in schemas.iter() {
        schema_manager.add_bind(schema);
    }

    // load all auto subscribe rule
    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    let auto_subscribe_rules = auto_subscribe_storage.list_auto_subscribe_rule().await?;
    for auto_subscribe_rule in auto_subscribe_rules.iter() {
        cache_manager.add_auto_subscribe_rule(auto_subscribe_rule.clone());
    }

    info!(
        "Cache loading successful.topic:{},user:{},acl:{},blacklist:{},topic_rewrite_rule:{},connectors:{},schemas:{},auto_subscribe_rules:{}",
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
    message_storage_adapter: &ArcStorageAdapter,
    metrics_manager: &Arc<MQTTMetricsCache>,
    record: &UpdateCacheRecord,
) -> ResultMqttBrokerError {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::Node => match record.action_type() {
            BrokerUpdateCacheActionType::Set => {
                let node = serialize::deserialize::<BrokerNode>(&record.data)?;
                info!(
                    "Node {} is online. Node information: {:?}",
                    node.node_id, node
                );
                cache_manager.broker_cache.add_node(node);
            }
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
            BrokerUpdateCacheActionType::Set => {
                let session = serialize::deserialize::<MqttSession>(&record.data)?;
                cache_manager.add_session(&session.client_id, &session);
            }
            BrokerUpdateCacheActionType::Delete => {
                let session = serialize::deserialize::<MqttSession>(&record.data)?;
                cache_manager.remove_session(&session.client_id);
            }
        },
        BrokerUpdateCacheResourceType::User => match record.action_type() {
            BrokerUpdateCacheActionType::Set => {
                let user = serialize::deserialize::<MqttUser>(&record.data)?;
                cache_manager.add_user(user);
            }
            BrokerUpdateCacheActionType::Delete => {
                let user = serialize::deserialize::<MqttUser>(&record.data)?;
                cache_manager.del_user(user.username);
            }
        },

        BrokerUpdateCacheResourceType::Subscribe => match record.action_type() {
            BrokerUpdateCacheActionType::Set => {
                let subscribe = serialize::deserialize::<MqttSubscribe>(&record.data)?;
                subscribe_manager.add_subscribe(&subscribe);

                subscribe_manager
                    .add_wait_parse_data(ParseSubscribeData {
                        action_type: BrokerUpdateCacheActionType::Set,
                        resource_type: BrokerUpdateCacheResourceType::Subscribe,
                        subscribe: Some(subscribe),
                        topic: None,
                    })
                    .await;
            }
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
            BrokerUpdateCacheActionType::Set => {
                let topic = serialize::deserialize::<MQTTTopic>(&record.data)?;
                cache_manager.add_topic(&topic.topic_name, &topic);
                subscribe_manager
                    .add_wait_parse_data(ParseSubscribeData {
                        action_type: BrokerUpdateCacheActionType::Set,
                        resource_type: BrokerUpdateCacheResourceType::Topic,
                        subscribe: None,
                        topic: Some(topic),
                    })
                    .await;
            }

            BrokerUpdateCacheActionType::Delete => {
                let topic = serialize::deserialize::<MQTTTopic>(&record.data)?;
                delete_topic(
                    cache_manager,
                    &topic.topic_name,
                    message_storage_adapter,
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
            BrokerUpdateCacheActionType::Set => {
                let connector = serialize::deserialize::<MQTTConnector>(&record.data)?;
                connector_manager.add_connector(&connector);
            }
            BrokerUpdateCacheActionType::Delete => {
                let connector = serialize::deserialize::<MQTTConnector>(&record.data)?;
                connector_manager.remove_connector(&connector.connector_name);
            }
        },
        BrokerUpdateCacheResourceType::Schema => match record.action_type() {
            BrokerUpdateCacheActionType::Set => {
                let schema = serialize::deserialize::<SchemaData>(&record.data)?;
                schema_manager.add_schema(schema);
            }
            BrokerUpdateCacheActionType::Delete => {
                let schema = serialize::deserialize::<SchemaData>(&record.data)?;
                schema_manager.remove_schema(&schema.name);
            }
        },
        BrokerUpdateCacheResourceType::SchemaResource => match record.action_type() {
            BrokerUpdateCacheActionType::Set => {
                let schema_resource = serialize::deserialize::<SchemaResourceBind>(&record.data)?;
                schema_manager.add_bind(&schema_resource);
            }

            BrokerUpdateCacheActionType::Delete => {
                let schema_resource = serialize::deserialize::<SchemaResourceBind>(&record.data)?;
                schema_manager.remove_bind(&schema_resource);
            }
        },
        _ => {}
    }
    Ok(())
}
