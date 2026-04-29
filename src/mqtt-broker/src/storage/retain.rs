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
use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

pub const RETAIN_MESSAGE_TOPIC: &str = "$retain-message";

pub struct RetainStorage {
    storage_driver_manager: Arc<StorageDriverManager>,
}

impl RetainStorage {
    pub fn new(storage_driver_manager: Arc<StorageDriverManager>) -> Self {
        RetainStorage {
            storage_driver_manager,
        }
    }

    pub async fn set_retain_message(
        &self,
        tenant: &str,
        topic_name: &str,
        retain_message: &MQTTRetainMessage,
    ) -> ResultMqttBrokerError {
        let data = retain_message.encode()?;
        let record = AdapterWriteRecord::new(RETAIN_MESSAGE_TOPIC, data).with_key(topic_name);
        self.storage_driver_manager
            .write(tenant, RETAIN_MESSAGE_TOPIC, &[record])
            .await?;
        Ok(())
    }

    pub async fn delete_retain_message(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> ResultMqttBrokerError {
        self.storage_driver_manager
            .delete_by_key(tenant, RETAIN_MESSAGE_TOPIC, topic_name)
            .await?;
        Ok(())
    }

    pub async fn get_retain_message(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<Option<MQTTRetainMessage>, MqttBrokerError> {
        let records = self
            .storage_driver_manager
            .read_by_key(tenant, RETAIN_MESSAGE_TOPIC, topic_name)
            .await?;
        if let Some(record) = records.into_iter().next() {
            let message = MQTTRetainMessage::decode(&record.data)?;
            return Ok(Some(message));
        }
        Ok(None)
    }
}
