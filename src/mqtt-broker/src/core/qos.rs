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

use crate::core::{
    cache::MQTTCacheManager, error::MqttBrokerError, pkid_manager::ReceiveQosPkidData,
};
use broker_core::cache::NodeCacheManager;
use common_base::utils::serialize;
use common_base::{tools::now_second, uuid::unique_id};
use common_config::storage::StorageType;
use connector::storage::message::MessageStorage;
use delay_message::manager::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    mqtt::topic::Topic,
    storage::{adapter_record::AdapterWriteRecord, shard::EngineShardConfig},
    tenant::DEFAULT_TENANT,
};
use node_call::{NodeCallData, NodeCallManager};
use prost::Message;
use protocol::broker::broker_mqtt::{GetQosDataByClientIdRaw, GetQosDataByClientIdReply};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use storage_adapter::{driver::StorageDriverManager, topic::create_topic_full};
use tracing::info;

pub const DELAY_QUEUE_MESSAGE_TOPIC: &str = "$sys/qos2-inner-topic";

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct Qos2TemporaryMessage {
    tenant: String,
    topic: String,
    record: AdapterWriteRecord,
}

pub async fn init_qos2_inner_topic(
    client_pool: &Arc<ClientPool>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    broker_cache: &Arc<NodeCacheManager>,
) -> Result<(), MqttBrokerError> {
    if broker_cache
        .get_topic_by_name(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC)
        .is_some()
    {
        info!(
            "Delay task inner topic '{}' already exists, skipping creation",
            DELAY_QUEUE_MESSAGE_TOPIC
        );
        return Ok(());
    }

    let uid = unique_id();
    let topic = Topic {
        topic_id: uid.clone(),
        tenant: DEFAULT_TENANT.to_string(),
        topic_name: DELAY_QUEUE_MESSAGE_TOPIC.to_string(),
        storage_type: StorageType::EngineRocksDB,
        partition: 1,
        replication: 1,
        storage_name_list: Topic::create_partition_name(&uid, 1),
        create_time: now_second(),
    };
    let shard_config = EngineShardConfig {
        replica_num: 1,
        retention_sec: 86400,
        storage_type: StorageType::EngineRocksDB,
        max_segment_size: 1024 * 1024 * 1024,
    };

    create_topic_full(
        broker_cache,
        storage_driver_manager,
        client_pool,
        &topic,
        &shard_config,
    )
    .await?;

    Ok(())
}

pub async fn save_temporary_qos2_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    record: &AdapterWriteRecord,
    tenant: &str,
    topic_name: &str,
    client_id: &str,
    pkid: u16,
) -> Result<Vec<u64>, MqttBrokerError> {
    let message_storage = MessageStorage::new(storage_driver_manager.clone());
    let qos2_message = Qos2TemporaryMessage {
        tenant: tenant.to_string(),
        topic: topic_name.to_string(),
        record: record.clone(),
    };

    let data = serialize::serialize(&qos2_message)?;
    let mut new_record = AdapterWriteRecord::from_bytes(data);
    new_record.key = Some(uniq_key(client_id, pkid));

    let offsets = message_storage
        .append_topic_message(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, vec![new_record])
        .await?;
    return Ok(offsets);
}

pub async fn persistent_save_qos2_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_id: &str,
    pkid: u16,
) -> Result<u64, MqttBrokerError> {
    let key = uniq_key(client_id, pkid);
    let results = storage_driver_manager
        .read_by_key(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, &key)
        .await?;

    if results.is_empty() {
        return Err(MqttBrokerError::CommonError(format!(
            "No temporary QoS2 message found for client_id: {client_id}, pkid: {pkid}"
        )));
    }

    // Find the record with the largest offset (most recent re-publish)
    let record = results
        .iter()
        .max_by_key(|r| r.metadata.offset)
        .ok_or_else(|| {
            MqttBrokerError::CommonError(format!(
                "No temporary QoS2 message found for client_id: {client_id}, pkid: {pkid}"
            ))
        })?
        .clone();

    let qos2_msg: Qos2TemporaryMessage = serialize::deserialize(&record.data)
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let resp = storage_driver_manager
        .write(&qos2_msg.tenant, &qos2_msg.topic, &[qos2_msg.record])
        .await?;

    let write_resp = if let Some(data) = resp.first() {
        data.clone()
    } else {
        return Err(MqttBrokerError::CommonError(format!(
            "Write response is empty when persisting QoS2 message to topic '{}'",
            qos2_msg.topic
        )));
    };

    if write_resp.is_error() {
        return Err(MqttBrokerError::CommonError(write_resp.error_info()));
    }

    Ok(write_resp.offset)
}

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

fn uniq_key(client_id: &str, pkid: u16) -> String {
    format!("{}_{}", client_id, pkid)
}
