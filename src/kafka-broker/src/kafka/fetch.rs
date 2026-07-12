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
use std::time::{Duration, Instant};

use crate::core::consumer_group_meta::topic_uuid;
use crate::handler::tenant::get_tenant;
use common_config::broker::broker_config;
use futures_util::future::join_all;
use kafka_protocol::error::ResponseError;
use kafka_protocol::indexmap::IndexMap;
use kafka_protocol::messages::fetch_request::{FetchPartition, FetchTopic};
use kafka_protocol::messages::fetch_response::{FetchableTopicResponse, PartitionData};
use kafka_protocol::messages::{FetchRequest, FetchResponse, TopicName};
use kafka_protocol::protocol::StrBytes;
use kafka_protocol::records::{
    Compression, Record, RecordBatchEncoder, RecordEncodeOptions, TimestampType,
};
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::{ArcStorageAdapter, StorageDriverManager};
use tracing::warn;
use uuid::Uuid;

use crate::core::constants::{NO_LAST_STABLE_OFFSET, NO_OFFSET, NO_PRODUCER_EPOCH, NO_PRODUCER_ID};

/// A single requested (topic, partition), either rejected up front or
/// resolved to a concrete shard ready to be read.
enum FetchUnitPlan {
    Error(ResponseError),
    Data {
        driver: ArcStorageAdapter,
        shard_name: String,
        fetch_offset: u64,
        max_bytes: u64,
        high_watermark: i64,
        log_start_offset: i64,
    },
}

struct FetchUnit {
    topic_idx: usize,
    partition_index: i32,
    plan: FetchUnitPlan,
    records: Vec<StorageRecord>,
}

pub async fn process_fetch(
    sdm: &Arc<StorageDriverManager>,
    req: &FetchRequest,
) -> Option<KafkaPacket> {
    let start = Instant::now();

    // Fetch v12+ identifies topics by UUID (the `topic` name is left empty), so
    // resolve each requested topic to its name up front and echo the id back on
    // the response — clients match responses to requests by topic_id at those
    // versions.
    let mut topic_idents: Vec<(TopicName, Uuid)> = Vec::with_capacity(req.topics.len());
    let mut units: Vec<FetchUnit> = Vec::new();
    for (topic_idx, fetch_topic) in req.topics.iter().enumerate() {
        let resolved_name = resolve_fetch_topic_name(sdm, fetch_topic);
        let response_name = resolved_name
            .as_deref()
            .map(|n| TopicName(StrBytes::from_string(n.to_string())))
            .unwrap_or_else(|| fetch_topic.topic.clone());
        topic_idents.push((response_name, fetch_topic.topic_id));
        units.extend(
            resolve_fetch_topic(sdm, topic_idx, fetch_topic, resolved_name.as_deref(), req).await,
        );
    }

    let first_pass = read_all_units(&units).await;
    for (unit, records) in units.iter_mut().zip(first_pass) {
        unit.records = records;
    }

    let min_bytes = req.min_bytes.max(0) as u64;
    let max_wait = Duration::from_millis(req.max_wait_ms.max(0) as u64);
    let total_bytes: u64 = units
        .iter()
        .flat_map(|u| u.records.iter())
        .map(|r| r.data.len() as u64)
        .sum();

    // Long-poll once: if min_bytes isn't met yet, wait up to the remaining
    // max_wait_ms for new data on the still-empty partitions, then re-read
    // exactly once more. We deliberately don't loop re-checking min_bytes
    // against max_wait — one wait-then-reread pass matches what real
    // consumers need and keeps the latency bound simple to reason about.
    if total_bytes < min_bytes {
        let remaining = max_wait.saturating_sub(start.elapsed());
        if !remaining.is_zero() {
            let waiting_indices: Vec<usize> = units
                .iter()
                .enumerate()
                .filter(|(_, u)| {
                    u.records.is_empty() && matches!(u.plan, FetchUnitPlan::Data { .. })
                })
                .map(|(i, _)| i)
                .collect();

            if !waiting_indices.is_empty() {
                let remaining_ms = remaining.as_millis() as u64;
                join_all(waiting_indices.iter().map(|&i| {
                    let engine_storage_handler = sdm.engine_storage_handler.clone();
                    let (_, shard_name, fetch_offset, _) = data_plan_fields(&units[i]);
                    let shard_name = shard_name.to_string();
                    async move {
                        engine_storage_handler
                            .wait_for_new_data(&shard_name, fetch_offset, remaining_ms)
                            .await;
                    }
                }))
                .await;

                let second_pass =
                    join_all(waiting_indices.iter().map(|&i| {
                        let (driver, shard_name, fetch_offset, max_bytes) =
                            data_plan_fields(&units[i]);
                        let driver = driver.clone();
                        let shard_name = shard_name.to_string();
                        async move {
                            read_fetch_unit(&driver, &shard_name, fetch_offset, max_bytes).await
                        }
                    }))
                    .await;

                for (&i, records) in waiting_indices.iter().zip(second_pass) {
                    units[i].records = records;
                }
            }
        }
    }

    let mut topic_responses: Vec<FetchableTopicResponse> = topic_idents
        .into_iter()
        .map(|(name, id)| {
            FetchableTopicResponse::default()
                .with_topic(name)
                .with_topic_id(id)
        })
        .collect();
    let mut per_topic_partitions: Vec<Vec<PartitionData>> = vec![Vec::new(); topic_responses.len()];
    for unit in &units {
        per_topic_partitions[unit.topic_idx].push(build_partition_data(unit));
    }
    for (resp, partitions) in topic_responses.iter_mut().zip(per_topic_partitions) {
        *resp = std::mem::take(resp).with_partitions(partitions);
    }

    let resp = FetchResponse::default()
        .with_error_code(0)
        .with_session_id(0)
        .with_responses(topic_responses);

    Some(KafkaPacket::FetchResponse(resp))
}

