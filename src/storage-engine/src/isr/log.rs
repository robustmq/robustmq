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

use crate::commitlog::offset::CommitLogOffset;
use crate::core::error::StorageEngineError;
use async_trait::async_trait;
use metadata_struct::storage::record::StorageRecord;
use std::sync::Arc;

/// Local-log abstraction shared by the memory / rocksdb / filesegment engines.
/// The ISR control plane (append / fetch / truncation) only talks to this trait
/// and is unaware of the engine type; `segment_seq` is always 0 for memory and
/// rocksdb, and the active/follower segment for filesegment.
///
/// Durability comes from replica redundancy (min.insync.replicas) plus the
/// engine's background flush, not a per-write fsync (matching Kafka's default).
/// Ordering still matters: a truncate/clear lowers the LEO before deleting
/// records so a crash never leaves the LEO past the log end, and the local log
/// must stay consistent with the LeaderEpochCache.
#[async_trait]
pub trait ReplicaLog: Send + Sync {
    /// Append records a follower received from the leader. Requires
    /// `base_offset == latest_offset`; a gap returns [`StorageEngineError::OutOfOrder`]
    /// and must trigger the truncation flow.
    async fn append_at(
        &self,
        shard: &str,
        segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError>;

    /// Read up to `max_bytes` starting at `offset`. Used by both follower fetch
    /// and consumer reads. Returns [`StorageEngineError::OffsetOutOfRange`] when
    /// `offset` falls outside `[log_start_offset, latest_offset]`.
    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError>;

    /// The largest offset written to this segment.
    fn latest_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError>;

    /// Truncate to `offset` (inclusive), discarding uncommitted log on a leader
    /// change. Lowers the LEO before deleting records.
    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError>;

    /// Drop all local data for the segment, used when the leader reports the
    /// whole segment was retention-deleted and the follower must re-fetch from
    /// scratch.
    async fn clear(&self, shard: &str, segment_seq: u32) -> Result<(), StorageEngineError>;

    /// The smallest readable offset locally; the follower resets its fetch
    /// offset to this after retention. memory/rocksdb return 0 (or the actual
    /// post-retention start); filesegment returns the current file's start.
    fn log_start_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError>;

    fn commit_log_offset(&self) -> &CommitLogOffset;
}

#[async_trait]
impl ReplicaLog for Arc<dyn ReplicaLog> {
    async fn append_at(
        &self,
        shard: &str,
        segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError> {
        (**self)
            .append_at(shard, segment_seq, base_offset, records)
            .await
    }

    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        (**self)
            .read_from(shard, segment_seq, offset, max_bytes)
            .await
    }

    fn latest_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        (**self).latest_offset(shard, segment_seq)
    }

    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        (**self).truncate_to(shard, segment_seq, offset).await
    }

    async fn clear(&self, shard: &str, segment_seq: u32) -> Result<(), StorageEngineError> {
        (**self).clear(shard, segment_seq).await
    }

    fn log_start_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        (**self).log_start_offset(shard, segment_seq)
    }

    fn commit_log_offset(&self) -> &CommitLogOffset {
        (**self).commit_log_offset()
    }
}
