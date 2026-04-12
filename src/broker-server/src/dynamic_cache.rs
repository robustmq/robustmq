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

use broker_core::cache::NodeCacheManager;
use broker_core::dynamic_config::{update_cluster_dynamic_config, ClusterDynamicConfig};
use common_base::error::{common::CommonError, ResultCommonError};
use common_base::utils::serialize;
use common_security::manager::SecurityManager;
use metadata_struct::auth::acl::SecurityAcl;
use metadata_struct::auth::blacklist::SecurityBlackList;
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::resource_config::ResourceConfig;
use metadata_struct::tenant::Tenant;
use mqtt_broker::{
    broker::MqttBrokerServerParams, core::dynamic_cache::update_mqtt_cache_metadata,
};
use nats_broker::{
    broker::NatsBrokerServerParams, core::dynamic_cache::update_nats_cache_metadata,
};
use protocol::broker::broker::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use std::str::FromStr;
use std::sync::Arc;
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
        | BrokerUpdateCacheResourceType::Topic
        | BrokerUpdateCacheResourceType::Connector
        | BrokerUpdateCacheResourceType::Schema
        | BrokerUpdateCacheResourceType::SchemaResource
        | BrokerUpdateCacheResourceType::AutoSubscribeRule
        | BrokerUpdateCacheResourceType::TopicRewriteRule => {
            if let Err(e) = update_mqtt_cache_metadata(
                &mqtt_params.cache_manager,
                &mqtt_params.connector_manager,
                &mqtt_params.subscribe_manager,
                &mqtt_params.schema_manager,
                &mqtt_params.storage_driver_manager,
                &mqtt_params.security_manager,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

        // Cluster — Node, Config, Tenant, User, Acl, Blacklist, Group
        BrokerUpdateCacheResourceType::ClusterResourceConfig
        | BrokerUpdateCacheResourceType::Node
        | BrokerUpdateCacheResourceType::Tenant
        | BrokerUpdateCacheResourceType::User
        | BrokerUpdateCacheResourceType::Acl
        | BrokerUpdateCacheResourceType::Blacklist
        | BrokerUpdateCacheResourceType::Group => {
            if let Err(e) = update_cluster_cache_metadata(
                &mqtt_params.cache_manager.node_cache,
                &mqtt_params.security_manager,
                record,
            )
            .await
            {
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
    node_cache: &Arc<NodeCacheManager>,
    security_manager: &Arc<SecurityManager>,
    record: &UpdateCacheRecord,
) -> Result<(), CommonError> {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::Node => {
            let node: BrokerNode = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    node_cache.add_node(node);
                }
                BrokerUpdateCacheActionType::Delete => {
                    node_cache.remove_node(node);
                }
            }
        }
        BrokerUpdateCacheResourceType::ClusterResourceConfig => {
            let config: ResourceConfig = serialize::deserialize(&record.data)?;
            if let Ok(config_type) = ClusterDynamicConfig::from_str(&config.resource) {
                update_cluster_dynamic_config(node_cache, config_type, config.config)?;
            }
        }
        BrokerUpdateCacheResourceType::Tenant => {
            let tenant: Tenant = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    node_cache.add_tenant(tenant);
                }
                BrokerUpdateCacheActionType::Delete => {
                    node_cache.remove_tenant(&tenant.tenant_name);
                }
            }
        }
        BrokerUpdateCacheResourceType::User => {
            let user: SecurityUser = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    security_manager.metadata.add_user(user);
                }
                BrokerUpdateCacheActionType::Delete => {
                    security_manager
                        .metadata
                        .del_user(&user.tenant, &user.username);
                }
            }
        }
        BrokerUpdateCacheResourceType::Acl => {
            let acl: SecurityAcl = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    security_manager.metadata.add_acl(acl);
                }
                BrokerUpdateCacheActionType::Delete => {
                    security_manager.metadata.remove_acl(acl);
                }
            }
        }
        BrokerUpdateCacheResourceType::Blacklist => {
            let blacklist: SecurityBlackList = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    security_manager.metadata.add_blacklist(blacklist);
                }
                BrokerUpdateCacheActionType::Delete => {
                    security_manager.metadata.remove_blacklist(blacklist);
                }
            }
        }
        BrokerUpdateCacheResourceType::Group => {}
        _ => {}
    }
    Ok(())
}
