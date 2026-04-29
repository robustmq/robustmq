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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::meta::mqtt::call::{
    placement_get_last_will_message, placement_save_last_will_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use protocol::meta::meta_service_mqtt::{GetLastWillMessageRequest, SaveLastWillMessageRequest};
use std::sync::Arc;

pub struct LastWillStorage {
    client_pool: Arc<ClientPool>,
}

impl LastWillStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        LastWillStorage { client_pool }
    }

    pub async fn save_last_will_message(
        &self,
        client_id: String,
        last_will_message: Vec<u8>,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = SaveLastWillMessageRequest {
            client_id,
            last_will_message,
        };
        placement_save_last_will_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn get_last_will_message(
        &self,
        client_id: String,
    ) -> Result<Option<MqttLastWillData>, CommonError> {
        let config = broker_config();
        let request = GetLastWillMessageRequest { client_id };
        let reply = placement_get_last_will_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        if reply.message.is_empty() {
            return Ok(None);
        }
        let data = MqttLastWillData::decode(&reply.message)?;
        Ok(Some(data))
    }
}
