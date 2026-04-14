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

use broker_core::dynamic_config::{update_cluster_dynamic_config, ClusterDynamicConfig};
use common_base::error::{common::CommonError, ResultCommonError};
use common_base::utils::serialize;
use metadata_struct::auth::acl::SecurityAcl;
use metadata_struct::auth::blacklist::SecurityBlackList;
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::connector::MQTTConnector;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::resource_config::ResourceConfig;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use metadata_struct::tenant::Tenant;
use metadata_struct::topic::Topic;
use mqtt_broker::core::topic::{create_topic_by_mqtt, delete_topic_by_mqtt};
use mqtt_broker::{
    broker::MqttBrokerServerParams, core::dynamic_cache::update_mqtt_cache_metadata,
};
use nats_broker::core::topic::{create_topic_by_nats_and_mq9, delete_topic_by_nats_and_mq9};
use nats_broker::{
    broker::NatsBrokerServerParams, core::dynamic_cache::update_nats_cache_metadata,
};
use protocol::broker::broker::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use std::str::FromStr;
use storage_engine::{core::dynamic_cache::update_storage_cache_metadata, StorageEngineParams};

pub async fn update_cache(
    mqtt_params: &MqttBrokerServerParams,
    nats_params: &NatsBrokerServerParams,
    storage_params: &StorageEngineParams,
    record: &UpdateCacheRecord,
) -> ResultCommonError {
    match record.resource_type() {
        // MQTT Broker
        BrokerUpdateCacheResourceType::Session
        | BrokerUpdateCacheResourceType::Subscribe
        | BrokerUpdateCacheResourceType::AutoSubscribeRule
        | BrokerUpdateCacheResourceType::TopicRewriteRule => {
            if let Err(e) = update_mqtt_cache_metadata(
                &mqtt_params.cache_manager,
                &mqtt_params.subscribe_manager,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

        // Cluster — Node, Config, Tenant, User, Acl, Blacklist, Group, Connector, Schema, Topic
        BrokerUpdateCacheResourceType::ClusterResourceConfig
        | BrokerUpdateCacheResourceType::Node
        | BrokerUpdateCacheResourceType::Tenant
        | BrokerUpdateCacheResourceType::User
        | BrokerUpdateCacheResourceType::Acl
        | BrokerUpdateCacheResourceType::Blacklist
        | BrokerUpdateCacheResourceType::Group
        | BrokerUpdateCacheResourceType::Connector
        | BrokerUpdateCacheResourceType::Schema
        | BrokerUpdateCacheResourceType::SchemaResource
        | BrokerUpdateCacheResourceType::Topic => {
            if let Err(e) = update_cluster_cache_metadata(mqtt_params, nats_params, record).await {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

        // NATS / MQ9
        BrokerUpdateCacheResourceType::NatsSubscribe | BrokerUpdateCacheResourceType::Mq9Email => {
            if let Err(e) = update_nats_cache_metadata(
                &nats_params.cache_manager,
                &nats_params.subscribe_manager,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

        // Storage Engine
        BrokerUpdateCacheResourceType::Shard
        | BrokerUpdateCacheResourceType::Segment
        | BrokerUpdateCacheResourceType::SegmentMeta => {
            if let Err(e) = update_storage_cache_metadata(
                &storage_params.cache_manager,
                &storage_params.rocksdb_engine_handler,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }
    Ok(())
}

pub async fn update_cluster_cache_metadata(
    mqtt_params: &MqttBrokerServerParams,
    nats_params: &NatsBrokerServerParams,
    record: &UpdateCacheRecord,
) -> Result<(), CommonError> {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::Node => {
            let node: BrokerNode = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params.node_cache.add_node(node);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params.node_cache.remove_node(node);
                }
            }
        }

        BrokerUpdateCacheResourceType::ClusterResourceConfig => {
            let config: ResourceConfig = serialize::deserialize(&record.data)?;
            if let Ok(config_type) = ClusterDynamicConfig::from_str(&config.resource) {
                update_cluster_dynamic_config(&mqtt_params.node_cache, config_type, config.config)?;
            }
        }

        BrokerUpdateCacheResourceType::Topic => match record.action_type() {
            BrokerUpdateCacheActionType::Create => {
                // Cache
                let topic = serialize::deserialize::<Topic>(&record.data)?;
                mqtt_params.node_cache.add_topic(&topic);

                // MQTT Broker
                create_topic_by_mqtt(
                    &mqtt_params.cache_manager,
                    &mqtt_params.subscribe_manager,
                    &topic,
                )
                .await?;

                // Nats & MQ9
                create_topic_by_nats_and_mq9(nats_params.subscribe_manager.clone(), &topic).await;

                // Kafka
            }

            BrokerUpdateCacheActionType::Update => {}
            BrokerUpdateCacheActionType::Delete => {
                // Cache
                let topic = serialize::deserialize::<Topic>(&record.data)?;
                mqtt_params
                    .node_cache
                    .delete_topic(&topic.tenant, &topic.topic_name);

                // MQTT Broker
                delete_topic_by_mqtt(&topic, &mqtt_params.subscribe_manager).await?;

                // Nats & MQ9
                delete_topic_by_nats_and_mq9(nats_params.subscribe_manager.clone(), &topic).await;

                // Kafka
            }
        },

        BrokerUpdateCacheResourceType::Tenant => {
            let tenant: Tenant = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params.node_cache.add_tenant(tenant);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params.node_cache.remove_tenant(&tenant.tenant_name);
                }
            }
        }

        BrokerUpdateCacheResourceType::User => {
            let user: SecurityUser = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params.security_manager.metadata.add_user(user);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params
                        .security_manager
                        .metadata
                        .del_user(&user.tenant, &user.username);
                }
            }
        }

        BrokerUpdateCacheResourceType::Acl => {
            let acl: SecurityAcl = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params.security_manager.metadata.add_acl(acl);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params.security_manager.metadata.remove_acl(acl);
                }
            }
        }

        BrokerUpdateCacheResourceType::Blacklist => {
            let blacklist: SecurityBlackList = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params
                        .security_manager
                        .metadata
                        .add_blacklist(blacklist);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params
                        .security_manager
                        .metadata
                        .remove_blacklist(blacklist);
                }
            }
        }

        BrokerUpdateCacheResourceType::Connector => {
            let connector: MQTTConnector = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params.connector_manager.add_connector(&connector);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params
                        .connector_manager
                        .remove_connector(&connector.connector_name);
                }
            }
        }

        BrokerUpdateCacheResourceType::Schema => {
            let schema: SchemaData = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params.schema_manager.add_schema(schema);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params
                        .schema_manager
                        .remove_schema(&schema.tenant, &schema.name);
                }
            }
        }

        BrokerUpdateCacheResourceType::SchemaResource => {
            let schema_resource: SchemaResourceBind = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    mqtt_params.schema_manager.add_bind(&schema_resource);
                }
                BrokerUpdateCacheActionType::Delete => {
                    mqtt_params.schema_manager.remove_bind(&schema_resource);
                }
            }
        }

        BrokerUpdateCacheResourceType::Group => {}
        _ => {}
    }
    Ok(())
}
