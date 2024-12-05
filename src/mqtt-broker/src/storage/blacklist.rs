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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::placement::mqtt::call::{create_blacklist, delete_blacklist, list_blacklist};
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use protocol::placement_center::placement_center_mqtt::{
    CreateBlacklistRequest, DeleteBlacklistRequest, ListBlacklistRequest,
};

use crate::handler::error::MqttBrokerError;

pub struct BlackListStorage {
    client_pool: Arc<ClientPool>,
}

impl BlackListStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        BlackListStorage { client_pool }
    }

    pub async fn list_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ListBlacklistRequest {
            cluster_name: config.cluster_name.clone(),
        };
        let reply =
            list_blacklist(self.client_pool.clone(), &config.placement_center, request).await?;
        let mut list = Vec::new();
        for raw in reply.blacklists {
            list.push(serde_json::from_slice::<MqttAclBlackList>(raw.as_slice())?);
        }
        Ok(list)
    }

    pub async fn save_blacklist(&self, blacklist: MqttAclBlackList) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = CreateBlacklistRequest {
            cluster_name: config.cluster_name.clone(),
            blacklist: blacklist.encode()?,
        };
        create_blacklist(self.client_pool.clone(), &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn delete_blacklist(
        &self,
        blacklist: MqttAclBlackList,
    ) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = DeleteBlacklistRequest {
            cluster_name: config.cluster_name.clone(),
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name,
        };
        delete_blacklist(self.client_pool.clone(), &config.placement_center, request).await?;
        Ok(())
    }
}