fn fetch_partition_error(index: i32, err: ResponseError) -> PartitionData {
    PartitionData::default()
        .with_partition_index(index)
        .with_error_code(err.code())
        .with_high_watermark(NO_OFFSET)
        .with_last_stable_offset(NO_LAST_STABLE_OFFSET)
        .with_log_start_offset(NO_OFFSET)
        .with_records(None)
}

fn effective_max_bytes(req: &FetchRequest, fetch_partition: &FetchPartition) -> u64 {
    let cap = broker_config().kafka_runtime.max_fetch_bytes as u64;
    let requested = if fetch_partition.partition_max_bytes > 0 {
        fetch_partition.partition_max_bytes as u64
    } else if req.max_bytes > 0 {
        req.max_bytes as u64
    } else {
        cap
    };
    requested.min(cap)
}

/// Resolve every partition of one Fetch topic to either an error or a
/// concrete shard, doing the topic/driver lookup and the shard-detail
/// (high_watermark/log_start_offset) lookup exactly once for the whole topic.
/// The topic name a Fetch entry refers to. Pre-v12 clients send the name
/// directly; v12+ clients send only a topic_id (UUID) and leave the name empty,
/// so we reverse the deterministic `topic_uuid` mapping by scanning the tenant's
/// topics. Returns None when neither identifier resolves to a known topic.
fn resolve_fetch_topic_name(
    sdm: &Arc<StorageDriverManager>,
    fetch_topic: &FetchTopic,
) -> Option<String> {
    let name = fetch_topic.topic.to_string();
    if !name.is_empty() {
        return Some(name);
    }

    let id = fetch_topic.topic_id;
    if id == Uuid::nil() {
        return None;
    }
    sdm.broker_cache
        .list_topics_by_tenant(get_tenant())
        .into_iter()
        .find(|t| topic_uuid(get_tenant(), &t.topic_name) == id)
        .map(|t| t.topic_name)
}

