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
use common_base::config::broker_mqtt::{broker_mqtt_conf, ConfigAvailableFlag};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::cluster::{
    AvailableFlag, MqttClusterDynamicConfig, MqttClusterDynamicConfigFeature,
    MqttClusterDynamicConfigNetwork, MqttClusterDynamicConfigProtocol,
    MqttClusterDynamicConfigSecurity, MqttClusterDynamicFlappingDetect,
    MqttClusterDynamicOfflineMessage, MqttClusterDynamicSchemaMessage,
    MqttClusterDynamicSchemaMessageFailedOperation, MqttClusterDynamicSchemaMessageStrategy,
    MqttClusterDynamicSlowSub, DEFAULT_DYNAMIC_CONFIG_FEATURE,
    DEFAULT_DYNAMIC_CONFIG_FLAPPING_DETECT, DEFAULT_DYNAMIC_CONFIG_NETWORK,
    DEFAULT_DYNAMIC_CONFIG_OFFLINE_MESSAGE, DEFAULT_DYNAMIC_CONFIG_PROTOCOL,
    DEFAULT_DYNAMIC_CONFIG_SLOW_SUB,
};
use protocol::broker_mqtt::broker_mqtt_admin::SetClusterConfigRequest;
use protocol::mqtt::common::{qos, QoS};

/// This section primarily implements cache management for cluster-related configuration operations.
/// Through this implementation, we can retrieve configuration information within the cluster
/// and set corresponding cluster configuration attributes.
impl CacheManager {
    pub async fn update_flapping_detect_config(
        &self,
        flapping_detect: MqttClusterDynamicFlappingDetect,
    ) -> Result<(), MqttBrokerError> {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.flapping_detect = flapping_detect.clone();
        }

        Ok(())
    }

    pub fn get_flapping_detect_config(&self) -> MqttClusterDynamicFlappingDetect {
        self.get_cluster_info().flapping_detect
    }

    pub async fn update_slow_sub_config(
        &self,
        cluster_config_request: SetClusterConfigRequest,
    ) -> Result<bool, MqttBrokerError> {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.slow.enable = cluster_config_request.is_enable;
            Ok(cluster_config_request.is_enable)
        } else {
            Err(MqttBrokerError::CommonError(format!(
                "Failed to update slow sub config, cluster name: {} not found",
                self.cluster_name,
            )))
        }
    }
    pub async fn update_offline_message_config(
        &self,
        cluster_config_request: SetClusterConfigRequest,
    ) -> Result<bool, MqttBrokerError> {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.offline_message.enable = cluster_config_request.is_enable;
            Ok(cluster_config_request.is_enable)
        } else {
            Err(MqttBrokerError::CommonError(format!(
                "Failed to update offline message config, cluster name: {} not found",
                self.cluster_name
            )))
        }
    }

    pub fn get_offline_message_config(&self) -> MqttClusterDynamicOfflineMessage {
        self.get_cluster_info().offline_message
    }

    pub fn get_slow_sub_config(&self) -> MqttClusterDynamicSlowSub {
        self.get_cluster_info().slow
    }

    pub fn set_cluster_info(&self, cluster: MqttClusterDynamicConfig) {
        self.cluster_info.insert(self.cluster_name.clone(), cluster);
    }

    pub fn get_cluster_info(&self) -> MqttClusterDynamicConfig {
        self.cluster_info.get(&self.cluster_name).unwrap().clone()
    }

    pub fn get_cluster_config(&self) -> Result<MqttClusterDynamicConfig, MqttBrokerError> {
        if let Some(config) = self.cluster_info.get(&self.cluster_name) {
            Ok(config.clone())
        } else {
            Err(MqttBrokerError::CommonError(format!(
                "Failed to get cluster config, cluster name: {} not found",
                self.cluster_name
            )))
        }
    }
}

