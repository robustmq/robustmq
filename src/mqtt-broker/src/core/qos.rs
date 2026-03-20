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

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{
    cache::MQTTCacheManager, error::MqttBrokerError, pkid_manager::ReceiveQosPkidData,
};
use node_call::{NodeCallData, NodeCallManager};
use prost::Message;
use protocol::broker::broker_mqtt::{GetQosDataByClientIdRaw, GetQosDataByClientIdReply};

pub async fn try_broadcast_get_pkid(
    node_call: &Arc<NodeCallManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    client_id: &str,
    pkid: u16,
) -> Result<Option<ReceiveQosPkidData>, MqttBrokerError> {
    let raw_replies = node_call
        .send_with_reply(NodeCallData::GetQosData(client_id.to_string()))
        .await?;

    for raw in raw_replies {
        let reply = GetQosDataByClientIdReply::decode(raw)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
        for record in reply.data {
            let pkid_map: HashMap<u64, ReceiveQosPkidData> =
                serde_json::from_slice(&record.qos_data)
                    .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
            for (_, pkid_data) in pkid_map {
                cache_manager
                    .pkid_manager
                    .add_qos_pkid_data(&record.client_id, pkid_data);
            }
        }
    }

    Ok(cache_manager
        .pkid_manager
        .get_qos_pkid_data(client_id, pkid))
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
