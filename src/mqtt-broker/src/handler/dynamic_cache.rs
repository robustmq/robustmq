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

use crate::bridge::manager::ConnectorManager;
use crate::common::types::ResultMqttBrokerError;
use crate::handler::dynamic_config::{update_cluster_dynamic_config, ClusterDynamicConfig};
use crate::storage::auto_subscribe::AutoSubscribeStorage;
use crate::storage::connector::ConnectorStorage;
use crate::storage::topic::TopicStorage;
use crate::{security::AuthDriver, subscribe::manager::SubscribeManager};

use common_config::broker::broker_config;
use grpc_clients::placement::inner::call::list_schema;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::placement::node::BrokerNode;
use metadata_struct::resource_config::ClusterResourceConfig;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker_mqtt::broker_mqtt_inner::{
    MqttBrokerUpdateCacheActionType, MqttBrokerUpdateCacheResourceType, UpdateMqttCacheRequest,
};
use protocol::placement_center::placement_center_inner::ListSchemaRequest;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use tracing::{error, info};

use super::cache::CacheManager;
use super::dynamic_config::build_cluster_config;

pub async fn load_metadata_cache(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    auth_driver: &Arc<AuthDriver>,
    connector_manager: &Arc<ConnectorManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
) {
    // load cluster config
    let cluster = match build_cluster_config(client_pool).await {
        Ok(cluster) => cluster,
        Err(e) => {
            panic!("Failed to load the cluster configuration with error message:{e}");
        }
    };
    cache_manager.set_cluster_config(cluster);

    // load all topic
    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_list = match topic_storage.all().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the topic list with error message:{e}");
        }
    };

    for (_, topic) in topic_list {
        cache_manager.add_topic(&topic.topic_name, &topic);
    }

    // load all user
    let user_list = match auth_driver.read_all_user().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the user list with error message:{e}");
        }
    };

    for (_, user) in user_list {
        cache_manager.add_user(user);
    }

    // load all acl
    let acl_list = match auth_driver.read_all_acl().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the acl list with error message:{e}");
        }
    };
    for acl in acl_list {
        cache_manager.add_acl(acl);
    }

    // load all blacklist
    let blacklist_list = match auth_driver.read_all_blacklist().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the blacklist list with error message:{e}");
        }
    };
    for blacklist in blacklist_list {
        cache_manager.add_blacklist(blacklist);
    }

    // load All topic_rewrite rule
    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_rewrite_rules = match topic_storage.all_topic_rewrite_rule().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the topic_rewrite_rule list with error message:{e}");
        }
    };
    for topic_rewrite_rule in topic_rewrite_rules {
        cache_manager.add_topic_rewrite_rule(topic_rewrite_rule);
    }

    // load all connectors
    let connector_storage = ConnectorStorage::new(client_pool.clone());
    let connectors = match connector_storage.list_all_connectors().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the connector list with error message:{e}");
        }
    };
    for connector in connectors.iter() {
        connector_manager.add_connector(connector);
    }

    // load all schemas
    let config = broker_config();
    let request = ListSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: "".to_owned(),
    };

    match list_schema(client_pool, &config.get_placement_center_addr(), request).await {
        Ok(reply) => {
            for raw in reply.schemas {
                match serde_json::from_slice::<SchemaData>(raw.as_slice()) {
                    Ok(schema) => {
                        schema_manager.add_schema(schema);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
        Err(e) => {
            panic!("Failed to load the schema list with error message:{e}");
        }
    }

    // load all auto subscribe rule
    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    let auto_subscribe_rules = match auto_subscribe_storage.list_auto_subscribe_rule().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the auto subscribe list with error message:{e}");
        }
    };
    for auto_subscribe_rule in auto_subscribe_rules {
        cache_manager.add_auto_subscribe_rule(auto_subscribe_rule);
    }
}

pub async fn update_cache_metadata(
    cache_manager: &Arc<CacheManager>,
    connector_manager: &Arc<ConnectorManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
    request: UpdateMqttCacheRequest,
) -> ResultMqttBrokerError {
    match request.resource_type() {
        MqttBrokerUpdateCacheResourceType::Node => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let node = serde_json::from_str::<BrokerNode>(&request.data)?;
                info!(
                    "Node {} is online. Node information: {:?}",
                    node.node_id, node
                );
                cache_manager.add_node(node);
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                let node = serde_json::from_str::<BrokerNode>(&request.data)?;
                info!(
                    "Node {} has been taken offline. Node information: {:?}",
                    node.node_id, node
                );
                cache_manager.remove_node(node);
            }
        },

        MqttBrokerUpdateCacheResourceType::Session => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let session = serde_json::from_str::<MqttSession>(&request.data)?;
                cache_manager.add_session(&session.client_id, &session);
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                let session = serde_json::from_str::<MqttSession>(&request.data)?;
                cache_manager.remove_session(&session.client_id);
            }
        },
        MqttBrokerUpdateCacheResourceType::User => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let user = serde_json::from_str::<MqttUser>(&request.data)?;
                cache_manager.add_user(user);
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                let user = serde_json::from_str::<MqttUser>(&request.data)?;
                cache_manager.del_user(user.username);
            }
        },
        MqttBrokerUpdateCacheResourceType::Subscribe => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let subscribe = serde_json::from_str::<MqttSubscribe>(&request.data)?;
                subscribe_manager.add_subscribe(subscribe);
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                let subscribe = serde_json::from_str::<MqttSubscribe>(&request.data)?;
                subscribe_manager.remove_subscribe(&subscribe.client_id, &subscribe.path);
            }
        },
        MqttBrokerUpdateCacheResourceType::Topic => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let topic = serde_json::from_str::<MQTTTopic>(&request.data)?;
                cache_manager.add_topic(&topic.topic_name, &topic);
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                let topic = serde_json::from_str::<MQTTTopic>(&request.data)?;
                cache_manager.delete_topic(&topic.topic_name, &topic);
            }
        },
        MqttBrokerUpdateCacheResourceType::Connector => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let connector = serde_json::from_str::<MQTTConnector>(&request.data)?;
                connector_manager.add_connector(&connector);
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                let connector = serde_json::from_str::<MQTTConnector>(&request.data)?;
                connector_manager.remove_connector(&connector.connector_name);
            }
        },
        MqttBrokerUpdateCacheResourceType::Schema => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let schema = serde_json::from_str::<SchemaData>(&request.data)?;
                schema_manager.add_schema(schema);
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                let schema = serde_json::from_str::<SchemaData>(&request.data)?;
                schema_manager.remove_schema(&schema.name);
            }
        },
        MqttBrokerUpdateCacheResourceType::SchemaResource => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let schema_resource = serde_json::from_str::<SchemaResourceBind>(&request.data)?;
                schema_manager.add_schema_resource(&schema_resource);
            }

            MqttBrokerUpdateCacheActionType::Delete => {
                let schema_resource = serde_json::from_str::<SchemaResourceBind>(&request.data)?;
                schema_manager.remove_resource_schema(
                    &schema_resource.resource_name,
                    &schema_resource.schema_name,
                );
            }
        },

        MqttBrokerUpdateCacheResourceType::ClusterResourceConfig => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                let data = serde_json::from_str::<ClusterResourceConfig>(&request.data)?;
                let config = data.resource.parse::<ClusterDynamicConfig>()?;
                update_cluster_dynamic_config(cache_manager, config, data.config).await?;
            }
            MqttBrokerUpdateCacheActionType::Delete => {}
        },
    }
    Ok(())
}
