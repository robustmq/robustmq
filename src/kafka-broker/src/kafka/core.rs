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

use dashmap::DashMap;
use kafka_protocol::messages::fetch_response::{FetchableTopicResponse, PartitionData};
use kafka_protocol::messages::list_offsets_response::{
    ListOffsetsPartitionResponse, ListOffsetsResponse, ListOffsetsTopicResponse,
};
use kafka_protocol::messages::{FetchRequest, FetchResponse, ListOffsetsRequest, ProduceRequest};
use kafka_protocol::records::{
    Compression, Record, RecordBatchEncoder, RecordEncodeOptions, TimestampType,
};
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::tenant::DEFAULT_TENANT;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

pub type ShardOffsets = Arc<DashMap<(u64, String), HashMap<String, u64>>>;

pub fn process_produce(_req: &ProduceRequest) -> Option<KafkaPacket> {
    None
}

pub async fn process_fetch(
    storage_driver_manager: Option<&Arc<StorageDriverManager>>,
    shard_offsets: &ShardOffsets,
    req: &FetchRequest,
    connection_id: u64,
) -> Option<KafkaPacket> {
    let sdm = storage_driver_manager?;

    let read_config = AdapterReadConfig::new();
    let mut topic_responses = Vec::new();

    for fetch_topic in &req.topics {
        let topic_name = fetch_topic.topic.to_string();
        let key = (connection_id, topic_name.clone());

        let mut offsets = shard_offsets
            .get(&key)
            .map(|r| r.clone())
            .unwrap_or_default();

        let records_bytes = match sdm
            .read_by_offset(DEFAULT_TENANT, &topic_name, &offsets, &read_config)
            .await
        {
            Ok(records) if records.is_empty() => None,
            Ok(records) => {
                let mut kafka_records = Vec::new();
                for (i, record) in records.iter().enumerate() {
                    offsets.insert(record.metadata.shard.clone(), record.metadata.offset + 1);
                    kafka_records.push(Record {
                        transactional: false,
                        control: false,
                        partition_leader_epoch: 0,
                        producer_id: -1,
                        producer_epoch: -1,
                        timestamp_type: TimestampType::Creation,
                        offset: record.metadata.offset as i64,
                        sequence: i as i32,
                        timestamp: 0,
                        key: None,
                        value: Some(record.data.clone()),
                        headers: Default::default(),
                    });
                }
                shard_offsets.insert(key, offsets);

                let mut buf = bytes::BytesMut::new();
                let opts = RecordEncodeOptions {
                    version: 2,
                    compression: Compression::None,
                };
                RecordBatchEncoder::encode(&mut buf, kafka_records.iter(), &opts).ok()?;
                Some(buf.freeze())
            }
            Err(e) => {
                warn!("Kafka Fetch storage error for {}: {}", topic_name, e);
                None
            }
        };

        let mut partition_responses = Vec::new();
        for fetch_partition in &fetch_topic.partitions {
            partition_responses.push(
                PartitionData::default()
                    .with_partition_index(fetch_partition.partition)
                    .with_error_code(0)
                    .with_high_watermark(i64::MAX)
                    .with_last_stable_offset(-1)
                    .with_log_start_offset(0)
                    .with_records(records_bytes.clone()),
            );
        }

        topic_responses.push(
            FetchableTopicResponse::default()
                .with_topic(fetch_topic.topic.clone())
                .with_partitions(partition_responses),
        );
    }

    let resp = FetchResponse::default()
        .with_error_code(0)
        .with_session_id(0)
        .with_responses(topic_responses);

    Some(KafkaPacket::FetchResponse(resp))
}

pub fn process_list_offsets(req: &ListOffsetsRequest) -> Option<KafkaPacket> {
    // Return offset=0 for earliest (-2) and the end for latest (-1).
    // We always report offset 0 as both earliest and latest to keep it simple.
    let topics = req
        .topics
        .iter()
        .map(|t| {
            let partitions = t
                .partitions
                .iter()
                .map(|p| {
                    ListOffsetsPartitionResponse::default()
                        .with_partition_index(p.partition_index)
                        .with_error_code(0)
                        .with_offset(0)
                })
                .collect();
            ListOffsetsTopicResponse::default()
                .with_name(t.name.clone())
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::ListOffsetsResponse(
        ListOffsetsResponse::default().with_topics(topics),
    ))
}
