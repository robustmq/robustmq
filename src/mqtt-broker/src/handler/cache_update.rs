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
use crate::storage::auto_subscribe::AutoSubscribeStorage;
use crate::storage::connector::ConnectorStorage;
use crate::storage::topic::TopicStorage;
use crate::{security::AuthDriver, subscribe::subscribe_manager::SubscribeManager};
use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::placement::inner::call::list_schema;
use grpc_clients::pool::ClientPool;
use log::error;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MqttTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker_mqtt::broker_mqtt_inner::{
    MqttBrokerUpdateCacheActionType, MqttBrokerUpdateCacheResourceType, UpdateMqttCacheRequest,
};
use protocol::placement_center::placement_center_inner::ListSchemaRequest;
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;

use super::cache::CacheManager;
use super::cluster_config::build_cluster_config;

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
            panic!(
                "Failed to load the cluster configuration with error message:{}",
                e
            );
        }
    };
    cache_manager.set_cluster_info(cluster);

    // load all topic
    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_list = match topic_storage.all().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the topic list with error message:{}", e);
        }
    };

    for (_, topic) in topic_list {
        cache_manager.add_topic(&topic.topic_name, &topic);
    }

    // load all user
    let user_list = match auth_driver.read_all_user().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the user list with error message:{}", e);
        }
    };

    for (_, user) in user_list {
        cache_manager.add_user(user);
    }

    // load all acl
    let acl_list = match auth_driver.read_all_acl().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the acl list with error message:{}", e);
        }
    };
    for acl in acl_list {
        cache_manager.add_acl(acl);
    }

    // load all blacklist
    let blacklist_list = match auth_driver.read_all_blacklist().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the blacklist list with error message:{}", e);
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
            panic!(
                "Failed to load the topic_rewrite_rule list with error message:{}",
                e
            );
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
            panic!("Failed to load the connector list with error message:{}", e);
        }
    };
    for connector in connectors.iter() {
        connector_manager.add_connector(connector);
    }

    // load all schemas
    let config = broker_mqtt_conf();
    let request = ListSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: "".to_owned(),
    };

    match list_schema(client_pool, &config.placement_center, request).await {
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
            panic!("Failed to load the schema list with error message:{}", e);
        }
    }

    // load all auto subscribe rule
    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    let auto_subscribe_rules = match auto_subscribe_storage.list_auto_subscribe_rule().await {
        Ok(list) => list,
        Err(e) => {
            panic!(
                "Failed to load the auto subscribe list with error message:{}",
                e
            );
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
) {
    match request.resource_type() {
        MqttBrokerUpdateCacheResourceType::Session => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttSession>(&request.data) {
                    Ok(session) => {
                        cache_manager.add_session(session.client_id.clone(), session);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttSession>(&request.data) {
                    Ok(session) => {
                        cache_manager.remove_session(&session.client_id);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::User => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttUser>(&request.data) {
                    Ok(user) => {
                        cache_manager.add_user(user);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttUser>(&request.data) {
                    Ok(user) => {
                        cache_manager.del_user(user.username);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::Subscribe => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttSubscribe>(&request.data) {
                    Ok(subscribe) => {
                        subscribe_manager.add_subscribe(subscribe);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttSubscribe>(&request.data) {
                    Ok(subscribe) => {
                        subscribe_manager.remove_subscribe(&subscribe.client_id, &subscribe.path)
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::Topic => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttTopic>(&request.data) {
                    Ok(topic) => {
                        cache_manager.add_topic(&topic.topic_name, &topic);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttTopic>(&request.data) {
                    Ok(topic) => {
                        cache_manager.delete_topic(&topic.topic_name, &topic);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::Connector => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MQTTConnector>(&request.data) {
                    Ok(connector) => {
                        connector_manager.add_connector(&connector);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MQTTConnector>(&request.data) {
                    Ok(connector) => {
                        connector_manager.remove_connector(&connector.connector_name);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::Schema => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<SchemaData>(&request.data) {
                    Ok(schema) => {
                        schema_manager.add_schema(schema);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<SchemaData>(&request.data) {
                    Ok(schema) => {
                        schema_manager.remove_schema(&schema.name);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::SchemaResource => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<SchemaResourceBind>(&request.data) {
                    Ok(schema_bind) => {
                        schema_manager.add_schema_resource(&schema_bind);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }

            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<SchemaResourceBind>(&request.data) {
                    Ok(schema_bind) => {
                        schema_manager.remove_resource_schema(
                            &schema_bind.resource_name,
                            &schema_bind.schema_name,
                        );
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
    }
}
