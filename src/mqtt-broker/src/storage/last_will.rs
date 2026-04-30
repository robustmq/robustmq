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

use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

pub const LAST_WILL_MESSAGE_TOPIC: &str = "$last-will-message";

pub struct LastWillStorage {
    storage_driver_manager: Arc<StorageDriverManager>,
}

impl LastWillStorage {
    pub fn new(storage_driver_manager: Arc<StorageDriverManager>) -> Self {
        LastWillStorage {
            storage_driver_manager,
        }
    }

    pub async fn save_last_will_message(
        &self,
        tenant: &str,
        client_id: &str,
        last_will: &MqttLastWillData,
    ) -> ResultMqttBrokerError {
        let data = last_will.encode()?;
        let record = AdapterWriteRecord::new(LAST_WILL_MESSAGE_TOPIC, data).with_key(client_id);
        self.storage_driver_manager
            .write(tenant, LAST_WILL_MESSAGE_TOPIC, &[record])
            .await?;
        Ok(())
    }

    pub async fn get_last_will_message(
        &self,
        tenant: &str,
        client_id: &str,
    ) -> Result<Option<MqttLastWillData>, MqttBrokerError> {
        let records = self
            .storage_driver_manager
            .read_by_key(tenant, LAST_WILL_MESSAGE_TOPIC, client_id)
            .await?;
        if let Some(record) = records.into_iter().next() {
            let data = MqttLastWillData::decode(&record.data)?;
            return Ok(Some(data));
        }
        Ok(None)
    }

    pub async fn delete_last_will_message(
        &self,
        tenant: &str,
        client_id: &str,
    ) -> ResultMqttBrokerError {
        self.storage_driver_manager
            .delete_by_key(tenant, LAST_WILL_MESSAGE_TOPIC, client_id)
            .await?;
        Ok(())
    }
}
