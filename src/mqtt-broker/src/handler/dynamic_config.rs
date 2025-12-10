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

use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;
use broker_core::cluster::ClusterStorage;
use bytes::Bytes;
use common_config::broker::broker_config;
use common_config::config::{
    BrokerConfig, MqttFlappingDetect, MqttOfflineMessage, MqttProtocolConfig, MqttSchema,
    MqttSecurity, MqttSlowSubscribeConfig, MqttSystemMonitor,
};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use strum_macros::{Display, EnumString};

#[derive(Default, EnumString, Display)]
pub enum ClusterDynamicConfig {
    #[default]
    MqttSlowSubscribeConfig,
    MqttFlappingDetect,
    MqttProtocol,
    MqttOfflineMessage,
    MqttSecurity,
    MqttSystemMonitor,
    MqttSchema,
}

impl MQTTCacheManager {
    // slow sub
    pub async fn update_slow_sub_config(&self, slow_sub: MqttSlowSubscribeConfig) {
        let mut config = self.broker_cache.cluster_config.write().await;
        config.mqtt_slow_subscribe_config = slow_sub;
    }

    pub async fn get_slow_sub_config(&self) -> MqttSlowSubscribeConfig {
        self.broker_cache
            .get_cluster_config()
            .await
            .mqtt_slow_subscribe_config
    }

    // flapping detect
    pub async fn update_flapping_detect_config(&self, flapping_detect: MqttFlappingDetect) {
        let mut config = self.broker_cache.cluster_config.write().await;
        config.mqtt_flapping_detect = flapping_detect;
    }

    pub async fn get_flapping_detect_config(&self) -> MqttFlappingDetect {
        self.broker_cache
            .get_cluster_config()
            .await
            .mqtt_flapping_detect
    }

    // mqtt protocol config
    pub async fn update_mqtt_protocol_config(&self, mqtt_protocol_config: MqttProtocolConfig) {
        let mut config = self.broker_cache.cluster_config.write().await;
        config.mqtt_protocol_config = mqtt_protocol_config;
    }

    pub async fn get_mqtt_protocol_config(&self) -> MqttProtocolConfig {
        self.broker_cache
            .get_cluster_config()
            .await
            .mqtt_protocol_config
    }

    // offline message
    pub async fn update_offline_message_config(&self, offline_message: MqttOfflineMessage) {
        let mut config = self.broker_cache.cluster_config.write().await;
        config.mqtt_offline_message = offline_message;
    }

    pub async fn get_offline_message_config(&self) -> MqttOfflineMessage {
        self.broker_cache
            .get_cluster_config()
            .await
            .mqtt_offline_message
    }

    // system monitor
    pub async fn update_system_monitor_config(&self, system_monitor: MqttSystemMonitor) {
        let mut config = self.broker_cache.cluster_config.write().await;
        config.mqtt_system_monitor = system_monitor;
    }

    pub async fn get_system_monitor_config(&self) -> MqttSystemMonitor {
        self.broker_cache
            .get_cluster_config()
            .await
            .mqtt_system_monitor
    }

    // schema
    pub async fn update_schema_config(&self, schema: MqttSchema) {
        let mut config = self.broker_cache.cluster_config.write().await;
        config.mqtt_schema = schema;
    }

    pub async fn get_schema_config(&self) -> MqttSchema {
        self.broker_cache.get_cluster_config().await.mqtt_schema
    }

    // schema
    pub async fn update_security_config(&self, security: MqttSecurity) {
        let mut config = self.broker_cache.cluster_config.write().await;
        config.mqtt_security = security;
    }

    pub async fn get_security_config(&self) -> MqttSecurity {
        self.broker_cache.get_cluster_config().await.mqtt_security
    }
}

