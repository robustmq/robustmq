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

use crate::controller::call_broker::call::{send_cache_update, BrokerCallManager};
use crate::core::error::MetaServiceError;
use common_base::utils::serialize;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::resource_config::ResourceConfig;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker::broker_common::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType};
use std::sync::Arc;

pub async fn update_cache_by_add_session(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::Session,
        || Ok(serialize::serialize(&session)?),
    )
    .await
}

pub async fn update_cache_by_delete_session(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Session,
        || Ok(serialize::serialize(&session)?),
    )
    .await
}

pub async fn update_cache_by_add_schema(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::Schema,
        || Ok(serialize::serialize(&schema)?),
    )
    .await
}

pub async fn update_cache_by_delete_schema(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Schema,
        || Ok(serialize::serialize(&schema)?),
    )
    .await
}

pub async fn update_cache_by_add_schema_bind(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::SchemaResource,
        || Ok(serialize::serialize(&bind_data)?),
    )
    .await
}

pub async fn update_cache_by_delete_schema_bind(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::SchemaResource,
        || Ok(serialize::serialize(&bind_data)?),
    )
    .await
}

pub async fn update_cache_by_add_connector(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    connector: MQTTConnector,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::Connector,
        || Ok(serialize::serialize(&connector)?),
    )
    .await
}

pub async fn update_cache_by_delete_connector(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    connector: MQTTConnector,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Connector,
        || Ok(serialize::serialize(&connector)?),
    )
    .await
}

pub async fn update_cache_by_add_user(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    user: MqttUser,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::User,
        || Ok(serialize::serialize(&user)?),
    )
    .await
}

pub async fn update_cache_by_delete_user(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    user: MqttUser,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::User,
        || Ok(serialize::serialize(&user)?),
    )
    .await
}

pub async fn update_cache_by_add_subscribe(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    subscribe: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::Subscribe,
        || Ok(serialize::serialize(&subscribe)?),
    )
    .await
}

pub async fn update_cache_by_delete_subscribe(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    subscribe: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Subscribe,
        || Ok(serialize::serialize(&subscribe)?),
    )
    .await
}

pub async fn update_cache_by_add_topic(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: Topic,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::Topic,
        || Ok(serialize::serialize(&topic)?),
    )
    .await
}

pub async fn update_cache_by_delete_topic(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: Topic,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Topic,
        || Ok(serialize::serialize(&topic)?),
    )
    .await
}

pub async fn update_cache_by_add_node(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::Node,
        || Ok(serialize::serialize(&node)?),
    )
    .await
}

pub async fn update_cache_by_delete_node(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Delete,
        BrokerUpdateCacheResourceType::Node,
        || Ok(serialize::serialize(&node)?),
    )
    .await
}

pub async fn update_cache_by_set_resource_config(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    config: ResourceConfig,
) -> Result<(), MetaServiceError> {
    send_cache_update(
        call_manager,
        client_pool,
        BrokerUpdateCacheActionType::Set,
        BrokerUpdateCacheResourceType::ClusterResourceConfig,
        || Ok(serialize::serialize(&config)?),
    )
    .await
}
