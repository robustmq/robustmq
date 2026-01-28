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

use crate::core::{cache::MQTTCacheManager, error::MqttBrokerError};
use std::sync::Arc;

pub async fn check_max_qos_flight_message(
    cache_manager: &Arc<MQTTCacheManager>,
    client_id: &str,
) -> Result<(), MqttBrokerError> {
    let cluster_config = cache_manager.broker_cache.cluster_config.read().await;
    let current_len = cache_manager
        .qos_data
        .get_receive_publish_pkid_data_len_by_client_ids(client_id);
    let config_len = cluster_config.mqtt_protocol_config.max_qos_flight_message as usize;
    if current_len > config_len {
        return Err(MqttBrokerError::CommonError(format!(
            "Receive maximum quota exceeded. Current: {}, Maximum: {}",
            current_len, config_len
        )));
    }
    Ok(())
}
