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

use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::cluster::ClusterStorage;
use common_config::mqtt::broker_mqtt_conf;
use common_config::mqtt::config::{
    BrokerMqttConfig, Feature, FlappingDetect, MqttProtocolConfig, NetworkThread, OfflineMessage,
    Schema, Security, SlowSub, SystemMonitor,
};
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::SetClusterConfigRequest;

pub const DEFAULT_DYNAMIC_CONFIG_SLOW_SUB: &str = "slow_sub";
pub const DEFAULT_DYNAMIC_CONFIG_FLAPPING_DETECT: &str = "flapping_detect";
pub const DEFAULT_DYNAMIC_CONFIG_PROTOCOL: &str = "protocol";
pub const DEFAULT_DYNAMIC_CONFIG_OFFLINE_MESSAGE: &str = "offline_message";
pub const DEFAULT_DYNAMIC_CONFIG_FEATURE: &str = "feature";
pub const DEFAULT_DYNAMIC_CONFIG_SECURITY: &str = "security";
pub const DEFAULT_DYNAMIC_CONFIG_TCP_THREAD: &str = "tcp_thread";
pub const DEFAULT_DYNAMIC_CONFIG_SYSTEM_MONITOR: &str = "system_monitor";
pub const DEFAULT_DYNAMIC_CONFIG_SCHEMA: &str = "schema";

impl CacheManager {
    // flapping detect
    pub async fn update_flapping_detect_config(&self, flapping_detect: FlappingDetect) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.flapping_detect = flapping_detect;
        }
    }

    pub fn get_flapping_detect_config(&self) -> FlappingDetect {
        self.get_cluster_config().flapping_detect
    }

    // slow sub
    pub async fn update_slow_sub_config(&self, request: &SetClusterConfigRequest) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.slow_sub.enable = request.is_enable;
        }
    }

    pub fn get_slow_sub_config(&self) -> SlowSub {
        self.get_cluster_config().slow_sub
    }

    // system monitor
    pub async fn update_system_monitor_config(&self, system_monitor: SystemMonitor) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.system_monitor = system_monitor;
        }
    }

    pub fn get_system_monitor_config(&self) -> SystemMonitor {
        self.get_cluster_config().system_monitor
    }

    // offline message
    pub async fn update_offline_message_config(&self, request: &SetClusterConfigRequest) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.offline_messages.enable = request.is_enable;
        }
    }

    pub fn get_offline_message_config(&self) -> OfflineMessage {
        self.get_cluster_config().offline_messages
    }

    // cluster config
    pub fn set_cluster_config(&self, cluster: BrokerMqttConfig) {
        self.cluster_info.insert(self.cluster_name.clone(), cluster);
    }

    pub fn get_cluster_config(&self) -> BrokerMqttConfig {
        self.cluster_info.get(&self.cluster_name).unwrap().clone()
    }
}

pub async fn build_cluster_config(
    client_pool: &Arc<ClientPool>,
) -> Result<BrokerMqttConfig, MqttBrokerError> {
    let mut conf = broker_mqtt_conf().clone();
    if let Some(data) = get_mqtt_protocol_config(client_pool).await? {
        conf.mqtt_protocol_config = data;
    }

    if let Some(data) = build_feature(client_pool).await? {
        conf.feature = data;
    }

    if let Some(data) = build_security(client_pool).await? {
        conf.security = data;
    }

    if let Some(data) = build_thread(client_pool).await? {
        conf.network_thread = data;
    }

    if let Some(data) = build_slow_sub(client_pool).await? {
        conf.slow_sub = data;
    }

    if let Some(data) = build_flapping_detect(client_pool).await? {
        conf.flapping_detect = data;
    }

    if let Some(data) = build_offline_message(client_pool).await? {
        conf.offline_messages = data;
    }

    if let Some(data) = build_schema(client_pool).await? {
        conf.schema = data;
    }

    if let Some(data) = builder_system_monitor(client_pool).await? {
        conf.system_monitor = data;
    }

    Ok(conf)
}

async fn get_mqtt_protocol_config(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttProtocolConfig>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_PROTOCOL)
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttProtocolConfig>(&data)?));
    }

    Ok(None)
}

async fn build_feature(client_pool: &Arc<ClientPool>) -> Result<Option<Feature>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_FEATURE)
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<Feature>(&data)?));
    }
    Ok(None)
}

async fn build_security(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<Security>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_FEATURE)
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<Security>(&data)?));
    }
    Ok(None)
}

async fn build_thread(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<NetworkThread>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_TCP_THREAD)
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<NetworkThread>(&data)?));
    }
    Ok(None)
}

async fn build_slow_sub(client_pool: &Arc<ClientPool>) -> Result<Option<SlowSub>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_SLOW_SUB)
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<SlowSub>(&data)?));
    }
    Ok(None)
}

async fn build_flapping_detect(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<FlappingDetect>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_FLAPPING_DETECT)
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<FlappingDetect>(&data)?));
    }
    Ok(None)
}

async fn build_offline_message(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<OfflineMessage>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_OFFLINE_MESSAGE)
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<OfflineMessage>(&data)?));
    }

    Ok(None)
}

async fn build_schema(client_pool: &Arc<ClientPool>) -> Result<Option<Schema>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_OFFLINE_MESSAGE)
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<Schema>(&data)?));
    }

    Ok(None)
}

async fn builder_system_monitor(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<SystemMonitor>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_SYSTEM_MONITOR)
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<SystemMonitor>(&data)?));
    }

    Ok(None)
}
