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

use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::security::AuthDriver;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use protocol::broker_mqtt::broker_mqtt_admin::{CreateAclRequest, DeleteAclRequest};
use std::sync::Arc;
use tonic::Request;

// List all ACL entries
pub async fn list_acl_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<Vec<Vec<u8>>, MqttBrokerError> {
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    let data = auth_driver.read_all_acl().await?;

    let mut acls_list = Vec::new();
    for ele in data {
        let acl = ele
            .encode()
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
        acls_list.push(acl);
    }

    Ok(acls_list)
}

// Create a new ACL entry
pub async fn create_acl_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateAclRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();

    let mqtt_acl =
        MqttAcl::decode(&req.acl).map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.save_acl(mqtt_acl).await?;

    Ok(())
}

// Delete an existing ACL entry
pub async fn delete_acl_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteAclRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();
    let mqtt_acl =
        MqttAcl::decode(&req.acl).map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.delete_acl(mqtt_acl).await?;

    Ok(())
}
