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

 use grpc_clients::placement::mqtt::call::list_blacklist;
 use grpc_clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use protocol::placement_center::generate::mqtt::ListBlacklistRequest;

pub struct BlackListStorage {
    client_poll: Arc<ClientPool>,
}

impl BlackListStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        BlackListStorage { client_poll }
    }

    pub async fn list_blacklist(&self) -> Result<Vec<MqttAclBlackList>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListBlacklistRequest {
            cluster_name: config.cluster_name.clone(),
        };
        match list_blacklist(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let mut list = Vec::new();
                for raw in reply.blacklists {
                    list.push(serde_json::from_slice::<MqttAclBlackList>(raw.as_slice())?);
                }
                Ok(list)
            }
            Err(e) => Err(e),
        }
    }
}
