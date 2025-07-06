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

use crate::handler::cache::CacheManager;
use crate::observability::system_topic::{replace_topic_name, write_topic_data};

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::storage::StorageAdapter;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tokio::time::sleep;
use tracing::error;

// sysmon topic
pub(crate) const SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE: &str =
    "$SYS/brokers/${node}/alarms/activate";
pub(crate) const SYSTEM_TOPIC_BROKERS_ALARMS_DEACTIVATE: &str =
    "$SYS/brokers/${node}/alarms/deactivate";

#[allow(clippy::enum_variant_names)]
enum AlarmType {
    HighCpuUsage,
    LowCpuUsage,
    MemoryUsage,
}

impl AlarmType {
    fn as_str(&self) -> &str {
        match self {
            AlarmType::HighCpuUsage => "HighCpuUsage",
            AlarmType::LowCpuUsage => "LowCpuUsage",
            AlarmType::MemoryUsage => "MemoryUsage",
        }
    }
}

impl fmt::Display for AlarmType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlarmType::HighCpuUsage => write!(f, "HighCpuUsage"),
            AlarmType::LowCpuUsage => write!(f, "LowCpuUsage"),
            AlarmType::MemoryUsage => write!(f, "MemoryUsage"),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SystemAlarmEventMessage {
    pub name: String,
    pub message: String,
    pub activate_at: i64,
    pub activated: bool,
}

pub async fn st_check_system_alarm<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    let mqtt_conf = broker_mqtt_conf();
    let cpu_usage =
        get_process_every_cpu_usage(mqtt_conf.system_monitor.os_cpu_check_interval_ms).await;

    is_send_a_new_system_event(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        AlarmType::HighCpuUsage,
        cpu_usage,
        mqtt_conf.system_monitor.os_cpu_high_watermark,
    )
    .await;

    is_send_a_new_system_event(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        AlarmType::LowCpuUsage,
        cpu_usage,
        mqtt_conf.system_monitor.os_cpu_low_watermark,
    )
    .await;

    let memory_usage = get_process_memory_usage();
    is_send_a_new_system_event(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        AlarmType::MemoryUsage,
        memory_usage,
        mqtt_conf.system_monitor.os_memory_high_watermark,
    )
    .await;
}

async fn is_send_a_new_system_event<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
    alarm_type: AlarmType,
    current_usage: f32,
    config_usage: f32,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    let mut message = SystemAlarmEventMessage {
        name: alarm_type.to_string(),
        message: format!("{alarm_type} is {current_usage}%, but config is {config_usage}%"),
        activate_at: chrono::Utc::now().timestamp(),
        activated: false,
    };

    if AlarmType::LowCpuUsage.as_str() == alarm_type.as_str() {
        // For LowCpuUsage, we want to activate the alarm when the usage is below the threshold
        message.activated = current_usage < config_usage;
    } else {
        // For HighCpuUsage and MemoryUsage, we want to activate the alarm when the usage exceeds the threshold
        message.activated = current_usage > config_usage;
    }

    let is_send_message = match metadata_cache.get_alarm_event(alarm_type.as_str()) {
        None => true,
        Some(alarm_message) => alarm_message.activated != message.activated,
    };

    if is_send_message {
        st_report_system_alarm_event(
            client_pool,
            metadata_cache,
            message_storage_adapter,
            &message,
        )
        .await;
    }

    metadata_cache.add_alarm_event(alarm_type.to_string(), message);
}

pub async fn st_report_system_alarm_event<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
    message_event: &SystemAlarmEventMessage,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    match serde_json::to_string(message_event) {
        Ok(data) => {
            let mut topic_name =
                replace_topic_name(SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE.to_string());
            if !message_event.activated {
                topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_ALARMS_DEACTIVATE.to_string());
            }
            if let Some(record) = MqttMessage::build_system_topic_message(topic_name.clone(), data)
            {
                write_topic_data(
                    message_storage_adapter,
                    metadata_cache,
                    client_pool,
                    topic_name,
                    record,
                )
                .await;
            };
        }
        Err(e) => {
            error!("{}", e.to_string());
        }
    }
}

