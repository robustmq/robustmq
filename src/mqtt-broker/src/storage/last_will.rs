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
use broker_core::inner_topic::LAST_WILL_MESSAGE_TOPIC;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
// The inner topic "$last-will-message" is a single broker-wide topic created under DEFAULT_TENANT.
// To isolate last-will messages across tenants and avoid key collisions between clients with the
// same client_id in different tenants, the storage key is composed as "{tenant}/{client_id}".
use metadata_struct::tenant::DEFAULT_TENANT;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

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
        let key = last_will_key(tenant, client_id);
        let data = last_will.encode()?;
        let record = AdapterWriteRecord::new(LAST_WILL_MESSAGE_TOPIC, data).with_key(&key);
        self.storage_driver_manager
            .write(DEFAULT_TENANT, LAST_WILL_MESSAGE_TOPIC, &[record])
            .await?;
        Ok(())
    }

    pub async fn get_last_will_message(
        &self,
        tenant: &str,
        client_id: &str,
    ) -> Result<Option<MqttLastWillData>, MqttBrokerError> {
        let key = last_will_key(tenant, client_id);
        let records = self
            .storage_driver_manager
            .read_by_key(DEFAULT_TENANT, LAST_WILL_MESSAGE_TOPIC, &key)
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
        let key = last_will_key(tenant, client_id);
        self.storage_driver_manager
            .delete_by_key(DEFAULT_TENANT, LAST_WILL_MESSAGE_TOPIC, &key)
            .await?;
        Ok(())
    }
}

fn last_will_key(tenant: &str, client_id: &str) -> String {
    format!("{}/{}", tenant, client_id)
}
