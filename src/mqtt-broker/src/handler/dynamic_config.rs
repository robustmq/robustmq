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
use strum_macros::{Display, EnumString};

#[derive(Default, EnumString, Display)]
pub enum ClusterDynamicConfig {
    #[default]
    SlowSub,
    FlappingDetect,
    Protocol,
    OfflineMessage,
    Feature,
    Security,
    NetworkThread,
    SystemMonitor,
    Schema,
}

impl CacheManager {
    // slow sub
    pub fn update_slow_sub_config(&self, slow_sub: SlowSub) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.slow_sub = slow_sub.to_owned();
        }
    }

    pub fn get_slow_sub_config(&self) -> SlowSub {
        self.get_cluster_config().slow_sub
    }

    // flapping detect
    pub fn update_flapping_detect_config(&self, flapping_detect: FlappingDetect) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.flapping_detect = flapping_detect;
        }
    }

    pub fn get_flapping_detect_config(&self) -> FlappingDetect {
        self.get_cluster_config().flapping_detect
    }

    // mqtt protocol config
    pub fn update_mqtt_protocol_config(&self, mqtt_protocol_config: MqttProtocolConfig) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.mqtt_protocol_config = mqtt_protocol_config;
        }
    }

    pub fn get_mqtt_protocol_config(&self) -> MqttProtocolConfig {
        self.get_cluster_config().mqtt_protocol_config
    }

    // offline message
    pub fn update_offline_message_config(&self, offline_message: OfflineMessage) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.offline_messages = offline_message;
        }
    }

    pub fn get_offline_message_config(&self) -> OfflineMessage {
        self.get_cluster_config().offline_messages
    }

    // feature
    pub fn update_feature_config(&self, feature_config: Feature) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.feature = feature_config;
        }
    }

    pub fn get_feature_config(&self) -> Feature {
        self.get_cluster_config().feature
    }

    // system monitor
    pub fn update_system_monitor_config(&self, system_monitor: SystemMonitor) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.system_monitor = system_monitor;
        }
    }

    pub fn get_system_monitor_config(&self) -> SystemMonitor {
        self.get_cluster_config().system_monitor
    }

    // schema
    pub fn update_schema_config(&self, schema: Schema) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.schema = schema;
        }
    }

    pub fn get_shema_config(&self) -> Schema {
        self.get_cluster_config().schema
    }

    // schema
    pub fn update_security_config(&self, security: Security) {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.security = security;
        }
    }

    pub fn get_security_config(&self) -> Security {
        self.get_cluster_config().security
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

    if let Some(data) = get_feature_support(client_pool).await? {
        conf.feature = data;
    }

    if let Some(data) = get_security_config(client_pool).await? {
        conf.security = data;
    }

    if let Some(data) = get_network_thread(client_pool).await? {
        conf.network_thread = data;
    }

    if let Some(data) = get_slow_sub(client_pool).await? {
        conf.slow_sub = data;
    }

    if let Some(data) = get_flapping_detect(client_pool).await? {
        conf.flapping_detect = data;
    }

    if let Some(data) = get_offline_message(client_pool).await? {
        conf.offline_messages = data;
    }

    if let Some(data) = get_schema(client_pool).await? {
        conf.schema = data;
    }

    if let Some(data) = get_system_monitor(client_pool).await? {
        conf.system_monitor = data;
    }

    Ok(conf)
}

pub async fn update_cluster_dynamic_config(
    cache_manager: &Arc<CacheManager>,
    resource_type: ClusterDynamicConfig,
    config: Vec<u8>,
) -> Result<(), MqttBrokerError> {
    match resource_type {
        ClusterDynamicConfig::SlowSub => {
            let slow_sub = serde_json::from_slice(&config)?;
            cache_manager.update_slow_sub_config(slow_sub);
        }
        ClusterDynamicConfig::FlappingDetect => {
            let flapping_detect = serde_json::from_slice(&config)?;
            cache_manager.update_flapping_detect_config(flapping_detect);
        }
        ClusterDynamicConfig::Protocol => {
            let mqtt_protocol = serde_json::from_slice(&config)?;
            cache_manager.update_mqtt_protocol_config(mqtt_protocol);
        }
        ClusterDynamicConfig::OfflineMessage => {
            let mqtt_protocol = serde_json::from_slice(&config)?;
            cache_manager.update_offline_message_config(mqtt_protocol);
        }
        ClusterDynamicConfig::Feature => {
            let mqtt_protocol = serde_json::from_slice(&config)?;
            cache_manager.update_feature_config(mqtt_protocol);
        }
        ClusterDynamicConfig::NetworkThread => {
            let mqtt_protocol = serde_json::from_slice(&config)?;
            cache_manager.update_feature_config(mqtt_protocol);
        }
        ClusterDynamicConfig::SystemMonitor => {
            let system_monitor = serde_json::from_slice(&config)?;
            cache_manager.update_system_monitor_config(system_monitor);
        }
        ClusterDynamicConfig::Schema => {
            let schema_config = serde_json::from_slice(&config)?;
            cache_manager.update_schema_config(schema_config);
        }
        ClusterDynamicConfig::Security => {
            let security_config = serde_json::from_slice(&config)?;
            cache_manager.update_security_config(security_config);
        }
    }
    Ok(())
}

pub async fn save_cluster_dynamic_cofig(
    client_pool: &Arc<ClientPool>,
    resource_config: ClusterDynamicConfig,
    data: Vec<u8>,
) -> Result<(), MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    cluster_storage
        .set_dynamic_config(&conf.cluster_name, &resource_config.to_string(), data)
        .await?;
    Ok(())
}

async fn get_mqtt_protocol_config(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttProtocolConfig>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::Protocol.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttProtocolConfig>(&data)?));
    }

    Ok(None)
}

async fn get_feature_support(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<Feature>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::Feature.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<Feature>(&data)?));
    }
    Ok(None)
}

async fn get_security_config(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<Security>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::Security.to_string(),
        )
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<Security>(&data)?));
    }
    Ok(None)
}

async fn get_network_thread(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<NetworkThread>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::NetworkThread.to_string(),
        )
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<NetworkThread>(&data)?));
    }
    Ok(None)
}

async fn get_slow_sub(client_pool: &Arc<ClientPool>) -> Result<Option<SlowSub>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::SlowSub.to_string(),
        )
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<SlowSub>(&data)?));
    }
    Ok(None)
}

async fn get_flapping_detect(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<FlappingDetect>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::FlappingDetect.to_string(),
        )
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<FlappingDetect>(&data)?));
    }
    Ok(None)
}

async fn get_offline_message(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<OfflineMessage>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::OfflineMessage.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<OfflineMessage>(&data)?));
    }

    Ok(None)
}

async fn get_schema(client_pool: &Arc<ClientPool>) -> Result<Option<Schema>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::Schema.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<Schema>(&data)?));
    }

    Ok(None)
}

async fn get_system_monitor(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<SystemMonitor>, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::SystemMonitor.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<SystemMonitor>(&data)?));
    }

    Ok(None)
}
