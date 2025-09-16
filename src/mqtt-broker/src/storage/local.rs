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

use broker_core::{
    engine::{engine_prefix_list_by_broker, engine_save_by_broker},
    rocksdb::RocksDBEngine,
};
use common_base::error::ResultCommonError;

use crate::{
    handler::{error::MqttBrokerError, system_alarm::SystemAlarmEventMessage},
    storage::keys::{system_event_key, system_event_prefix_key},
};

pub struct LocalStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl LocalStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        LocalStorage {
            rocksdb_engine_handler,
        }
    }

    pub async fn save_system_event(&self, alarm: SystemAlarmEventMessage) -> ResultCommonError {
        let key = system_event_key(&alarm);
        engine_save_by_broker(self.rocksdb_engine_handler.clone(), key, alarm)
    }

    pub async fn list_system_event(&self) -> Result<Vec<SystemAlarmEventMessage>, MqttBrokerError> {
        let prefix_key = system_event_prefix_key();
        let mut results = Vec::new();
        for raw in engine_prefix_list_by_broker(self.rocksdb_engine_handler.clone(), prefix_key)? {
            if let Ok(data) = serde_json::from_str::<SystemAlarmEventMessage>(&raw.data) {
                results.push(data);
            }
        }
        Ok(results)
    }
}