pub async fn build_cluster_config(
    client_pool: &Arc<ClientPool>,
) -> Result<BrokerConfig, MqttBrokerError> {
    let mut conf = broker_config().clone();
    if let Some(data) = get_mqtt_protocol_config(client_pool).await? {
        conf.mqtt_protocol_config = data;
    }

    if let Some(data) = get_security_config(client_pool).await? {
        conf.mqtt_security = data;
    }

    if let Some(data) = get_slow_subscribe_config(client_pool).await? {
        conf.mqtt_slow_subscribe_config = data;
    }

    if let Some(data) = get_flapping_detect(client_pool).await? {
        conf.mqtt_flapping_detect = data;
    }

    if let Some(data) = get_offline_message(client_pool).await? {
        conf.mqtt_offline_message = data;
    }

    if let Some(data) = get_schema(client_pool).await? {
        conf.mqtt_schema = data;
    }

    if let Some(data) = get_system_monitor(client_pool).await? {
        conf.mqtt_system_monitor = data;
    }

    Ok(conf)
}

pub async fn update_cluster_dynamic_config(
    cache_manager: &Arc<MQTTCacheManager>,
    resource_type: ClusterDynamicConfig,
    config: Bytes,
) -> ResultMqttBrokerError {
    match resource_type {
        ClusterDynamicConfig::MqttSlowSubscribeConfig => {
            let slow_subscribe_config = serde_json::from_slice(&config)?;
            cache_manager
                .update_slow_sub_config(slow_subscribe_config)
                .await;
        }
        ClusterDynamicConfig::MqttFlappingDetect => {
            let flapping_detect = serde_json::from_slice(&config)?;
            cache_manager
                .update_flapping_detect_config(flapping_detect)
                .await;
        }
        ClusterDynamicConfig::MqttProtocol => {
            let mqtt_protocol = serde_json::from_slice(&config)?;
            cache_manager
                .update_mqtt_protocol_config(mqtt_protocol)
                .await;
        }
        ClusterDynamicConfig::MqttOfflineMessage => {
            let mqtt_protocol = serde_json::from_slice(&config)?;
            cache_manager
                .update_offline_message_config(mqtt_protocol)
                .await;
        }

        ClusterDynamicConfig::MqttSystemMonitor => {
            let system_monitor = serde_json::from_slice(&config)?;
            cache_manager
                .update_system_monitor_config(system_monitor)
                .await;
        }
        ClusterDynamicConfig::MqttSchema => {
            let schema_config = serde_json::from_slice(&config)?;
            cache_manager.update_schema_config(schema_config).await;
        }
        ClusterDynamicConfig::MqttSecurity => {
            let security_config = serde_json::from_slice(&config)?;
            cache_manager.update_security_config(security_config).await;
        }
    }
    Ok(())
}

pub async fn save_cluster_dynamic_config(
    client_pool: &Arc<ClientPool>,
    resource_config: ClusterDynamicConfig,
    data: Vec<u8>,
) -> ResultMqttBrokerError {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    cluster_storage
        .set_dynamic_config(&conf.cluster_name, &resource_config.to_string(), data)
        .await?;
    Ok(())
}

async fn get_mqtt_protocol_config(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttProtocolConfig>, MqttBrokerError> {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::MqttProtocol.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttProtocolConfig>(&data)?));
    }

    Ok(None)
}

async fn get_security_config(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttSecurity>, MqttBrokerError> {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::MqttSecurity.to_string(),
        )
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttSecurity>(&data)?));
    }
    Ok(None)
}

async fn get_slow_subscribe_config(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttSlowSubscribeConfig>, MqttBrokerError> {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::MqttSlowSubscribeConfig.to_string(),
        )
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttSlowSubscribeConfig>(
            &data,
        )?));
    }
    Ok(None)
}

async fn get_flapping_detect(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttFlappingDetect>, MqttBrokerError> {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::MqttFlappingDetect.to_string(),
        )
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttFlappingDetect>(&data)?));
    }
    Ok(None)
}

async fn get_offline_message(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttOfflineMessage>, MqttBrokerError> {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::MqttOfflineMessage.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttOfflineMessage>(&data)?));
    }

    Ok(None)
}

async fn get_schema(client_pool: &Arc<ClientPool>) -> Result<Option<MqttSchema>, MqttBrokerError> {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::MqttSchema.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttSchema>(&data)?));
    }

    Ok(None)
}

async fn get_system_monitor(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttSystemMonitor>, MqttBrokerError> {
    let conf = broker_config();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(
            &conf.cluster_name,
            &ClusterDynamicConfig::MqttSystemMonitor.to_string(),
        )
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttSystemMonitor>(&data)?));
    }

    Ok(None)
}