// Get CPU usage percentage of the current process
pub async fn get_process_every_cpu_usage(check_interval: u64) -> f32 {
    let mut system = System::new_all();
    let pid = Pid::from(std::process::id() as usize);

    system.refresh_all();

    sleep(Duration::from_millis(check_interval)).await;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::message::cluster_name;

    use common_base::tools::unique_id;
    use common_config::mqtt::init_broker_mqtt_conf_by_path;
    use metadata_struct::adapter::read_config::ReadConfig;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use storage_adapter::memory::MemoryStorageAdapter;

    #[tokio::test]
    async fn test_alarm_type_to_string() {
        let high_cpu_alarm = AlarmType::HighCpuUsage;
        let low_cpu_alarm = AlarmType::LowCpuUsage;
        let memory_alarm = AlarmType::MemoryUsage;

        assert_eq!(high_cpu_alarm.to_string(), "HighCpuUsage");
        assert_eq!(low_cpu_alarm.to_string(), "LowCpuUsage");
        assert_eq!(memory_alarm.to_string(), "MemoryUsage");
    }

    #[tokio::test]
    async fn test_get_process_cpu_usage() {
        let cpu_usage = get_process_every_cpu_usage(15000).await;
        assert!(cpu_usage >= 0.0);
        assert!(cpu_usage <= 100.0);
    }

    #[tokio::test]
    async fn test_get_process_memory_usage() {
        let memory_usage = get_process_memory_usage();

        assert!(memory_usage >= 0.0);
        assert!(memory_usage <= 100.0);
    }

    #[tokio::test]
    async fn test_report_system_alarm_event() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let metadata_cache = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());

        let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE.to_string());
        let mqtt_topic = MQTTTopic::new(unique_id(), cluster_name(), topic_name.clone());
        metadata_cache.add_topic(&topic_name, &mqtt_topic);

        let message = SystemAlarmEventMessage {
            name: "High CPU Usage".to_string(),
            message: "CPU usage exceeds 80%".to_string(),
            activate_at: chrono::Utc::now().timestamp(),
            activated: true,
        };

        st_report_system_alarm_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            &message,
        )
        .await;

        let mqtt_topic = metadata_cache.get_topic_by_name(&topic_name).unwrap();

        let read_config = ReadConfig {
            max_record_num: 1,
            max_size: 1024 * 1024 * 1024,
        };

        let record = message_storage_adapter
            .read_by_offset(
                cluster_name().to_owned(),
                mqtt_topic.topic_id.clone(),
                0,
                read_config,
            )
            .await
            .unwrap();

        let expected_data = serde_json::to_string(&message).unwrap();

        let except_message =
            MqttMessage::build_system_topic_message(topic_name.clone(), expected_data).unwrap();

        assert_eq!(record[0].data, except_message.data);
        assert_eq!(record[0].crc_num, except_message.crc_num);
    }

    #[tokio::test]
    async fn test_is_send_a_new_system_event_current_usage_gt_config_usage() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let metadata_cache = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());

        let current_cpu_usage = 90.0; // Simulate current CPU usage
        let config_cpu_usage = broker_mqtt_conf().system_monitor.os_cpu_high_watermark;

        let except_key = AlarmType::HighCpuUsage;
        let except_value = SystemAlarmEventMessage {
            name: except_key.to_string(),
            message: format!(
                "{except_key} is {current_cpu_usage}%, but config is {config_cpu_usage}%"
            ),
            activate_at: chrono::Utc::now().timestamp(),
            activated: true,
        };

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::HighCpuUsage,
            current_cpu_usage,
            config_cpu_usage,
        )
        .await;

        let need_check_message = metadata_cache.get_alarm_event(except_key.as_str()).unwrap();

        assert_eq!(need_check_message.name, except_value.name);
        assert_eq!(need_check_message.message, except_value.message);
        assert_eq!(need_check_message.activated, except_value.activated);
    }

    #[tokio::test]
    async fn test_is_send_a_new_system_event_current_usage_le_config_usage() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let metadata_cache = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());

        let current_cpu_usage = 50.0; // Simulate current CPU usage
        let config_cpu_usage = broker_mqtt_conf().system_monitor.os_cpu_high_watermark;

        let except_key = AlarmType::HighCpuUsage;
        let except_value = SystemAlarmEventMessage {
            name: except_key.to_string(),
            message: format!(
                "{except_key} is {current_cpu_usage}%, but config is {config_cpu_usage}%"
            ),
            activate_at: chrono::Utc::now().timestamp(),
            activated: false,
        };

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::HighCpuUsage,
            current_cpu_usage,
            config_cpu_usage,
        )
        .await;

        let need_check_message = metadata_cache.get_alarm_event(except_key.as_str()).unwrap();

        assert_eq!(need_check_message.name, except_value.name);
        assert_eq!(need_check_message.message, except_value.message);
        assert_eq!(need_check_message.activated, except_value.activated);
    }

    #[tokio::test]
    async fn test_is_send_a_new_system_event_metadata_exist_value_but_the_value_is_different() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let metadata_cache = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());

        let current_cpu_usage = 90.0; // Simulate current CPU usage
        let config_cpu_usage = broker_mqtt_conf().system_monitor.os_cpu_high_watermark;

        let except_key = AlarmType::HighCpuUsage;
        let except_value = SystemAlarmEventMessage {
            name: except_key.to_string(),
            message: format!(
                "{except_key} is {current_cpu_usage}%, but config is {config_cpu_usage}%"
            ),
            activate_at: chrono::Utc::now().timestamp(),
            activated: true,
        };

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::HighCpuUsage,
            current_cpu_usage,
            config_cpu_usage,
        )
        .await;

        let first_check_message = metadata_cache.get_alarm_event(except_key.as_str()).unwrap();

        assert_eq!(first_check_message.name, except_value.name);
        assert_eq!(first_check_message.message, except_value.message);
        assert_eq!(first_check_message.activated, except_value.activated);

        let current_cpu_usage = 50.0;
        let except_value = SystemAlarmEventMessage {
            name: except_key.to_string(),
            message: format!(
                "{except_key} is {current_cpu_usage}%, but config is {config_cpu_usage}%"
            ),
            activate_at: chrono::Utc::now().timestamp(),
            activated: false,
        };

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::HighCpuUsage,
            current_cpu_usage,
            config_cpu_usage,
        )
        .await;

        let twice_check_message = metadata_cache.get_alarm_event(except_key.as_str()).unwrap();
        assert_eq!(twice_check_message.name, except_value.name);
        assert_eq!(twice_check_message.message, except_value.message);
        assert_eq!(twice_check_message.activated, except_value.activated);
    }

    #[tokio::test]
    async fn test_is_send_a_new_system_event_metadata_exist_value_and_the_value_is_same() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let metadata_cache = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());

        let current_cpu_usage = 90.0; // Simulate current CPU usage
        let config_cpu_usage = broker_mqtt_conf().system_monitor.os_cpu_high_watermark;

        let except_key = AlarmType::HighCpuUsage;
        let except_value = SystemAlarmEventMessage {
            name: except_key.to_string(),
            message: format!(
                "{except_key} is {current_cpu_usage}%, but config is {config_cpu_usage}%"
            ),
            activate_at: chrono::Utc::now().timestamp(),
            activated: true,
        };

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::HighCpuUsage,
            current_cpu_usage,
            config_cpu_usage,
        )
        .await;

        let first_check_message = metadata_cache.get_alarm_event(except_key.as_str()).unwrap();

        assert_eq!(first_check_message.name, except_value.name);
        assert_eq!(first_check_message.message, except_value.message);
        assert_eq!(first_check_message.activated, except_value.activated);

        let current_cpu_usage = 90.0;

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::HighCpuUsage,
            current_cpu_usage,
            config_cpu_usage,
        )
        .await;

        let twice_check_message = metadata_cache.get_alarm_event(except_key.as_str()).unwrap();
        assert_eq!(twice_check_message.name, except_value.name);
        assert_eq!(twice_check_message.message, except_value.message);
        assert_eq!(twice_check_message.activated, except_value.activated);
    }

    #[tokio::test]
    async fn test_is_send_a_new_system_event_metadata_param_is_different() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let metadata_cache = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());

        let current_cpu_usage = 90.0; // Simulate current CPU usage
        let config_cpu_usage = broker_mqtt_conf().system_monitor.os_cpu_high_watermark;

        let except_cpu_key = AlarmType::HighCpuUsage;
        let except_memory_value = SystemAlarmEventMessage {
            name: except_cpu_key.to_string(),
            message: format!(
                "{except_cpu_key} is {current_cpu_usage}%, but config is {config_cpu_usage}%"
            ),
            activate_at: chrono::Utc::now().timestamp(),
            activated: true,
        };

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::HighCpuUsage,
            current_cpu_usage,
            config_cpu_usage,
        )
        .await;

        let cpu_check_message = metadata_cache
            .get_alarm_event(except_cpu_key.as_str())
            .unwrap();

        assert_eq!(cpu_check_message.name, except_memory_value.name);
        assert_eq!(cpu_check_message.message, except_memory_value.message);
        assert_eq!(cpu_check_message.activated, except_memory_value.activated);

        let current_memory_usage = 95.0; // Simulate current CPU usage
        let config_memory_usage = broker_mqtt_conf().system_monitor.os_memory_high_watermark;

        let except_memory_key = AlarmType::MemoryUsage;
        let except_memory_value = SystemAlarmEventMessage {
            name: except_memory_key.to_string(),
            message: format!(
                "{except_memory_key} is {current_memory_usage}%, but config is {config_memory_usage}%"
            ),
            activate_at: chrono::Utc::now().timestamp(),
            activated: true,
        };

        is_send_a_new_system_event(
            &client_pool,
            &metadata_cache,
            &message_storage_adapter,
            AlarmType::MemoryUsage,
            current_memory_usage,
            config_memory_usage,
        )
        .await;

        let memory_check_message = metadata_cache
            .get_alarm_event(except_memory_key.as_str())
            .unwrap();

        assert_eq!(memory_check_message.name, except_memory_value.name);
        assert_eq!(memory_check_message.message, except_memory_value.message);
        assert_eq!(
            memory_check_message.activated,
            except_memory_value.activated
        );
    }
}
