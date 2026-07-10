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

//! Kafka protocol sentinel values shared across request/response handlers.

/// ListOffsets: return the partition's earliest available offset.
pub const LIST_OFFSETS_EARLIEST_TIMESTAMP: i64 = -2;
/// ListOffsets: return the partition's current high watermark.
pub const LIST_OFFSETS_LATEST_TIMESTAMP: i64 = -1;

/// Universal "no offset" sentinel used across OffsetCommit (client opts out of
/// committing this partition) and OffsetFetch (no committed offset found).
pub const NO_OFFSET: i64 = -1;

/// CreateTopics: use the broker's configured default partition count.
pub const USE_DEFAULT_PARTITIONS: i32 = -1;
/// CreateTopics: use the broker's configured default replication factor.
pub const USE_DEFAULT_REPLICATION_FACTOR: i16 = -1;

/// DeleteRecords: delete up to the partition's current high watermark.
pub const DELETE_RECORDS_HIGH_WATERMARK: i64 = -1;

/// Produce: no base offset (empty/no-op batch).
pub const NO_BASE_OFFSET: i64 = -1;
/// Produce: CreateTime is used for the topic, not LogAppendTime.
pub const NO_LOG_APPEND_TIME: i64 = -1;
/// Produce: acks=0 means the client does not wait for (or want) a response.
pub const PRODUCE_ACKS_NONE: i16 = 0;

/// Fetch: no producer ID (non-transactional / non-idempotent record).
pub const NO_PRODUCER_ID: i64 = -1;
/// Fetch: no producer epoch (non-transactional / non-idempotent record).
pub const NO_PRODUCER_EPOCH: i16 = -1;
/// Fetch: last stable offset unknown (transactions not yet supported).
pub const NO_LAST_STABLE_OFFSET: i64 = -1;

pub const ENDPOINT_TYPE_BROKERS: i8 = 1;
pub const ALL_OPERATIONS_AUTHORIZED: i32 = -1;
