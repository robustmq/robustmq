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
use common_config::storage::StorageType;
use connector::storage::message::MessageStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    mqtt::topic::Topic,
    storage::{adapter_record::AdapterWriteRecord, shard::EngineShardConfig},
    tenant::DEFAULT_TENANT,
};
use node_call::{NodeCallData, NodeCallManager};
use prost::Message;
use protocol::broker::broker::{GetQosDataByClientIdRaw, GetQosDataByClientIdReply};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use storage_adapter::{driver::StorageDriverManager, topic::create_topic_full};
use tracing::info;

pub const DELAY_QUEUE_MESSAGE_TOPIC: &str = "$sys/qos2-inner-topic";

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Qos2TemporaryMessage {
    pub tenant: String,
    pub topic: String,
    pub record: AdapterWriteRecord,
}

pub async fn init_qos2_inner_topic(
    client_pool: &Arc<ClientPool>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    broker_cache: &Arc<NodeCacheManager>,
) -> Result<(), MqttBrokerError> {
    if let Some(topic) = broker_cache.get_topic_by_name(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC) {
        // Topic exists in metadata; ensure the storage shard is also provisioned.
        info!(
            "QoS2 inner topic '{}' already exists, ensuring storage shard is provisioned",
            DELAY_QUEUE_MESSAGE_TOPIC
        );
        let shard_config = EngineShardConfig {
            replica_num: topic.replication,
            storage_type: topic.storage_type,
            max_segment_size: topic.config.max_segment_size,
            max_record_num: topic.config.max_record_num,
            retention_sec: topic.config.retention_sec,
        };
        storage_driver_manager
            .create_storage_resource(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, &shard_config)
            .await
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
        return Ok(());
    }

    let topic = Topic::new(
        DEFAULT_TENANT,
        DELAY_QUEUE_MESSAGE_TOPIC,
        StorageType::EngineRocksDB,
    );

    create_topic_full(broker_cache, storage_driver_manager, client_pool, &topic).await?;
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
    let new_record = AdapterWriteRecord::new(DELAY_QUEUE_MESSAGE_TOPIC.to_string(), data)
        .with_key(uniq_key(client_id, pkid));

    let offsets = message_storage
        .append_topic_message(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, vec![new_record])
        .await?;
    Ok(offsets)
}

pub async fn get_temporary_qos2_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_id: &str,
    pkid: u16,
) -> Result<Option<Qos2TemporaryMessage>, MqttBrokerError> {
    let key = uniq_key(client_id, pkid);
    let results = storage_driver_manager
        .read_by_keys(DEFAULT_TENANT, DELAY_QUEUE_MESSAGE_TOPIC, &[key.as_str()])
        .await?
        .remove(&key)
        .unwrap_or_default();

    if results.is_empty() {
        return Ok(None);
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

    Ok(Some(qos2_msg))
}

pub async fn persistent_save_qos2_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    qos2_msg: Qos2TemporaryMessage,
) -> Result<u64, MqttBrokerError> {
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

pub async fn _try_broadcast_get_pkid(
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
