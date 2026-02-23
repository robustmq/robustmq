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
use crate::system_topic::sysmon::st_report_system_alarm_alert;
use crate::{core::cache::MQTTCacheManager, core::tool::ResultMqttBrokerError};
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use system_info::{process_cpu_usage, process_memory_usage};
use tokio::sync::broadcast;
use tracing::info;

#[allow(clippy::enum_variant_names)]
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
    pub name: String,
    pub message: String,
    /// JSON field name is "activate_at" to align with EMQX system topic format.
    /// "create_time" is kept as alias for backward-compatible deserialization from storage.
    #[serde(rename = "activate_at", alias = "create_time")]
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
        let config = broker_config();
        if !config.mqtt_system_monitor.enable {
            return Ok(());
        }

        let record_func = async || -> ResultCommonError {
            let mqtt_conf = broker_config();
            let cpu_usage = process_cpu_usage().await;

            self.try_send_a_new_system_event(
                AlarmType::HighCpuUsage,
                cpu_usage,
                mqtt_conf.mqtt_system_monitor.os_cpu_high_watermark,
            )
            .await?;

            let memory_usage = process_memory_usage();
            self.try_send_a_new_system_event(
                AlarmType::HighMemoryUsage,
                memory_usage,
                mqtt_conf.mqtt_system_monitor.os_memory_high_watermark,
            )
            .await?;
            Ok(())
        };

        info!("System alarm thread start successfully");
        loop_select_ticket(record_func, 60000, &stop_send).await;
        Ok(())
    }

    async fn try_send_a_new_system_event(
        &self,
        alarm_type: AlarmType,
        current_usage: f32,
        config_usage: f32,
    ) -> ResultCommonError {
        if current_usage > config_usage {
            let message = SystemAlarmEventMessage {
                name: alarm_type.to_string(),
                message: format!("{alarm_type} is {current_usage}%, but config is {config_usage}%"),
                create_time: now_second(),
                activated: true,
            };
            st_report_system_alarm_alert(
                &self.client_pool,
                &self.metadata_cache,
                &self.storage_driver_manager,
                &message,
            )
            .await?;
            let log_storage = LocalStorage::new(self.rocksdb_engine_handler.clone());
            log_storage.save_system_event(message).await?;
        }
        Ok(())
    }
}
