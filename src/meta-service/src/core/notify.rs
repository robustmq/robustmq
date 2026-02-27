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

use crate::core::error::MetaServiceError;
use common_base::utils::serialize;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::resource_config::ResourceConfig;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use metadata_struct::storage::{
    segment::EngineSegment, segment_meta::EngineSegmentMetadata, shard::EngineShard,
};
use node_call::{NodeCallData, NodeCallManager, UpdateCacheData};
use protocol::broker::broker_common::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType};
use std::sync::Arc;

// MQTT Session
pub async fn send_notify_by_add_session(
    call_manager: &Arc<NodeCallManager>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Session,
        serialize::serialize(&session)?,
    )
    .await
}

pub async fn send_notify_by_delete_session(
    call_manager: &Arc<NodeCallManager>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Session,
        serialize::serialize(&session)?,
    )
    .await
}

// MQTT Schema
pub async fn send_notify_by_add_schema(
    call_manager: &Arc<NodeCallManager>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Schema,
        serialize::serialize(&schema)?,
    )
    .await
}

pub async fn send_notify_by_delete_schema(
    call_manager: &Arc<NodeCallManager>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Schema,
        serialize::serialize(&schema)?,
    )
    .await
}

pub async fn send_notify_by_add_schema_bind(
    call_manager: &Arc<NodeCallManager>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::SchemaResource,
        serialize::serialize(&bind_data)?,
    )
    .await
}

pub async fn send_notify_by_delete_schema_bind(
    call_manager: &Arc<NodeCallManager>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::SchemaResource,
        serialize::serialize(&bind_data)?,
    )
    .await
}

// MQTT Connector
pub async fn send_notify_by_add_connector(
    call_manager: &Arc<NodeCallManager>,
    connector: MQTTConnector,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Connector,
        serialize::serialize(&connector)?,
    )
    .await
}

pub async fn send_notify_by_delete_connector(
    call_manager: &Arc<NodeCallManager>,
    connector: MQTTConnector,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Connector,
        serialize::serialize(&connector)?,
    )
    .await
}

// MQTT User
pub async fn send_notify_by_add_user(
    call_manager: &Arc<NodeCallManager>,
    user: MqttUser,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::User,
        serialize::serialize(&user)?,
    )
    .await
}

pub async fn send_notify_by_delete_user(
    call_manager: &Arc<NodeCallManager>,
    user: MqttUser,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::User,
        serialize::serialize(&user)?,
    )
    .await
}

// MQTT Subscribe
pub async fn send_notify_by_add_subscribe(
    call_manager: &Arc<NodeCallManager>,
    subscribe: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Subscribe,
        serialize::serialize(&subscribe)?,
    )
    .await
}

pub async fn send_notify_by_delete_subscribe(
    call_manager: &Arc<NodeCallManager>,
    subscribe: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Subscribe,
        serialize::serialize(&subscribe)?,
    )
    .await
}

// MQTT Topic
pub async fn send_notify_by_add_topic(
    call_manager: &Arc<NodeCallManager>,
    topic: Topic,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Topic,
        serialize::serialize(&topic)?,
    )
    .await
}

pub async fn send_notify_by_delete_topic(
    call_manager: &Arc<NodeCallManager>,
    topic: Topic,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Topic,
        serialize::serialize(&topic)?,
    )
    .await
}

// Cluster Node
pub async fn send_notify_by_add_node(
    call_manager: &Arc<NodeCallManager>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Node,
        serialize::serialize(&node)?,
    )
    .await
}

pub async fn send_notify_by_delete_node(
    call_manager: &Arc<NodeCallManager>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Node,
        serialize::serialize(&node)?,
    )
    .await
}

// Cluster Config
pub async fn send_notify_by_set_resource_config(
    call_manager: &Arc<NodeCallManager>,
    config: ResourceConfig,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::ClusterResourceConfig,
        serialize::serialize(&config)?,
    )
    .await
}

// Storage Shard
pub async fn send_notify_by_set_shard(
    call_manager: &Arc<NodeCallManager>,
    shard_info: EngineShard,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Shard,
        shard_info.encode()?,
    )
    .await
}

pub async fn send_notify_by_delete_shard(
    call_manager: &Arc<NodeCallManager>,
    shard_info: EngineShard,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Shard,
        shard_info.encode()?,
    )
    .await
}

// Storage Segment
pub async fn send_notify_by_set_segment(
    call_manager: &Arc<NodeCallManager>,
    segment_info: EngineSegment,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::Segment,
        segment_info.encode()?,
    )
    .await
}

pub async fn send_notify_by_delete_segment(
    call_manager: &Arc<NodeCallManager>,
    segment_info: EngineSegment,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Segment,
        segment_info.encode()?,
    )
    .await
}

// Storage Segment Metadata
pub async fn send_notify_by_set_segment_meta(
    call_manager: &Arc<NodeCallManager>,
    segment_info: EngineSegmentMetadata,
) -> Result<(), MetaServiceError> {
    send_update_cache(
        call_manager,
        BrokerUpdateCacheActionType::Create,
        BrokerUpdateCacheResourceType::SegmentMeta,
        segment_info.encode()?,
    )
    .await
}

// Build and push one cache update notification into node-call manager.
async fn send_update_cache(
    call_manager: &Arc<NodeCallManager>,
    action_type: BrokerUpdateCacheActionType,
    resource_type: BrokerUpdateCacheResourceType,
    data: Vec<u8>,
) -> Result<(), MetaServiceError> {
    let data = NodeCallData::UpdateCache(UpdateCacheData {
        action_type,
        resource_type,
        data,
    });
    call_manager.send(data).await?;
    Ok(())
}
