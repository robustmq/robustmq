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
use crate::system_topic::sysmon::st_report_system_alarm_event;
use crate::{common::types::ResultMqttBrokerError, handler::cache::MQTTCacheManager};
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::storage::ArcStorageAdapter;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tokio::sync::broadcast;
use tokio::time::sleep;
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
    pub create_time: u64,
}

pub struct SystemAlarm {
    client_pool: Arc<ClientPool>,
    metadata_cache: Arc<MQTTCacheManager>,
    message_storage_adapter: ArcStorageAdapter,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_send: broadcast::Sender<bool>,
}

impl SystemAlarm {
    pub fn new(
        client_pool: Arc<ClientPool>,
        metadata_cache: Arc<MQTTCacheManager>,
        message_storage_adapter: ArcStorageAdapter,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        SystemAlarm {
            client_pool,
            metadata_cache,
            message_storage_adapter,
            rocksdb_engine_handler,
            stop_send,
        }
    }

    pub async fn start(&self) -> ResultMqttBrokerError {
        let config = broker_config();
        if !config.mqtt_system_monitor.enable {
            return Ok(());
        }

        let record_func = async || -> ResultCommonError {
            let mqtt_conf = broker_config();
            let cpu_usage = get_process_every_cpu_usage().await;

            self.try_send_a_new_system_event(
                AlarmType::HighCpuUsage,
                cpu_usage,
                mqtt_conf.mqtt_system_monitor.os_cpu_high_watermark,
            )
            .await?;

            let memory_usage = get_process_memory_usage();
            self.try_send_a_new_system_event(
                AlarmType::HighMemoryUsage,
                memory_usage,
                mqtt_conf.mqtt_system_monitor.os_memory_high_watermark,
            )
            .await?;
            Ok(())
        };

        info!("System alarm thread start successfully");
        loop_select_ticket(record_func, 60, &self.stop_send).await;
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
            };
            st_report_system_alarm_event(
                &self.client_pool,
                &self.metadata_cache,
                &self.message_storage_adapter,
                &message,
            )
            .await?;
            let log_storage = LocalStorage::new(self.rocksdb_engine_handler.clone());
            log_storage.save_system_event(message).await?;
        }
        Ok(())
    }
}

// Get CPU usage percentage of the current process
pub async fn get_process_every_cpu_usage() -> f32 {
    let mut system = System::new_all();
    let pid = Pid::from(std::process::id() as usize);

    system.refresh_all();

    sleep(Duration::from_millis(1000)).await;

    system.refresh_all();

    // Get current process ID
    if let Some(process) = system.process(pid) {
        // Get latest CPU usage
        let cpu_usage = process.cpu_usage();
        let cpu_count = system.cpus().len() as f32;

        // Calculate average CPU usage per core
        return cpu_usage / cpu_count;
    }

    // Return 0 if failed to get information
    0.0
}

// Get memory usage percentage of the current process
pub fn get_process_memory_usage() -> f32 {
    let mut system = System::new_all();
    system.refresh_all();

    let total_memory = system.total_memory();
    if total_memory == 0 {
        return 0.0;
    }

    // Get current process ID
    let pid = Pid::from(std::process::id() as usize);

    if let Some(process) = system.process(pid) {
        // Get process memory usage (in bytes)
        let used_memory = process.memory();

        // Calculate memory usage percentage
        return (used_memory as f32 / total_memory as f32) * 100.0;
    }

    0.0
}
