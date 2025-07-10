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
use crate::handler::cache::CacheManager;
use crate::handler::dynamic_cache::update_cache_metadata;
use crate::handler::error::MqttBrokerError;
use crate::handler::lastwill::send_last_will_message;
use crate::subscribe::manager::SubscribeManager;
use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::lastwill::LastWillData;
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateMqttCacheReply, UpdateMqttCacheRequest,
};
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tracing::info;

pub async fn update_cache_by_req(
    cache_manager: &Arc<CacheManager>,
    connector_manager: &Arc<ConnectorManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
    req: &UpdateMqttCacheRequest,
) -> Result<UpdateMqttCacheReply, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    if conf.cluster_name != req.cluster_name {
        return Ok(UpdateMqttCacheReply::default());
    }

    update_cache_metadata(
        cache_manager,
        connector_manager,
        subscribe_manager,
        schema_manager,
        req.clone(),
    )
    .await?;
    Ok(UpdateMqttCacheReply::default())
}

pub async fn delete_session_by_req(
    cache_manager: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &DeleteSessionRequest,
) -> Result<DeleteSessionReply, MqttBrokerError> {
    info!(
        "Received request from Placement center to delete expired Session. Cluster name :{}, clientId: {:?}",
        req.cluster_name, req.client_id
    );
    if cache_manager.cluster_name != req.cluster_name {
        return Err(MqttBrokerError::ClusterNotMatch(req.cluster_name.clone()));
    }

    if req.client_id.is_empty() {
        return Err(MqttBrokerError::ClientIDIsEmpty);
    }

    for client_id in req.client_id.iter() {
        subscribe_manager.remove_client_id(client_id);
        cache_manager.remove_session(client_id);
    }

    Ok(DeleteSessionReply::default())
}

pub async fn send_last_will_message_by_req<S>(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    message_storage_adapter: &Arc<S>,
    req: &SendLastWillMessageRequest,
) -> Result<SendLastWillMessageReply, MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let data = match serde_json::from_slice::<LastWillData>(req.last_will_message.as_slice()) {
        Ok(data) => data,
        Err(e) => {
            return Err(MqttBrokerError::CommonError(e.to_string()));
        }
    };
    info!(
        "Received will message from placement center, source client id: {},data:{:?}",
        req.client_id, data
    );
    send_last_will_message(
        req.client_id.as_str(),
        cache_manager,
        client_pool,
        &data.last_will,
        &data.last_will_properties,
        message_storage_adapter.clone(),
    )
    .await?;
    Ok(SendLastWillMessageReply::default())
}
