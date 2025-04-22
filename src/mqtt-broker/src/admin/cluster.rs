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
use crate::handler::error::MqttBrokerError;
use common_base::enum_type::feature_type::FeatureType;
use protocol::broker_mqtt::broker_mqtt_admin::SetClusterConfigRequest;
use std::str::FromStr;
use std::sync::Arc;

pub async fn set_cluster_config_by_req(
    cache_manager: &Arc<CacheManager>,
    cluster_config_request: SetClusterConfigRequest,
) -> Result<bool, MqttBrokerError> {
    match FeatureType::from_str(cluster_config_request.feature_name.as_str()) {
        Ok(FeatureType::SlowSubscribe) => Ok(cache_manager
            .update_slow_sub_config(cluster_config_request)
            .await?),
        Ok(FeatureType::OfflineMessage) => Ok(cache_manager
            .update_offline_message_config(cluster_config_request)
            .await?),
        Err(e) => Err(MqttBrokerError::CommonError(format!(
            "Failed to parse feature type: {}",
            e
        ))),
    }
}
