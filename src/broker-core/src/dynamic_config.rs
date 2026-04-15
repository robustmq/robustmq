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
    BrokerConfig, MetaRuntime, MqttFlappingDetect, MqttOfflineMessage, MqttProtocolConfig,
    MqttSchema, MqttSlowSubscribeConfig, MqttSystemMonitor,
};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use strum_macros::{Display, EnumString};

#[derive(Default, Display, EnumString, Clone, Copy)]
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
    MetaRuntime,
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

pub fn update_cluster_dynamic_config(
    node_cache: &Arc<NodeCacheManager>,
    resource_type: ClusterDynamicConfig,
    config: Bytes,
) -> Result<(), CommonError> {
    let mut new_config = node_cache.get_cluster_config();
    match resource_type {
        ClusterDynamicConfig::ClusterLimit => {
            new_config.cluster_limit = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttSlowSubscribeConfig => {
            new_config.mqtt_slow_subscribe = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttFlappingDetect => {
            new_config.mqtt_flapping_detect = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttProtocol => {
            new_config.mqtt_protocol = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttOfflineMessage => {
            new_config.mqtt_offline_message = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttSystemMonitor => {
            new_config.mqtt_system_monitor = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttSchema => {
            new_config.mqtt_schema = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttLimit => {
            new_config.mqtt_limit = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MetaRuntime => {
            new_config.meta_runtime = serde_json::from_slice::<MetaRuntime>(&config)?;
        }
    }
    node_cache.set_cluster_config(new_config);
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
