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

use crate::admin::query::{apply_filters, apply_pagination, apply_sorting, Queryable};
use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::security::AuthDriver;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use protocol::broker_mqtt::broker_mqtt_admin::{
    BlacklistRaw, CreateBlacklistRequest, DeleteBlacklistRequest, ListBlacklistRequest,
};
use std::sync::Arc;
use tonic::Request;

// List blacklists by request
pub async fn list_blacklist_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<ListBlacklistRequest>,
) -> Result<(Vec<BlacklistRaw>, usize), MqttBrokerError> {
    let req = request.into_inner();
    let blacklists = extract_blacklist(cache_manager, client_pool).await?;

    let filtered = apply_filters(blacklists, &req.options);
    let sorted = apply_sorting(filtered, &req.options);
    let pagination = apply_pagination(sorted, &req.options);

    Ok(pagination)
}

async fn extract_blacklist(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<Vec<BlacklistRaw>, MqttBrokerError> {
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.read_all_blacklist().await {
        Ok(data) => {
            let mut blacklists = Vec::new();
            for element in data {
                let blacklist_raw = BlacklistRaw::from(element);
                blacklists.push(blacklist_raw)
            }
            Ok(blacklists)
        }
        Err(e) => Err(e),
    }
}
// Delete blacklist entry
pub async fn delete_blacklist_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteBlacklistRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();
    let blacklist_type = match req.blacklist_type.as_str() {
        "ClientId" => MqttAclBlackListType::ClientId,
        "User" => MqttAclBlackListType::User,
        "Ip" => MqttAclBlackListType::Ip,
        "ClientIdMatch" => MqttAclBlackListType::ClientIdMatch,
        "UserMatch" => MqttAclBlackListType::UserMatch,
        "IPCIDR" => MqttAclBlackListType::IPCIDR,
        _ => {
            return Err(MqttBrokerError::CommonError(format!(
                "Failed BlackList Type: {}",
                req.blacklist_type
            )))
        }
    };

    let mqtt_blacklist = MqttAclBlackList {
        blacklist_type,
        resource_name: req.resource_name,
        end_time: 0,
        desc: "".to_string(),
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.delete_blacklist(mqtt_blacklist).await?;

    Ok(())
}

// Create new blacklist entry
pub async fn create_blacklist_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateBlacklistRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();
    let mqtt_blacklist = MqttAclBlackList::decode(&req.blacklist)
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.save_blacklist(mqtt_blacklist).await?;

    Ok(())
}

impl Queryable for BlacklistRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "blacklist_type" => Some(self.blacklist_type.to_string()),
            "resource_name" => Some(self.resource_name.clone()),
            "end_time" => Some(self.end_time.to_string()),
            "desc" => Some(self.desc.clone()),
            _ => None,
        }
    }
}
