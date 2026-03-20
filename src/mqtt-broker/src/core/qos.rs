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

use crate::core::{cache::MQTTCacheManager, error::MqttBrokerError};
use protocol::broker::broker_mqtt::{GetQosDataByClientIdRaw, GetQosDataByClientIdReply};

pub fn try_broadcast_get_pkid() {
    
}

pub async fn get_qos_data_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_ids: &[String],
) -> Result<GetQosDataByClientIdReply, MqttBrokerError> {
    let mut data = Vec::with_capacity(client_ids.len());

    for client_id in client_ids.iter() {
        if let Some(inner) = cache_manager.pkid_manager.qos_pkid_data.get(client_id) {
            let pkid_map: std::collections::HashMap<u64, _> = inner
                .iter()
                .map(|entry| (*entry.key(), entry.value().clone()))
                .collect();

            let qos_data = serde_json::to_vec(&pkid_map)
                .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
            data.push(GetQosDataByClientIdRaw {
                client_id: client_id.clone(),
                qos_data,
            });
        }
    }

    Ok(GetQosDataByClientIdReply { data })
}
