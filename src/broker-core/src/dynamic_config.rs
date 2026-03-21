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

use crate::cache::NodeCacheManager;
use crate::cluster::ClusterStorage;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use common_config::config::{
    BrokerConfig, MqttFlappingDetect, MqttOfflineMessage, MqttProtocolConfig, MqttSchema,
    MqttSlowSubscribeConfig, MqttSystemMonitor,
};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use strum_macros::Display;

#[derive(Default, Display, Clone, Copy)]
pub enum ClusterDynamicConfig {
    #[default]
    MqttSlowSubscribeConfig,
    MqttFlappingDetect,
    MqttProtocol,
    MqttOfflineMessage,
    MqttSystemMonitor,
    MqttSchema,
    MqttLimit,
    ClusterLimit,
}

pub async fn build_cluster_config(
    client_pool: &Arc<ClientPool>,
) -> Result<BrokerConfig, CommonError> {
    let mut conf = broker_config().clone();
    if let Some(data) = get_mqtt_protocol(client_pool).await? {
        conf.mqtt_protocol = data;
    }

    if let Some(data) = get_slow_subscribe_config(client_pool).await? {
        conf.mqtt_slow_subscribe = data;
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
    node_cache: &Arc<NodeCacheManager>,
    resource_type: ClusterDynamicConfig,
    config: Bytes,
) -> Result<(), CommonError> {
    match resource_type {
        ClusterDynamicConfig::ClusterLimit => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.cluster_limit = data;
        }

        ClusterDynamicConfig::MqttSlowSubscribeConfig => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.mqtt_slow_subscribe = data;
        }

        ClusterDynamicConfig::MqttFlappingDetect => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.mqtt_flapping_detect = data;
        }

        ClusterDynamicConfig::MqttProtocol => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.mqtt_protocol = data;
        }

        ClusterDynamicConfig::MqttOfflineMessage => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.mqtt_offline_message = data;
        }

        ClusterDynamicConfig::MqttSystemMonitor => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.mqtt_system_monitor = data;
        }

        ClusterDynamicConfig::MqttSchema => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.mqtt_schema = data;
        }
        ClusterDynamicConfig::MqttLimit => {
            let data = serde_json::from_slice(&config)?;
            let mut config = node_cache.cluster_config.write().await;
            config.mqtt_limit = data;
        }
    }
    Ok(())
}

pub async fn save_cluster_dynamic_config(
    client_pool: &Arc<ClientPool>,
    resource_config: ClusterDynamicConfig,
    data: Vec<u8>,
) -> Result<(), CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    cluster_storage
        .set_dynamic_config(&resource_config.to_string(), data)
        .await?;
    Ok(())
}

async fn get_mqtt_protocol(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttProtocolConfig>, CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&ClusterDynamicConfig::MqttProtocol.to_string())
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttProtocolConfig>(&data)?));
    }

    Ok(None)
}

async fn get_slow_subscribe_config(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttSlowSubscribeConfig>, CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&ClusterDynamicConfig::MqttSlowSubscribeConfig.to_string())
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
) -> Result<Option<MqttFlappingDetect>, CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&ClusterDynamicConfig::MqttFlappingDetect.to_string())
        .await?;
    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttFlappingDetect>(&data)?));
    }
    Ok(None)
}

async fn get_offline_message(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttOfflineMessage>, CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&ClusterDynamicConfig::MqttOfflineMessage.to_string())
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttOfflineMessage>(&data)?));
    }

    Ok(None)
}

async fn get_schema(client_pool: &Arc<ClientPool>) -> Result<Option<MqttSchema>, CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&ClusterDynamicConfig::MqttSchema.to_string())
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttSchema>(&data)?));
    }

    Ok(None)
}

async fn get_system_monitor(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<MqttSystemMonitor>, CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&ClusterDynamicConfig::MqttSystemMonitor.to_string())
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<MqttSystemMonitor>(&data)?));
    }

    Ok(None)
}
