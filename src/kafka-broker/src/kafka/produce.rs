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

use crate::core::cache::{KafkaCacheManager, SequenceCheck};
use crate::handler::tenant::get_tenant;
use common_base::error::common::CommonError;
use futures_util::future::join_all;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::produce_request::{PartitionProduceData, TopicProduceData};
use kafka_protocol::messages::produce_response::{PartitionProduceResponse, TopicProduceResponse};
use kafka_protocol::messages::{ProduceRequest, ProduceResponse};
use kafka_protocol::records::{Record, RecordBatchDecoder};
use metadata_struct::adapter::adapter_read_config::AdapterWriteRespRow;
use metadata_struct::adapter::adapter_record::{AdapterWriteRecord, RecordHeader};
use metadata_struct::topic::Topic;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::{ArcStorageAdapter, StorageDriverManager};
use tracing::warn;

use crate::core::constants::{
    NO_BASE_OFFSET, NO_LOG_APPEND_TIME, NO_PRODUCER_ID, PRODUCE_ACKS_NONE, VALID_ACKS,
};

pub async fn process_produce(
    sdm: &Arc<StorageDriverManager>,
    cache: &Arc<KafkaCacheManager>,
    req: &ProduceRequest,
) -> Option<KafkaPacket> {
    if !VALID_ACKS.contains(&req.acks) {
        return Some(KafkaPacket::ProduceResponse(produce_error_response(
            req,
            ResponseError::InvalidRequiredAcks,
        )));
    }

    // Transactions aren't implemented (only KIP-98 idempotence is): reject a
    // transactional produce rather than accept it with no atomicity guarantee.
    // Idempotent producers send a null transactional_id and are handled below.
    if req.transactional_id.is_some() {
        return Some(KafkaPacket::ProduceResponse(produce_error_response(
            req,
            ResponseError::TransactionalIdAuthorizationFailed,
        )));
    }

    let topic_responses = join_all(
        req.topic_data
            .iter()
            .map(|topic_data| produce_to_topic(sdm, cache, topic_data, req.acks)),
    )
    .await;

    if req.acks == PRODUCE_ACKS_NONE {
        return None;
    }

    Some(KafkaPacket::ProduceResponse(
        ProduceResponse::default().with_responses(topic_responses),
    ))
}

fn produce_partition_error(index: i32, err: ResponseError) -> PartitionProduceResponse {
    PartitionProduceResponse::default()
        .with_index(index)
        .with_error_code(err.code())
        .with_base_offset(NO_BASE_OFFSET)
        .with_log_append_time_ms(NO_LOG_APPEND_TIME)
}

fn produce_partition_ok(index: i32, base_offset: i64) -> PartitionProduceResponse {
    PartitionProduceResponse::default()
        .with_index(index)
        .with_error_code(0)
        .with_base_offset(base_offset)
        .with_log_append_time_ms(NO_LOG_APPEND_TIME)
}

/// Decoded produce payload plus the idempotence identity carried on the batch
/// (each decoded `Record` inherits the batch's producer id / base sequence).
struct DecodedProduce {
    records: Vec<AdapterWriteRecord>,
    producer_id: i64,
    base_sequence: i32,
}

fn decode_produce_records(topic_name: &str, records: &bytes::Bytes) -> Option<DecodedProduce> {
    let mut buf = records.clone();
    let batches = match RecordBatchDecoder::decode_all(&mut buf) {
        Ok(batches) => batches,
        Err(e) => {
            warn!(
                "Kafka Produce failed to decode record batch for {}: {}",
                topic_name, e
            );
            return None;
        }
    };

    let kafka_records: Vec<Record> = batches
        .into_iter()
        .flat_map(|batch| batch.records)
        .collect();
    let (producer_id, base_sequence) = kafka_records
        .first()
        .map(|r| (r.producer_id, r.sequence))
        .unwrap_or((NO_PRODUCER_ID, 0));

    Some(DecodedProduce {
        records: kafka_records
            .into_iter()
            .map(|record| adapter_record_from_kafka(topic_name, record))
            .collect(),
        producer_id,
        base_sequence,
    })
}

