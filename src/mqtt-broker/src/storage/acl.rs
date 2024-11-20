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

use std::sync::Arc;

use bincode::serialize;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use grpc_clients::placement::mqtt::call::{placement_create_acl, placement_delete_acl, placement_list_acl};
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use protocol::placement_center::placement_center_mqtt::{
    CreateAclRequest, DeleteAclRequest, ListAclRequest,
};

pub struct AclStorage {
    client_pool: Arc<ClientPool>,
}

impl AclStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        AclStorage { client_pool }
    }

    pub async fn list_acl(&self) -> Result<Vec<MqttAcl>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListAclRequest {
            cluster_name: config.cluster_name.clone(),
        };
        match placement_list_acl(
            self.client_pool.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let mut list = Vec::new();
                for raw in reply.acls {
                    list.push(serde_json::from_slice::<MqttAcl>(raw.as_slice())?);
                }
                Ok(list)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn save_acl(&self, acl: MqttAcl) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let value = serialize(&acl).map_err(|e| CommonError::CommmonError(e.to_string()))?;
        let request = CreateAclRequest {
            cluster_name: config.cluster_name.clone(),
            acl: value,
        };

        match placement_create_acl(
            self.client_pool.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_acl(&self, acl: MqttAcl) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let value = serialize(&acl).map_err(|e| CommonError::CommmonError(e.to_string()))?;
        let request = DeleteAclRequest {
            cluster_name: config.cluster_name.clone(),
            acl: value,
        };

        match placement_delete_acl(
            self.client_pool.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