async fn resolve_fetch_topic(
    sdm: &Arc<StorageDriverManager>,
    topic_idx: usize,
    fetch_topic: &FetchTopic,
    resolved_name: Option<&str>,
    req: &FetchRequest,
) -> Vec<FetchUnit> {
    let to_error_units = |err: ResponseError| {
        fetch_topic
            .partitions
            .iter()
            .map(|p| FetchUnit {
                topic_idx,
                partition_index: p.partition,
                plan: FetchUnitPlan::Error(err),
                records: Vec::new(),
            })
            .collect::<Vec<_>>()
    };

    let Some(topic_name) = resolved_name else {
        return to_error_units(ResponseError::UnknownTopicOrPartition);
    };

    let Ok((topic, driver)) = sdm.resolve_topic_driver(get_tenant(), topic_name).await else {
        return to_error_units(ResponseError::UnknownTopicOrPartition);
    };

    let details = match sdm.list_storage_resource(get_tenant(), topic_name).await {
        Ok(details) => details,
        Err(e) => {
            warn!(
                "Kafka Fetch failed to list shard details for {}: {}",
                topic_name, e
            );
            return to_error_units(ResponseError::UnknownServerError);
        }
    };

    fetch_topic
        .partitions
        .iter()
        .map(|p| {
            let partition = p.partition as u32;
            let plan = match (
                topic.storage_name_list.get(&partition),
                details.get(&partition),
            ) {
                (Some(shard_name), Some(detail)) => FetchUnitPlan::Data {
                    driver: driver.clone(),
                    shard_name: shard_name.clone(),
                    fetch_offset: p.fetch_offset.max(0) as u64,
                    max_bytes: effective_max_bytes(req, p),
                    high_watermark: detail.offset.high_watermark as i64,
                    log_start_offset: detail.offset.start_offset as i64,
                },
                _ => FetchUnitPlan::Error(ResponseError::UnknownTopicOrPartition),
            };
            FetchUnit {
                topic_idx,
                partition_index: p.partition,
                plan,
                records: Vec::new(),
            }
        })
        .collect()
}

async fn read_fetch_unit(
    driver: &ArcStorageAdapter,
    shard_name: &str,
    fetch_offset: u64,
    max_bytes: u64,
) -> Vec<StorageRecord> {
    let read_config = AdapterReadConfig {
        max_record_num: u64::MAX,
        max_size: max_bytes,
    };
    match driver
        .read_by_offset(shard_name, fetch_offset, &read_config)
        .await
    {
        Ok(records) => records,
        Err(e) => {
            warn!("Kafka Fetch read failed for shard {}: {}", shard_name, e);
            Vec::new()
        }
    }
}

/// Pull the shard details out of a unit known to hold `FetchUnitPlan::Data`
/// (callers filter to such units first, e.g. via `waiting_indices`).
fn data_plan_fields(unit: &FetchUnit) -> (&ArcStorageAdapter, &str, u64, u64) {
    match &unit.plan {
        FetchUnitPlan::Data {
            driver,
            shard_name,
            fetch_offset,
            max_bytes,
            ..
        } => (driver, shard_name, *fetch_offset, *max_bytes),
        FetchUnitPlan::Error(_) => unreachable!("caller guarantees this unit holds Data"),
    }
}

async fn read_all_units(units: &[FetchUnit]) -> Vec<Vec<StorageRecord>> {
    join_all(units.iter().map(|u| async move {
        match &u.plan {
            FetchUnitPlan::Data {
                driver,
                shard_name,
                fetch_offset,
                max_bytes,
                ..
            } => read_fetch_unit(driver, shard_name, *fetch_offset, *max_bytes).await,
            FetchUnitPlan::Error(_) => Vec::new(),
        }
    }))
    .await
}