fn adapter_record_from_kafka(topic_name: &str, record: Record) -> AdapterWriteRecord {
    let mut adapter_record = AdapterWriteRecord::new(topic_name, record.value.unwrap_or_default());

    if let Some(key) = record.key {
        adapter_record = adapter_record.with_key(key);
    }

    let headers: Vec<RecordHeader> = record
        .headers
        .into_iter()
        .map(|(name, value)| RecordHeader {
            name: name.to_string(),
            value: value
                .map(|v| String::from_utf8_lossy(&v).into_owned())
                .unwrap_or_default(),
        })
        .collect();
    if !headers.is_empty() {
        adapter_record = adapter_record.with_header(headers);
    }

    adapter_record
}

async fn produce_to_partition(
    driver: &ArcStorageAdapter,
    cache: &Arc<KafkaCacheManager>,
    topic: &Topic,
    topic_name: &str,
    partition_data: &PartitionProduceData,
    acks: i16,
) -> PartitionProduceResponse {
    if partition_data.index < 0 || partition_data.index as u32 >= topic.partition {
        return produce_partition_error(
            partition_data.index,
            ResponseError::UnknownTopicOrPartition,
        );
    }

    let Some(records) = &partition_data.records else {
        return produce_partition_ok(partition_data.index, NO_BASE_OFFSET);
    };

    let Some(decoded) = decode_produce_records(topic_name, records) else {
        return produce_partition_error(partition_data.index, ResponseError::CorruptMessage);
    };

    if decoded.records.is_empty() {
        return produce_partition_ok(partition_data.index, NO_BASE_OFFSET);
    }

    // Shard for this partition was already resolved once at the topic level
    // (see `produce_to_topic`), so no per-partition topic/driver lookup here.
    let Some(shard_name) = topic.storage_name_list.get(&(partition_data.index as u32)) else {
        return produce_partition_error(
            partition_data.index,
            ResponseError::UnknownTopicOrPartition,
        );
    };

    // Idempotent producers (producer_id >= 0) tag each batch with a base
    // sequence; dedup exact retries and reject gaps so a resend can't create
    // duplicate records. Non-idempotent produce (producer_id < 0) skips this.
    let idempotent = decoded.producer_id >= 0;
    if idempotent {
        match cache.check_producer_sequence(decoded.producer_id, shard_name, decoded.base_sequence)
        {
            SequenceCheck::Accept => {}
            SequenceCheck::Duplicate(base_offset) => {
                return produce_partition_ok(partition_data.index, base_offset);
            }
            SequenceCheck::OutOfOrder => {
                return produce_partition_error(
                    partition_data.index,
                    ResponseError::OutOfOrderSequenceNumber,
                );
            }
        }
    }

    let result = driver.write(shard_name, &decoded.records, acks as i8).await;

    if idempotent {
        if let Ok(rows) = &result {
            if !rows.iter().any(|r| r.is_error()) {
                if let Some(base_offset) = rows.first().map(|r| r.offset as i64) {
                    let last_seq = decoded.base_sequence + decoded.records.len() as i32 - 1;
                    cache.record_producer_sequence(
                        decoded.producer_id,
                        shard_name,
                        decoded.base_sequence,
                        last_seq,
                        base_offset,
                    );
                }
            }
        }
    }

    build_produce_response(topic_name, partition_data.index, result)
}

fn build_produce_response(
    topic_name: &str,
    index: i32,
    result: Result<Vec<AdapterWriteRespRow>, CommonError>,
) -> PartitionProduceResponse {
    match result {
        Ok(rows) => {
            if let Some(failed) = rows.iter().find(|r| r.is_error()) {
                warn!(
                    "Kafka Produce partial write failure for {}[{}]: {}",
                    topic_name,
                    index,
                    failed.error_info()
                );
                return produce_partition_error(index, ResponseError::UnknownServerError);
            }
            let base_offset = rows.first().map_or(NO_BASE_OFFSET, |r| r.offset as i64);
            produce_partition_ok(index, base_offset)
        }
        Err(e) => {
            warn!(
                "Kafka Produce write failed for {}[{}]: {}",
                topic_name, index, e
            );
            produce_partition_error(index, ResponseError::UnknownServerError)
        }
    }
}

