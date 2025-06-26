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

use crate::admin::query::{apply_filters, apply_pagination, apply_sorting};
use crate::handler::error::MqttBrokerError;
use crate::security::AuthDriver;
use crate::{admin::query::Queryable, handler::cache::CacheManager};
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use protocol::broker_mqtt::broker_mqtt_admin::{
    AclRaw, CreateAclRequest, DeleteAclRequest, ListAclRequest,
};
use std::sync::Arc;
use tonic::Request;

// List all ACL entries
pub async fn list_acl_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<ListAclRequest>,
) -> Result<(Vec<AclRaw>, usize), MqttBrokerError> {
    let req = request.into_inner();
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    let data = auth_driver.read_all_acl().await?;

    let acls: Vec<AclRaw> = data.into_iter().map(AclRaw::from).collect();
    let filtered = apply_filters(acls, &req.options);
    let sorted = apply_sorting(filtered, &req.options);
    let pagination = apply_pagination(sorted, &req.options);

    Ok(pagination)
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

impl Queryable for AclRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "resource_type" => Some(self.resource_type.to_string()),
            "resource_name" => Some(self.resource_name.clone()),
            "topic" => Some(self.topic.clone()),
            "ip" => Some(self.ip.clone()),
            "action" => Some(self.action.to_string()),
            "permission" => Some(self.permission.to_string()),
            _ => None,
        }
    }
}