pub fn build_default_cluster_config() -> MqttClusterDynamicConfig {
    MqttClusterDynamicConfig {
        protocol: MqttClusterDynamicConfigProtocol {
            session_expiry_interval: 1800,
            topic_alias_max: 65535,
            max_qos: QoS::ExactlyOnce,
            max_packet_size: 1024 * 1024 * 10,
            max_server_keep_alive: 3600,
            default_server_keep_alive: 60,
            receive_max: 65535,
            client_pkid_persistent: false,
            max_message_expiry_interval: 3600,
        },
        feature: MqttClusterDynamicConfigFeature {
            retain_available: AvailableFlag::Enable,
            wildcard_subscription_available: AvailableFlag::Enable,
            subscription_identifiers_available: AvailableFlag::Enable,
            shared_subscription_available: AvailableFlag::Enable,
            exclusive_subscription_available: AvailableFlag::Enable,
        },
        security: MqttClusterDynamicConfigSecurity {
            secret_free_login: false,
            is_self_protection_status: false,
        },
        network: MqttClusterDynamicConfigNetwork {
            tcp_max_connection_num: 1000,
            tcps_max_connection_num: 1000,
            websocket_max_connection_num: 1000,
            websockets_max_connection_num: 1000,
            response_max_try_mut_times: 128,
            response_try_mut_sleep_time_ms: 100,
        },
        slow: MqttClusterDynamicSlowSub {
            enable: false,
            whole_ms: 0,
            internal_ms: 0,
            response_ms: 0,
        },
        flapping_detect: MqttClusterDynamicFlappingDetect {
            enable: false,
            window_time: 1,
            max_client_connections: 15,
            ban_time: 5,
        },
        offline_message: MqttClusterDynamicOfflineMessage { enable: true },
        schema: MqttClusterDynamicSchemaMessage {
            enable: true,
            strategy: MqttClusterDynamicSchemaMessageStrategy::ALL,
            failed_operation: MqttClusterDynamicSchemaMessageFailedOperation::Discard,
            echo_log: true,
            log_level: "info".to_string(),
        },
    }
}

pub async fn build_cluster_config(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicConfig, MqttBrokerError> {
    Ok(MqttClusterDynamicConfig {
        protocol: build_protocol(client_pool).await?,
        feature: build_feature(client_pool).await?,
        security: build_security(client_pool).await?,
        network: build_network(client_pool).await?,
        slow: build_slow_sub(client_pool).await?,
        flapping_detect: build_flapping_detect(client_pool).await?,
        offline_message: build_offline_message(client_pool).await?,
        schema: build_schema(client_pool).await?,
    })
}

async fn build_protocol(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicConfigProtocol, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_PROTOCOL)
        .await?;

    if !data.is_empty() {
        let cluster = serde_json::from_slice::<MqttClusterDynamicConfigProtocol>(&data)?;
        return Ok(cluster);
    }

    Ok(MqttClusterDynamicConfigProtocol {
        session_expiry_interval: conf.cluster_dynamic_config_protocol.session_expiry_interval,
        topic_alias_max: conf.cluster_dynamic_config_protocol.topic_alias_max,
        max_qos: qos(conf.cluster_dynamic_config_protocol.max_qos).unwrap(),
        max_packet_size: conf.cluster_dynamic_config_protocol.max_packet_size,
        max_server_keep_alive: conf.cluster_dynamic_config_protocol.max_server_keep_alive,
        default_server_keep_alive: conf
            .cluster_dynamic_config_protocol
            .default_server_keep_alive,
        receive_max: conf.cluster_dynamic_config_protocol.receive_max,
        client_pkid_persistent: conf.cluster_dynamic_config_protocol.client_pkid_persistent,
        max_message_expiry_interval: conf
            .cluster_dynamic_config_protocol
            .max_message_expiry_interval,
    })
}

async fn build_feature(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicConfigFeature, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_FEATURE)
        .await?;
    if !data.is_empty() {
        let cluster = serde_json::from_slice::<MqttClusterDynamicConfigFeature>(&data)?;
        return Ok(cluster);
    }
    Ok(MqttClusterDynamicConfigFeature {
        retain_available: to_available_flag(
            conf.cluster_dynamic_config_feature.retain_available.clone(),
        ),
        wildcard_subscription_available: to_available_flag(
            conf.cluster_dynamic_config_feature
                .wildcard_subscription_available
                .clone(),
        ),
        subscription_identifiers_available: to_available_flag(
            conf.cluster_dynamic_config_feature
                .subscription_identifiers_available
                .clone(),
        ),
        shared_subscription_available: to_available_flag(
            conf.cluster_dynamic_config_feature
                .shared_subscription_available
                .clone(),
        ),
        exclusive_subscription_available: to_available_flag(
            conf.cluster_dynamic_config_feature
                .exclusive_subscription_available
                .clone(),
        ),
    })
}

