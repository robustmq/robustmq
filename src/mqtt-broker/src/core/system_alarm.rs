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

use crate::storage::local::LocalStorage;
use crate::system_topic::try_report_system_data;
use crate::{core::cache::MQTTCacheManager, core::tool::ResultMqttBrokerError};
use common_base::error::ResultCommonError;
use common_base::tools::{get_local_ip, loop_select_ticket, now_second};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use system_info::{process_cpu_usage, process_memory_usage};
use tokio::sync::broadcast;
use tracing::warn;

// System alarm
pub const SYSTEM_TOPIC_BROKERS_ALARMS_ALERT: &str = "$SYS/brokers/alarms/alert";

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum AlarmType {
    HighCpuUsage,
    HighMemoryUsage,
}

impl fmt::Display for AlarmType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlarmType::HighCpuUsage => write!(f, "HighCpuUsage"),
            AlarmType::HighMemoryUsage => write!(f, "HighMemoryUsage"),
        }
    }
}

// sysmon topic
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SystemAlarmEventMessage {
    #[serde(default)]
    pub node_id: u64,
    #[serde(default)]
    pub node_ip: String,
    pub name: String,
    pub message: String,
    /// JSON field name is "activate_at" to align with EMQX system topic format.
    /// "create_time" is kept as alias for backward-compatible deserialization from storage.
    #[serde(rename = "activate_at", alias = "create_time")]
    pub create_time: u64,
    pub activated: bool,
}

#[derive(Serialize, Clone)]
struct SystemAlarmPayload {
    pub name: String,
    pub message: String,
    #[serde(rename = "activate_at")]
    pub create_time: u64,
    pub activated: bool,
}

pub struct SystemAlarm {
    client_pool: Arc<ClientPool>,
    metadata_cache: Arc<MQTTCacheManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SystemAlarm {
    pub fn new(
        client_pool: Arc<ClientPool>,
        metadata_cache: Arc<MQTTCacheManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        SystemAlarm {
            client_pool,
            metadata_cache,
            storage_driver_manager,
            rocksdb_engine_handler,
        }
    }

    pub async fn start(&self, stop_send: broadcast::Sender<bool>) -> ResultMqttBrokerError {
        let record_func = async || -> ResultCommonError {
            let mqtt_conf = broker_config();
            let cpu_usage = process_cpu_usage().await;

            if let Err(e) = self
                .report_if_alarm_active(
                    AlarmType::HighCpuUsage,
                    cpu_usage,
                    mqtt_conf.mqtt_runtime.system_monitor.os_cpu_high_watermark,
                )
                .await
            {
                warn!("Failed to report system CPU alarm; next check will retry: {e}");
            }

            let memory_usage = process_memory_usage();
            if let Err(e) = self
                .report_if_alarm_active(
                    AlarmType::HighMemoryUsage,
                    memory_usage,
                    mqtt_conf
                        .mqtt_runtime
                        .system_monitor
                        .os_memory_high_watermark,
                )
                .await
            {
                warn!("Failed to report system memory alarm; next check will retry: {e}");
            }
            Ok(())
        };

        loop_select_ticket(record_func, 60000, &stop_send).await;
        Ok(())
    }

    async fn report_if_alarm_active(
        &self,
        alarm_type: AlarmType,
        current_usage: f32,
        config_usage: f32,
    ) -> ResultCommonError {
        let activated = current_usage > config_usage;
        if activated {
            let message_text =
                format!("{alarm_type} is {current_usage}%, but config is {config_usage}%");
            let message = SystemAlarmEventMessage {
                node_id: broker_config().broker_id,
                node_ip: broker_config()
                    .broker_ip
                    .clone()
                    .unwrap_or_else(get_local_ip),
                name: alarm_type.to_string(),
                message: message_text,
                create_time: now_second(),
                activated,
            };

            let payload = SystemAlarmPayload {
                name: message.name.clone(),
                message: message.message.clone(),
                create_time: message.create_time,
                activated: message.activated,
            };
            if let Err(e) = try_report_system_data(
                &self.client_pool,
                &self.metadata_cache,
                &self.storage_driver_manager,
                SYSTEM_TOPIC_BROKERS_ALARMS_ALERT,
                || async move { payload },
            )
            .await
            {
                // Keep the local history record even when publication fails. The next
                // polling interval will retry the active system-topic report.
                warn!(
                    topic = SYSTEM_TOPIC_BROKERS_ALARMS_ALERT,
                    "Failed to publish active system alarm; retaining history record: {e}"
                );
            }
            let log_storage = LocalStorage::new(self.rocksdb_engine_handler.clone());
            log_storage.save_system_event(message).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SystemAlarmPayload;

    #[test]
    fn mqtt_alarm_payload_keeps_node_identity_in_outer_envelope() {
        let payload = SystemAlarmPayload {
            name: "HighCpuUsage".to_string(),
            message: "HighCpuUsage is 85%, but config is 70%".to_string(),
            create_time: 1000,
            activated: true,
        };
        let json = serde_json::to_value(payload).unwrap();

        assert!(json.get("node_id").is_none());
        assert!(json.get("node_ip").is_none());
        assert_eq!(json["activate_at"], 1000);
    }
}
