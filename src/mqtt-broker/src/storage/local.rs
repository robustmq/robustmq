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

use common_base::error::ResultCommonError;
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::broker::{engine_prefix_list_by_broker, engine_save_by_broker},
};

use rocksdb_engine::keys::broker::{
    ban_log_key, ban_log_prefix_key, ban_log_prefix_key_by_tenant, slow_sub_log_key,
    slow_sub_log_prefix_key, slow_sub_log_prefix_key_by_tenant, system_event_key,
    system_event_prefix_key,
};

use crate::core::{
    error::MqttBrokerError, flapping_detect::BanLog, sub_slow::SlowSubscribeData,
    system_alarm::SystemAlarmEventMessage,
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
        let key = system_event_key(&alarm.name, alarm.create_time as i64);
        engine_save_by_broker(&self.rocksdb_engine_handler, &key, alarm)
    }

    pub async fn list_system_event(&self) -> Result<Vec<SystemAlarmEventMessage>, MqttBrokerError> {
        let prefix_key = system_event_prefix_key();
        let data = engine_prefix_list_by_broker::<SystemAlarmEventMessage>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub async fn save_ban_log(&self, log: BanLog) -> ResultCommonError {
        let key = ban_log_key(
            &log.tenant,
            &log.ban_type,
            &log.resource_name,
            log.create_time as i64,
        );
        engine_save_by_broker(&self.rocksdb_engine_handler, &key, log)
    }

    pub async fn list_ban_log(&self, tenant: Option<&str>) -> Result<Vec<BanLog>, MqttBrokerError> {
        let prefix_key = match tenant {
            Some(t) => ban_log_prefix_key_by_tenant(t),
            None => ban_log_prefix_key(),
        };
        let data =
            engine_prefix_list_by_broker::<BanLog>(&self.rocksdb_engine_handler, &prefix_key)?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub async fn save_slow_sub_log(&self, log: SlowSubscribeData) -> ResultCommonError {
        let key = slow_sub_log_key(&log.tenant, &log.client_id, &log.topic_name);
        engine_save_by_broker(&self.rocksdb_engine_handler, &key, log)
    }

    pub async fn list_slow_sub_log(
        &self,
        tenant: Option<&str>,
    ) -> Result<Vec<SlowSubscribeData>, MqttBrokerError> {
        let prefix_key = match tenant {
            Some(t) => slow_sub_log_prefix_key_by_tenant(t),
            None => slow_sub_log_prefix_key(),
        };
        let data = engine_prefix_list_by_broker::<SlowSubscribeData>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb_engine::test::test_rocksdb_instance;

    use crate::core::{
        flapping_detect::BanLog, sub_slow::SlowSubscribeData, system_alarm::SystemAlarmEventMessage,
    };

    #[tokio::test]
    async fn test_system_event_save_and_list() {
        let db = test_rocksdb_instance();
        let storage = LocalStorage::new(db);

        for i in 0..5u64 {
            let event = SystemAlarmEventMessage {
                name: format!("alarm_{}", i),
                message: format!("message_{}", i),
                create_time: 1000 + i,
                activated: true,
            };
            storage.save_system_event(event).await.unwrap();
        }

        let list = storage.list_system_event().await.unwrap();
        assert_eq!(list.len(), 5);
    }

    #[tokio::test]
    async fn test_ban_log_save_and_list() {
        let db = test_rocksdb_instance();
        let storage = LocalStorage::new(db);

        for i in 0..5u64 {
            let log = BanLog {
                tenant: "test_tenant".to_string(),
                ban_type: "client".to_string(),
                resource_name: format!("client_{}", i),
                ban_source: "flapping".to_string(),
                end_time: 9999,
                create_time: 1000 + i,
            };
            storage.save_ban_log(log).await.unwrap();
        }

        let all = storage.list_ban_log(None).await.unwrap();
        assert_eq!(all.len(), 5);

        let by_tenant = storage.list_ban_log(Some("test_tenant")).await.unwrap();
        assert_eq!(by_tenant.len(), 5);

        let empty = storage.list_ban_log(Some("other_tenant")).await.unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[tokio::test]
    async fn test_slow_sub_log_save_and_list() {
        let db = test_rocksdb_instance();
        let storage = LocalStorage::new(db);

        for i in 0..5u64 {
            let log = SlowSubscribeData {
                tenant: "test_tenant".to_string(),
                subscribe_name: format!("sub_{}", i),
                client_id: format!("client_{}", i),
                topic_name: format!("topic/{}", i),
                node_info: "127.0.0.1".to_string(),
                time_span: 100 + i,
                create_time: 1000 + i,
            };
            storage.save_slow_sub_log(log).await.unwrap();
        }

        let all = storage.list_slow_sub_log(None).await.unwrap();
        assert_eq!(all.len(), 5);

        let by_tenant = storage
            .list_slow_sub_log(Some("test_tenant"))
            .await
            .unwrap();
        assert_eq!(by_tenant.len(), 5);

        let empty = storage
            .list_slow_sub_log(Some("other_tenant"))
            .await
            .unwrap();
        assert_eq!(empty.len(), 0);
    }
}
