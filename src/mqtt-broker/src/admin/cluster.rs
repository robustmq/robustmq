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
use crate::handler::dynamic_config::{save_cluster_dynamic_config, ClusterDynamicConfig};
use crate::handler::error::MqttBrokerError;
use common_base::enum_type::feature_type::FeatureType;
use grpc_clients::pool::ClientPool;
use protocol::broker::broker_mqtt_admin::{
    GetClusterConfigReply, SetClusterConfigReply, SetClusterConfigRequest,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn set_cluster_config_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: &SetClusterConfigRequest,
) -> Result<SetClusterConfigReply, MqttBrokerError> {
    match FeatureType::from_str(request.feature_name.as_str()) {
        Ok(FeatureType::SlowSubscribe) => {
            let mut config = cache_manager.get_slow_sub_config();
            config.enable = request.is_enable;
            cache_manager.update_slow_sub_config(config.clone());
            save_cluster_dynamic_config(
                client_pool,
                ClusterDynamicConfig::MqttFlappingDetect,
                config.encode(),
            )
            .await?;
        }

        Ok(FeatureType::OfflineMessage) => {
            let mut config = cache_manager.get_offline_message_config();
            config.enable = request.is_enable;
            cache_manager.update_offline_message_config(config.clone());
            save_cluster_dynamic_config(
                client_pool,
                ClusterDynamicConfig::MqttOfflineMessage,
                config.encode(),
            )
            .await?;
        }

        Err(e) => {
            return Err(MqttBrokerError::CommonError(format!(
                "Failed to parse feature type: {e}"
            )));
        }
    }
    Ok(SetClusterConfigReply {
        feature_name: request.feature_name.clone(),
        is_enable: true,
    })
}

pub fn get_cluster_config_by_req(
    cache_manager: &Arc<CacheManager>,
) -> Result<GetClusterConfigReply, MqttBrokerError> {
    Ok(GetClusterConfigReply {
        mqtt_broker_cluster_dynamic_config: serde_json::to_vec(
            &cache_manager.get_cluster_config(),
        )?,
    })
}
