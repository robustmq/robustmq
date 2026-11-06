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
use serde::Deserialize;
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
    KafkaDynamic,
}

/// Wire shape for the `KafkaDynamic` dynamic-config entry. `KafkaDynamic` was
/// folded into `KafkaRuntime` as a plain field, but the RocksDB/Raft-stored
/// bytes under the `KafkaDynamic` key still hold this exact single-field JSON
/// shape, so this struct exists purely to decode/encode that stored value.
#[derive(Debug, Deserialize, serde::Serialize, Clone)]
struct KafkaDynamicWire {
    auto_create_topics_enable: bool,
}

pub async fn build_cluster_config(
    client_pool: &Arc<ClientPool>,
) -> Result<BrokerConfig, CommonError> {
    let mut conf = broker_config().clone();
    if let Some(data) = get_mqtt_protocol(client_pool).await? {
        conf.mqtt_runtime.protocol = data;
    }

    if let Some(data) = get_slow_subscribe_config(client_pool).await? {
        conf.mqtt_runtime.slow_subscribe = data;
    }

    if let Some(data) = get_flapping_detect(client_pool).await? {
        conf.mqtt_runtime.flapping_detect = data;
    }

    if let Some(data) = get_offline_message(client_pool).await? {
        conf.mqtt_runtime.offline_message = data;
    }

    if let Some(data) = get_schema(client_pool).await? {
        conf.mqtt_runtime.schema = data;
    }

    if let Some(data) = get_system_monitor(client_pool).await? {
        conf.mqtt_runtime.system_monitor = data;
    }

    if let Some(data) = get_kafka_dynamic(client_pool).await? {
        conf.kafka_runtime.auto_create_topics_enable = data.auto_create_topics_enable;
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
            new_config.mqtt_runtime.slow_subscribe = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttFlappingDetect => {
            new_config.mqtt_runtime.flapping_detect = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttProtocol => {
            new_config.mqtt_runtime.protocol = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttOfflineMessage => {
            new_config.mqtt_runtime.offline_message = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttSystemMonitor => {
            new_config.mqtt_runtime.system_monitor = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttSchema => {
            new_config.mqtt_runtime.schema = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MqttLimit => {
            new_config.mqtt_runtime.limit = serde_json::from_slice(&config)?;
        }
        ClusterDynamicConfig::MetaRuntime => {
            new_config.meta_runtime = serde_json::from_slice::<MetaRuntime>(&config)?;
        }
        ClusterDynamicConfig::KafkaDynamic => {
            let data: KafkaDynamicWire = serde_json::from_slice(&config)?;
            new_config.kafka_runtime.auto_create_topics_enable = data.auto_create_topics_enable;
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

async fn get_kafka_dynamic(
    client_pool: &Arc<ClientPool>,
) -> Result<Option<KafkaDynamicWire>, CommonError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&ClusterDynamicConfig::KafkaDynamic.to_string())
        .await?;

    if !data.is_empty() {
        return Ok(Some(serde_json::from_slice::<KafkaDynamicWire>(&data)?));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{update_cluster_dynamic_config, ClusterDynamicConfig};
    use crate::cache::NodeCacheManager;
    use bytes::Bytes;
    use common_config::broker::default_broker_config;
    use std::str::FromStr;
    use std::sync::Arc;

    #[test]
    fn dynamic_config_names_match_stored_keys() {
        let cases = [
            (
                ClusterDynamicConfig::MqttSlowSubscribeConfig,
                "MqttSlowSubscribeConfig",
            ),
            (
                ClusterDynamicConfig::MqttFlappingDetect,
                "MqttFlappingDetect",
            ),
            (ClusterDynamicConfig::MqttProtocol, "MqttProtocol"),
            (
                ClusterDynamicConfig::MqttOfflineMessage,
                "MqttOfflineMessage",
            ),
            (ClusterDynamicConfig::MqttSystemMonitor, "MqttSystemMonitor"),
            (ClusterDynamicConfig::MqttSchema, "MqttSchema"),
            (ClusterDynamicConfig::MqttLimit, "MqttLimit"),
            (ClusterDynamicConfig::ClusterLimit, "ClusterLimit"),
            (ClusterDynamicConfig::MetaRuntime, "MetaRuntime"),
            (ClusterDynamicConfig::KafkaDynamic, "KafkaDynamic"),
        ];

        for (config, expected_key) in cases {
            assert_eq!(config.to_string(), expected_key);
            assert_eq!(
                ClusterDynamicConfig::from_str(expected_key)
                    .unwrap()
                    .to_string(),
                expected_key
            );
        }
    }

    #[test]
    fn updates_each_supported_dynamic_config() {
        let cache = Arc::new(NodeCacheManager::new(default_broker_config()));
        let cases = [
            (
                ClusterDynamicConfig::ClusterLimit,
                r#"{"max_network_connection":17}"#,
                "cluster_limit.max_network_connection",
                "17",
            ),
            (
                ClusterDynamicConfig::MqttSlowSubscribeConfig,
                r#"{"enable":true,"record_time":23}"#,
                "mqtt_runtime.slow_subscribe.record_time",
                "23",
            ),
            (
                ClusterDynamicConfig::MqttFlappingDetect,
                r#"{"enable":true,"window_time":29}"#,
                "mqtt_runtime.flapping_detect.window_time",
                "29",
            ),
            (
                ClusterDynamicConfig::MqttProtocol,
                r#"{"topic_alias_max":31}"#,
                "mqtt_runtime.protocol.topic_alias_max",
                "31",
            ),
            (
                ClusterDynamicConfig::MqttOfflineMessage,
                r#"{"max_messages_num":37}"#,
                "mqtt_runtime.offline_message.max_messages_num",
                "37",
            ),
            (
                ClusterDynamicConfig::MqttSystemMonitor,
                r#"{"enable":true,"system_topic_interval_ms":41}"#,
                "mqtt_runtime.system_monitor.system_topic_interval_ms",
                "41",
            ),
            (
                ClusterDynamicConfig::MqttSchema,
                r#"{"enable":true,"log_level":"debug"}"#,
                "mqtt_runtime.schema.log_level",
                r#""debug""#,
            ),
            (
                ClusterDynamicConfig::MqttLimit,
                r#"{"cluster":{"max_topics":43},"tenant":{"max_topics":47}}"#,
                "mqtt_runtime.limit.cluster.max_topics",
                "43",
            ),
            (
                ClusterDynamicConfig::MetaRuntime,
                r#"{"heartbeat_timeout_ms":53}"#,
                "meta_runtime.heartbeat_timeout_ms",
                "53",
            ),
            (
                ClusterDynamicConfig::KafkaDynamic,
                r#"{"auto_create_topics_enable":false}"#,
                "kafka_runtime.auto_create_topics_enable",
                "false",
            ),
        ];

        for (kind, payload, field_path, expected) in cases {
            update_cluster_dynamic_config(&cache, kind, Bytes::from(payload)).unwrap();
            let value = serde_json::to_value(cache.get_cluster_config()).unwrap();
            let actual = field_path
                .split('.')
                .fold(&value, |value, field| &value[field]);
            let expected = serde_json::from_str::<serde_json::Value>(expected).unwrap();
            assert_eq!(actual, &expected, "unexpected value for {field_path}");
        }
    }

    #[test]
    fn rejects_invalid_dynamic_config_without_changing_cached_config() {
        let cache = Arc::new(NodeCacheManager::new(default_broker_config()));
        let original = serde_json::to_value(cache.get_cluster_config()).unwrap();

        let result = update_cluster_dynamic_config(
            &cache,
            ClusterDynamicConfig::KafkaDynamic,
            Bytes::from_static(b"not-json"),
        );

        assert!(result.is_err());
        assert_eq!(
            serde_json::to_value(cache.get_cluster_config()).unwrap(),
            original
        );
    }
}
