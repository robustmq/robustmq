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

use crate::core::cache::MQTTCacheManager;
use crate::core::error::MqttBrokerError;
use crate::core::last_will::send_last_will_message;
use crate::subscribe::manager::SubscribeManager;
use broker_core::tool::wait_cluster_running;
use common_metrics::mqtt::session::record_mqtt_session_deleted;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use protocol::broker::broker_mqtt::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tracing::debug;

pub async fn delete_session_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    req: &DeleteSessionRequest,
) -> Result<DeleteSessionReply, MqttBrokerError> {
    debug!(
        "Received request from Meta service to delete expired Session. clientId count: {:?}",
        req.client_id.len()
    );
    wait_cluster_running(&cache_manager.broker_cache).await;

    if req.client_id.is_empty() {
        return Err(MqttBrokerError::ClientIDIsEmpty);
    }

    for client_id in req.client_id.iter() {
        subscribe_manager.remove_by_client_id(client_id);
        cache_manager.remove_session(client_id);
    }
    record_mqtt_session_deleted();
    Ok(DeleteSessionReply::default())
}

pub async fn send_last_will_message_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    req: &SendLastWillMessageRequest,
) -> Result<SendLastWillMessageReply, MqttBrokerError> {
    let data = match MqttLastWillData::decode(&req.last_will_message) {
        Ok(data) => data,
        Err(e) => {
            return Err(MqttBrokerError::CommonError(e.to_string()));
        }
    };

    wait_cluster_running(&cache_manager.broker_cache).await;
    debug!(
        "Received will message from meta service, source client id: {},data:{:?}",
        req.client_id, data.client_id
    );
    if let Err(e) = send_last_will_message(
        req.client_id.as_str(),
        cache_manager,
        client_pool,
        &data.last_will,
        &data.last_will_properties,
        storage_driver_manager,
    )
    .await
    {
        debug!("send_last_will_message:{}", e);
    }
    Ok(SendLastWillMessageReply::default())
}