fn kafka_record_from_storage(sequence: i32, record: &StorageRecord) -> Record {
    let headers: IndexMap<StrBytes, Option<bytes::Bytes>> = record
        .metadata
        .header
        .as_ref()
        .map(|hs| {
            hs.iter()
                .map(|h| {
                    (
                        StrBytes::from(h.name.clone()),
                        Some(bytes::Bytes::from(h.value.clone())),
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    Record {
        transactional: false,
        control: false,
        partition_leader_epoch: 0,
        producer_id: NO_PRODUCER_ID,
        producer_epoch: NO_PRODUCER_EPOCH,
        timestamp_type: TimestampType::Creation,
        offset: record.metadata.offset as i64,
        sequence,
        timestamp: record.metadata.create_t as i64,
        key: record.metadata.key.clone(),
        value: Some(record.data.clone()),
        headers,
    }
}

fn encode_fetch_records(shard_name: &str, records: &[StorageRecord]) -> Option<bytes::Bytes> {
    if records.is_empty() {
        return None;
    }
    let kafka_records: Vec<Record> = records
        .iter()
        .enumerate()
        .map(|(i, r)| kafka_record_from_storage(i as i32, r))
        .collect();

    let mut buf = bytes::BytesMut::new();
    let opts = RecordEncodeOptions {
        version: 2,
        compression: Compression::None,
    };
    match RecordBatchEncoder::encode(&mut buf, kafka_records.iter(), &opts) {
        Ok(()) => Some(buf.freeze()),
        Err(e) => {
            warn!(
                "Kafka Fetch failed to encode record batch for shard {}: {}",
                shard_name, e
            );
            None
        }
    }
}

/// A shard read only turns into an error when there *were* records to encode
/// and the encode itself failed; "nothing new to fetch" is a normal success.
fn partition_error_code(has_records: bool, encode_succeeded: bool) -> i16 {
    if has_records && !encode_succeeded {
        ResponseError::UnknownServerError.code()
    } else {
        0
    }
}

fn build_partition_data(unit: &FetchUnit) -> PartitionData {
    match &unit.plan {
        FetchUnitPlan::Error(err) => fetch_partition_error(unit.partition_index, *err),
        FetchUnitPlan::Data {
            shard_name,
            high_watermark,
            log_start_offset,
            ..
        } => {
            let records_bytes = encode_fetch_records(shard_name, &unit.records);
            let error_code =
                partition_error_code(!unit.records.is_empty(), records_bytes.is_some());
            PartitionData::default()
                .with_partition_index(unit.partition_index)
                .with_error_code(error_code)
                .with_high_watermark(*high_watermark)
                .with_last_stable_offset(NO_LAST_STABLE_OFFSET)
                .with_log_start_offset(*log_start_offset)
                .with_records(records_bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kafka_protocol::records::RecordBatchDecoder;

    #[test]
    fn fetch_partition_error_sets_sentinels() {
        let resp = fetch_partition_error(1, ResponseError::UnknownTopicOrPartition);
        assert_eq!(resp.partition_index, 1);
        assert_eq!(
            resp.error_code,
            ResponseError::UnknownTopicOrPartition.code()
        );
        assert_eq!(resp.high_watermark, NO_OFFSET);
        assert_eq!(resp.log_start_offset, NO_OFFSET);
        assert_eq!(resp.last_stable_offset, NO_LAST_STABLE_OFFSET);
        assert!(resp.records.is_none());
    }

    #[test]
    fn partition_error_code_only_errors_when_records_present_but_unencoded() {
        assert_eq!(partition_error_code(false, false), 0);
        assert_eq!(partition_error_code(false, true), 0);
        assert_eq!(partition_error_code(true, true), 0);
        assert_eq!(
            partition_error_code(true, false),
            ResponseError::UnknownServerError.code()
        );
    }

    fn make_storage_record(offset: u64, key: Option<&[u8]>, data: &[u8]) -> StorageRecord {
        let mut metadata = metadata_struct::storage::record::StorageRecordMetadata::build(
            offset,
            "shard".to_string(),
            0,
        )
        .with_create_t(1234);
        if let Some(k) = key {
            metadata = metadata.with_key(Some(bytes::Bytes::copy_from_slice(k)));
        }
        StorageRecord {
            metadata,
            protocol_data: None,
            data: bytes::Bytes::copy_from_slice(data),
        }
    }

    #[test]
    fn kafka_record_from_storage_round_trips_key_value_and_timestamp() {
        let record = make_storage_record(7, Some(b"\xffbin-key"), b"payload");
        let kafka_record = kafka_record_from_storage(0, &record);

        assert_eq!(kafka_record.offset, 7);
        assert_eq!(kafka_record.value.as_deref(), Some(b"payload".as_ref()));
        assert_eq!(kafka_record.key.as_deref(), Some(b"\xffbin-key".as_ref()));
        assert_eq!(kafka_record.timestamp, 1234);
    }

    #[test]
    fn kafka_record_from_storage_converts_headers() {
        let mut record = make_storage_record(0, None, b"payload");
        record.metadata.header = Some(vec![metadata_struct::storage::record::StorageHeader {
            name: "trace-id".to_string(),
            value: "abc".to_string(),
        }]);

        let kafka_record = kafka_record_from_storage(0, &record);

        let (name, value) = kafka_record.headers.iter().next().expect("one header");
        assert_eq!(name.as_str(), "trace-id");
        assert_eq!(value.as_deref(), Some(b"abc".as_ref()));
    }

    #[test]
    fn encode_fetch_records_returns_none_for_empty_input() {
        assert!(encode_fetch_records("shard", &[]).is_none());
    }

    #[test]
    fn encode_fetch_records_round_trips_via_decode() {
        let records = vec![
            make_storage_record(0, None, b"one"),
            make_storage_record(1, Some(b"k"), b"two"),
        ];
        let encoded = encode_fetch_records("shard", &records).expect("non-empty input encodes");

        let mut buf = encoded;
        let batches = RecordBatchDecoder::decode_all(&mut buf).unwrap();
        let decoded: Vec<_> = batches.into_iter().flat_map(|b| b.records).collect();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].value.as_deref(), Some(b"one".as_ref()));
        assert_eq!(decoded[1].value.as_deref(), Some(b"two".as_ref()));
        assert_eq!(decoded[1].key.as_deref(), Some(b"k".as_ref()));
    }

    fn ensure_test_broker_config() {
        use common_config::broker::{default_broker_config, init_broker_conf_by_config};
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            init_broker_conf_by_config(default_broker_config());
        });
    }

    #[test]
    fn effective_max_bytes_caps_at_broker_limit() {
        ensure_test_broker_config();
        let req = FetchRequest::default().with_max_bytes(i32::MAX);
        let partition = FetchPartition::default().with_partition_max_bytes(i32::MAX);
        let cap = broker_config().kafka_runtime.max_fetch_bytes as u64;
        assert_eq!(effective_max_bytes(&req, &partition), cap);
    }

    #[test]
    fn effective_max_bytes_selects_by_priority() {
        ensure_test_broker_config();
        let broker_cap = broker_config().kafka_runtime.max_fetch_bytes as u64;

        // partition-level wins even when the request-level value differs.
        let req = FetchRequest::default().with_max_bytes(999);
        let partition = FetchPartition::default().with_partition_max_bytes(500);
        assert_eq!(effective_max_bytes(&req, &partition), 500);

        // falls back to request-level when partition-level is unset.
        let req = FetchRequest::default().with_max_bytes(2048);
        let partition = FetchPartition::default();
        assert_eq!(effective_max_bytes(&req, &partition), 2048);

        // falls back to the broker default when neither is set.
        let req = FetchRequest::default();
        let partition = FetchPartition::default();
        assert_eq!(effective_max_bytes(&req, &partition), broker_cap);
    }
}
