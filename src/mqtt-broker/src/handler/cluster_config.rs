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
use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::cluster::{
    AvailableFlag, MqttClusterDynamicConfig, MqttClusterDynamicConfigFeature,
    MqttClusterDynamicConfigNetwork, MqttClusterDynamicConfigProtocol,
    MqttClusterDynamicConfigSecurity, MqttClusterDynamicFlappingDetect,
    MqttClusterDynamicOfflineMessage, MqttClusterDynamicSlowSub,
    DEFAULT_DYNAMIC_CONFIG_FLAPPING_DETECT, DEFAULT_DYNAMIC_CONFIG_OFFLINE_MESSAGE,
    DEFAULT_DYNAMIC_CONFIG_PROTOCOL, DEFAULT_DYNAMIC_CONFIG_SLOW_SUB,
};
use protocol::mqtt::common::QoS;

/// This section primarily implements cache management for cluster-related configuration operations.
/// Through this implementation, we can retrieve configuration information within the cluster
/// and set corresponding cluster configuration attributes.
impl CacheManager {
    pub async fn set_flapping_detect_config(
        &self,
        flapping_detect: MqttClusterDynamicFlappingDetect,
    ) -> Result<(), MqttBrokerError> {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.flapping_detect = flapping_detect.clone();
        }

        self.save_dynamic_config(
            DEFAULT_DYNAMIC_CONFIG_FLAPPING_DETECT,
            flapping_detect.encode(),
        )
        .await?;

        Ok(())
    }

    pub fn get_flapping_detect_config(&self) -> MqttClusterDynamicFlappingDetect {
        self.get_cluster_info().flapping_detect
    }

    pub async fn set_slow_sub_config(
        &self,
        slow_sub: MqttClusterDynamicSlowSub,
    ) -> Result<(), MqttBrokerError> {
        if let Some(mut config) = self.cluster_info.get_mut(&self.cluster_name) {
            config.slow = slow_sub.clone();
        }

        self.save_dynamic_config(DEFAULT_DYNAMIC_CONFIG_SLOW_SUB, slow_sub.encode())
            .await?;

        Ok(())
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

    async fn save_dynamic_config(
        &self,
        resource: &str,
        data: Vec<u8>,
    ) -> Result<(), MqttBrokerError> {
        let client_pool = self.client_pool.clone();
        let cluster_storage = ClusterStorage::new(client_pool);
        cluster_storage
            .set_dynamic_config(&self.cluster_name, resource, data)
            .await?;
        Ok(())
    }
}

pub async fn build_cluster_config(
    client_pool: &Arc<ClientPool>,
) -> Result<MqttClusterDynamicConfig, MqttBrokerError> {
    Ok(MqttClusterDynamicConfig {
        protocol: build_protocol(client_pool).await?,
        feature: build_feature().await?,
        security: build_security().await?,
        network: build_network().await?,
        slow: build_slow_sub().await?,
        flapping_detect: build_flapping_detect().await?,
        offline_message: build_offline_message(client_pool).await?,
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
        session_expiry_interval: 1800,
        topic_alias_max: 65535,
        max_qos: QoS::ExactlyOnce,
        max_packet_size: 1024 * 1024 * 10,
        max_server_keep_alive: 3600,
        default_server_keep_alive: 60,
        receive_max: 65535,
        client_pkid_persistent: false,
        max_message_expiry_interval: 3600,
    })
}

async fn build_feature() -> Result<MqttClusterDynamicConfigFeature, MqttBrokerError> {
    Ok(MqttClusterDynamicConfigFeature {
        retain_available: AvailableFlag::Enable,
        wildcard_subscription_available: AvailableFlag::Enable,
        subscription_identifiers_available: AvailableFlag::Enable,
        shared_subscription_available: AvailableFlag::Enable,
        exclusive_subscription_available: AvailableFlag::Enable,
    })
}

async fn build_security() -> Result<MqttClusterDynamicConfigSecurity, MqttBrokerError> {
    Ok(MqttClusterDynamicConfigSecurity {
        secret_free_login: false,
        is_self_protection_status: false,
    })
}

async fn build_network() -> Result<MqttClusterDynamicConfigNetwork, MqttBrokerError> {
    Ok(MqttClusterDynamicConfigNetwork {
        tcp_max_connection_num: 1000,
        tcps_max_connection_num: 1000,
        websocket_max_connection_num: 1000,
        websockets_max_connection_num: 1000,
        response_max_try_mut_times: 128,
        response_try_mut_sleep_time_ms: 100,
    })
}

async fn build_slow_sub() -> Result<MqttClusterDynamicSlowSub, MqttBrokerError> {
    Ok(MqttClusterDynamicSlowSub {
        enable: false,
        whole_ms: 0,
        internal_ms: 0,
        response_ms: 0,
    })
}

async fn build_flapping_detect() -> Result<MqttClusterDynamicFlappingDetect, MqttBrokerError> {
    Ok(MqttClusterDynamicFlappingDetect {
        enable: false,
        window_time: 1,
        max_client_connections: 15,
        ban_time: 5,
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
