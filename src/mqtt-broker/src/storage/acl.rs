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

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::placement::mqtt::call::{create_acl, delete_acl, list_acl};
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use protocol::placement_center::placement_center_mqtt::{
    CreateAclRequest, DeleteAclRequest, ListAclRequest,
};

use crate::handler::error::MqttBrokerError;

pub struct AclStorage {
    client_pool: Arc<ClientPool>,
}

impl AclStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        AclStorage { client_pool }
    }

    pub async fn list_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ListAclRequest {
            cluster_name: config.cluster_name.clone(),
        };
        let reply = list_acl(&self.client_pool, &config.placement_center, request).await?;
        let mut list = Vec::new();
        for raw in reply.acls {
            list.push(serde_json::from_slice::<MqttAcl>(raw.as_slice())?);
        }
        Ok(list)
    }

    pub async fn save_acl(&self, acl: MqttAcl) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();

        let value = acl.encode()?;
        let request = CreateAclRequest {
            cluster_name: config.cluster_name.clone(),
            acl: value,
        };
        create_acl(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn delete_acl(&self, acl: MqttAcl) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let value = acl.encode()?;
        let request = DeleteAclRequest {
            cluster_name: config.cluster_name.clone(),
            acl: value,
        };
        delete_acl(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }
}