async fn produce_to_topic(
    sdm: &Arc<StorageDriverManager>,
    cache: &Arc<KafkaCacheManager>,
    topic_data: &TopicProduceData,
    acks: i16,
) -> TopicProduceResponse {
    let topic_name = topic_data.name.to_string();

    // Resolve the topic and its storage driver once per topic (not once per
    // partition): both `get_topic_by_name` (clones the whole `Topic`,
    // including its per-partition shard-name map) and driver lookup are not
    // free, and every partition in this request shares the same topic.
    let Ok((topic, driver)) = sdm.resolve_topic_driver(get_tenant(), &topic_name).await else {
        let partitions = topic_data
            .partition_data
            .iter()
            .map(|p| produce_partition_error(p.index, ResponseError::UnknownTopicOrPartition))
            .collect();
        return TopicProduceResponse::default()
            .with_name(topic_data.name.clone())
            .with_partition_responses(partitions);
    };

    // Partitions are independent shards, so write them concurrently instead
    // of one at a time.
    let partitions = join_all(
        topic_data
            .partition_data
            .iter()
            .map(|p| produce_to_partition(&driver, cache, &topic, &topic_name, p, acks)),
    )
    .await;

    TopicProduceResponse::default()
        .with_name(topic_data.name.clone())
        .with_partition_responses(partitions)
}