fn to_available_flag(flag: ConfigAvailableFlag) -> AvailableFlag {
    match flag {
        ConfigAvailableFlag::Enable => AvailableFlag::Enable,
        ConfigAvailableFlag::Disable => AvailableFlag::Disable,
    }
}
async fn build_security(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicConfigSecurity, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_FEATURE)
        .await?;
    if !data.is_empty() {
        let cluster = serde_json::from_slice::<MqttClusterDynamicConfigSecurity>(&data)?;
        return Ok(cluster);
    }
    Ok(MqttClusterDynamicConfigSecurity {
        secret_free_login: conf.cluster_dynamic_config_security.secret_free_login,
        is_self_protection_status: conf
            .cluster_dynamic_config_security
            .is_self_protection_status,
    })
}

async fn build_network(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicConfigNetwork, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_NETWORK)
        .await?;
    if !data.is_empty() {
        let cluster = serde_json::from_slice::<MqttClusterDynamicConfigNetwork>(&data)?;
        return Ok(cluster);
    }
    Ok(MqttClusterDynamicConfigNetwork {
        tcp_max_connection_num: conf.cluster_dynamic_config_network.tcp_max_connection_num,
        tcps_max_connection_num: conf.cluster_dynamic_config_network.tcps_max_connection_num,
        websocket_max_connection_num: conf
            .cluster_dynamic_config_network
            .websocket_max_connection_num,
        websockets_max_connection_num: conf
            .cluster_dynamic_config_network
            .websockets_max_connection_num,
        response_max_try_mut_times: conf
            .cluster_dynamic_config_network
            .response_max_try_mut_times,
        response_try_mut_sleep_time_ms: conf
            .cluster_dynamic_config_network
            .response_try_mut_sleep_time_ms,
    })
}

async fn build_slow_sub(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicSlowSub, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_SLOW_SUB)
        .await?;
    if !data.is_empty() {
        let cluster = serde_json::from_slice::<MqttClusterDynamicSlowSub>(&data)?;
        return Ok(cluster);
    }
    Ok(MqttClusterDynamicSlowSub {
        enable: conf.cluster_dynamic_config_slow_sub.enable,
        whole_ms: conf.cluster_dynamic_config_slow_sub.whole_ms,
        internal_ms: conf.cluster_dynamic_config_slow_sub.internal_ms,
        response_ms: conf.cluster_dynamic_config_slow_sub.response_ms,
    })
}

async fn build_flapping_detect(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicFlappingDetect, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_FLAPPING_DETECT)
        .await?;
    if !data.is_empty() {
        let cluster = serde_json::from_slice::<MqttClusterDynamicFlappingDetect>(&data)?;
        return Ok(cluster);
    }
    Ok(MqttClusterDynamicFlappingDetect {
        enable: conf.cluster_dynamic_config_flapping_detect.enable,
        window_time: conf.cluster_dynamic_config_flapping_detect.window_time,
        max_client_connections: conf
            .cluster_dynamic_config_flapping_detect
            .max_client_connections as u64,
        ban_time: conf.cluster_dynamic_config_flapping_detect.ban_time,
    })
}

async fn build_offline_message(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicOfflineMessage, MqttBrokerError> {
    let conf = broker_mqtt_conf();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage
        .get_dynamic_config(&conf.cluster_name, DEFAULT_DYNAMIC_CONFIG_OFFLINE_MESSAGE)
        .await?;

    if !data.is_empty() {
        let cluster = serde_json::from_slice::<MqttClusterDynamicOfflineMessage>(&data)?;
        return Ok(cluster);
    }

    Ok(MqttClusterDynamicOfflineMessage {
        enable: conf.offline_messages.enable,
    })
}

async fn build_schema(
    _client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicSchemaMessage, MqttBrokerError> {
    Ok(MqttClusterDynamicSchemaMessage {
        enable: true,
        strategy: MqttClusterDynamicSchemaMessageStrategy::ALL,
        failed_operation: MqttClusterDynamicSchemaMessageFailedOperation::Discard,
        echo_log: true,
        log_level: "info".to_string(),
    })
}
