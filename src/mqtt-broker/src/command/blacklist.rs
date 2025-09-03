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

use crate::handler::error::MqttBrokerError;
use crate::handler::{cache::MQTTCacheManager, flapping_detect::enable_flapping_detect};
use crate::security::AuthDriver;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use protocol::broker::broker_mqtt_admin::{
    BlacklistRaw, CreateBlacklistReply, CreateBlacklistRequest, DeleteBlacklistReply,
    DeleteBlacklistRequest, EnableFlappingDetectReply, EnableFlappingDetectRequest,
    FlappingDetectRaw, ListBlacklistReply, ListBlacklistRequest, ListFlappingDetectReply,
    ListFlappingDetectRequest,
};
use std::sync::Arc;

// List blacklists by request
pub async fn list_blacklist_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    _: &ListBlacklistRequest,
) -> Result<ListBlacklistReply, MqttBrokerError> {
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    let data = auth_driver.read_all_blacklist().await?;
    let mut blacklists = Vec::new();
    for element in data {
        let blacklist_raw = BlacklistRaw::from(element);
        blacklists.push(blacklist_raw)
    }
    Ok(ListBlacklistReply { blacklists })
}

// Delete blacklist entry
pub async fn delete_blacklist_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    request: &DeleteBlacklistRequest,
) -> Result<DeleteBlacklistReply, MqttBrokerError> {
    let blacklist_type = match request.blacklist_type.as_str() {
        "ClientId" => MqttAclBlackListType::ClientId,
        "User" => MqttAclBlackListType::User,
        "Ip" => MqttAclBlackListType::Ip,
        "ClientIdMatch" => MqttAclBlackListType::ClientIdMatch,
        "UserMatch" => MqttAclBlackListType::UserMatch,
        "IPCIDR" => MqttAclBlackListType::IPCIDR,
        _ => {
            return Err(MqttBrokerError::CommonError(format!(
                "Failed BlackList Type: {}",
                request.blacklist_type
            )))
        }
    };

    let mqtt_blacklist = MqttAclBlackList {
        blacklist_type,
        resource_name: request.resource_name.clone(),
        end_time: 0,
        desc: "".to_string(),
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.delete_blacklist(mqtt_blacklist).await?;

    Ok(DeleteBlacklistReply {})
}

// Create new blacklist entry
pub async fn create_blacklist_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    request: &CreateBlacklistRequest,
) -> Result<CreateBlacklistReply, MqttBrokerError> {
    let mqtt_blacklist = MqttAclBlackList::decode(&request.blacklist)
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.save_blacklist(mqtt_blacklist).await?;

    Ok(CreateBlacklistReply {})
}

pub async fn enable_flapping_detect_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    request: &EnableFlappingDetectRequest,
) -> Result<EnableFlappingDetectReply, MqttBrokerError> {
    match enable_flapping_detect(client_pool, cache_manager, *request).await {
        Ok(_) => Ok(EnableFlappingDetectReply {
            is_enable: request.is_enable,
        }),
        Err(e) => Err(e),
    }
}

pub async fn list_flapping_detect_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    _request: &ListFlappingDetectRequest,
) -> Result<ListFlappingDetectReply, MqttBrokerError> {
    let flapping_detects = cache_manager
        .acl_metadata
        .flapping_detect_map
        .iter()
        .map(|entry| {
            let flapping_detect = entry.value();
            FlappingDetectRaw {
                client_id: flapping_detect.client_id.clone(),
                before_last_windows_connections: flapping_detect.before_last_window_connections,
                first_request_time: flapping_detect.first_request_time,
            }
        })
        .collect();

    Ok(ListFlappingDetectReply {
        flapping_detect_raw: flapping_detects,
    })
}