/// Reject every partition in the request with the same error, without
/// touching storage (used for request-level validation failures like a bad
/// `acks` value or an unsupported transactional produce).
fn produce_error_response(req: &ProduceRequest, err: ResponseError) -> ProduceResponse {
    let topic_responses = req
        .topic_data
        .iter()
        .map(|topic_data| {
            let partitions = topic_data
                .partition_data
                .iter()
                .map(|p| produce_partition_error(p.index, err))
                .collect();
            TopicProduceResponse::default()
                .with_name(topic_data.name.clone())
                .with_partition_responses(partitions)
        })
        .collect();
    ProduceResponse::default().with_responses(topic_responses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::{NO_PRODUCER_EPOCH, NO_PRODUCER_ID};
    use kafka_protocol::records::{
        Compression, RecordBatchEncoder, RecordEncodeOptions, TimestampType,
    };

    #[test]
    fn build_produce_response_uses_first_row_as_base_offset() {
        let rows = vec![
            AdapterWriteRespRow {
                offset: 10,
                ..Default::default()
            },
            AdapterWriteRespRow {
                offset: 11,
                ..Default::default()
            },
        ];
        let resp = build_produce_response("t", 0, Ok(rows));
        assert_eq!(resp.error_code, 0);
        assert_eq!(resp.base_offset, 10);
        assert_eq!(resp.log_append_time_ms, NO_LOG_APPEND_TIME);
    }

    #[test]
    fn build_produce_response_reports_error_when_any_row_failed() {
        let rows = vec![
            AdapterWriteRespRow {
                offset: 10,
                ..Default::default()
            },
            AdapterWriteRespRow {
                offset: 11,
                error: Some("boom".to_string()),
                ..Default::default()
            },
        ];
        let resp = build_produce_response("t", 0, Ok(rows));
        assert_eq!(resp.error_code, ResponseError::UnknownServerError.code());
        assert_eq!(resp.base_offset, NO_BASE_OFFSET);
    }

    #[test]
    fn build_produce_response_reports_error_on_write_failure() {
        let resp = build_produce_response(
            "t",
            0,
            Err(CommonError::CommonError("write failed".to_string())),
        );
        assert_eq!(resp.error_code, ResponseError::UnknownServerError.code());
        assert_eq!(resp.base_offset, NO_BASE_OFFSET);
    }

    fn produce_request_with_one_partition(acks: i16) -> ProduceRequest {
        use kafka_protocol::messages::TopicName;
        use kafka_protocol::protocol::StrBytes;

        ProduceRequest::default()
            .with_acks(acks)
            .with_topic_data(vec![TopicProduceData::default()
                .with_name(TopicName(StrBytes::from_static_str("t")))
                .with_partition_data(
                    vec![PartitionProduceData::default().with_index(0)],
                )])
    }

    #[test]
    fn produce_error_response_applies_error_to_every_partition() {
        let req = produce_request_with_one_partition(2);
        let resp = produce_error_response(&req, ResponseError::InvalidRequiredAcks);
        assert_eq!(resp.responses.len(), 1);
        let partition = &resp.responses[0].partition_responses[0];
        assert_eq!(
            partition.error_code,
            ResponseError::InvalidRequiredAcks.code()
        );
    }

    #[test]
    fn adapter_record_from_kafka_carries_key_value_and_headers() {
        use kafka_protocol::indexmap::IndexMap;
        use kafka_protocol::protocol::StrBytes;

        let mut headers = IndexMap::new();
        headers.insert(
            StrBytes::from_static_str("x-trace"),
            Some(bytes::Bytes::from_static(b"abc")),
        );

        let record = Record {
            transactional: false,
            control: false,
            partition_leader_epoch: 0,
            producer_id: NO_PRODUCER_ID,
            producer_epoch: NO_PRODUCER_EPOCH,
            timestamp_type: TimestampType::Creation,
            offset: 0,
            sequence: 0,
            timestamp: 0,
            key: Some(bytes::Bytes::from_static(b"\xff\x00binary-key")),
            value: Some(bytes::Bytes::from_static(b"payload")),
            headers,
        };

        let adapter_record = adapter_record_from_kafka("my-topic", record);
        assert_eq!(adapter_record.topic, "my-topic");
        assert_eq!(adapter_record.data.as_ref(), b"payload");
        assert_eq!(
            adapter_record.key(),
            Some(b"\xff\x00binary-key".as_ref()),
            "binary keys must round-trip losslessly"
        );
        assert_eq!(adapter_record.header().len(), 1);
        assert_eq!(adapter_record.header()[0].name, "x-trace");
        assert_eq!(adapter_record.header()[0].value, "abc");
    }

    #[test]
    fn decode_produce_records_round_trips_a_batch() {
        let records = [
            Record {
                transactional: false,
                control: false,
                partition_leader_epoch: 0,
                producer_id: NO_PRODUCER_ID,
                producer_epoch: NO_PRODUCER_EPOCH,
                timestamp_type: TimestampType::Creation,
                offset: 0,
                sequence: 0,
                timestamp: 0,
                key: None,
                value: Some(bytes::Bytes::from_static(b"one")),
                headers: Default::default(),
            },
            Record {
                transactional: false,
                control: false,
                partition_leader_epoch: 0,
                producer_id: NO_PRODUCER_ID,
                producer_epoch: NO_PRODUCER_EPOCH,
                timestamp_type: TimestampType::Creation,
                offset: 1,
                sequence: 1,
                timestamp: 0,
                key: Some(bytes::Bytes::from_static(b"k")),
                value: Some(bytes::Bytes::from_static(b"two")),
                headers: Default::default(),
            },
        ];

        let mut buf = bytes::BytesMut::new();
        let opts = RecordEncodeOptions {
            version: 2,
            compression: Compression::None,
        };
        RecordBatchEncoder::encode(&mut buf, records.iter(), &opts).unwrap();

        let decoded = decode_produce_records("my-topic", &buf.freeze()).unwrap();
        assert_eq!(decoded.records.len(), 2);
        assert_eq!(decoded.records[0].data.as_ref(), b"one");
        assert_eq!(decoded.records[0].key(), None);
        assert_eq!(decoded.records[1].data.as_ref(), b"two");
        assert_eq!(decoded.records[1].key(), Some(b"k".as_ref()));
    }

    #[test]
    fn decode_produce_records_rejects_garbage() {
        let garbage = bytes::Bytes::from_static(b"not a valid record batch");
        assert!(decode_produce_records("my-topic", &garbage).is_none());
    }
}
