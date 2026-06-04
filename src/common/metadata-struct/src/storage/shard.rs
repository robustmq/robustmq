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

use common_base::{
    error::common::CommonError, tools::now_second, utils::serialize, uuid::unique_id,
};
use common_config::storage::StorageType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]

pub struct EngineShard {
    pub shard_uid: String,
    pub shard_name: String,
    pub topic_name: String,
    pub start_segment_seq: u32,
    pub active_segment_seq: u32,
    pub last_segment_seq: u32,
    pub status: EngineShardStatus,
    pub config: EngineShardConfig,
    pub desc: String,
    pub create_time: u64,
}

impl EngineShard {
    pub fn new(
        shard_name: String,
        topic_name: String,
        config: EngineShardConfig,
        desc: String,
    ) -> Self {
        EngineShard {
            shard_uid: unique_id(),
            shard_name,
            topic_name,
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            status: EngineShardStatus::Run,
            desc,
            config,
            create_time: now_second(),
        }
    }
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EngineShardStatus {
    #[default]
    Run,
    PrepareDelete,
    Deleting,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EngineShardConfig {
    pub replica_num: u32,
    pub storage_type: StorageType,
    pub max_segment_size: Option<u64>,
    pub max_record_num: Option<u64>,
    pub retention_sec: u64,

    // ISR protocol config, names follow Kafka. Times in ms.
    #[serde(default = "default_min_in_sync_replicas")]
    pub min_in_sync_replicas: u32,
    #[serde(default = "default_replica_lag_time_max_ms")]
    pub replica_lag_time_max_ms: u64,
    #[serde(default = "default_replica_fetch_max_bytes")]
    pub replica_fetch_max_bytes: u64,
    #[serde(default = "default_replica_fetch_wait_max_ms")]
    pub replica_fetch_wait_max_ms: u64,
    #[serde(default = "default_replica_fetch_min_bytes")]
    pub replica_fetch_min_bytes: u64,
    #[serde(default = "default_replica_hw_checkpoint_interval_ms")]
    pub replica_hw_checkpoint_interval_ms: u64,
    #[serde(default = "default_metadata_reconcile_interval_ms")]
    pub metadata_reconcile_interval_ms: u64,
    #[serde(default = "default_reconcile_min_interval_ms")]
    pub reconcile_min_interval_ms: u64,
    #[serde(default = "default_unavailable_recovery_wait_ms")]
    pub unavailable_recovery_wait_ms: u64,
    // Protocol disables unclean election; kept only to hard-reject an override.
    #[serde(default)]
    pub unclean_leader_election_enable: bool,
}

/// 1 GiB (1024 * 1024 * 1024 bytes)
pub const DEFAULT_MAX_SEGMENT_SIZE: u64 = 1073741824;

/// 24 hours in seconds
pub const DEFAULT_RETENTION_SEC: u64 = 7 * 86400;

pub const DEFAULT_MIN_IN_SYNC_REPLICAS: u32 = 1;
pub const DEFAULT_REPLICA_LAG_TIME_MAX_MS: u64 = 30_000;
/// 1 MiB
pub const DEFAULT_REPLICA_FETCH_MAX_BYTES: u64 = 1024 * 1024;
pub const DEFAULT_REPLICA_FETCH_WAIT_MAX_MS: u64 = 500;
pub const DEFAULT_REPLICA_FETCH_MIN_BYTES: u64 = 1;
pub const DEFAULT_REPLICA_HW_CHECKPOINT_INTERVAL_MS: u64 = 5_000;
pub const DEFAULT_METADATA_RECONCILE_INTERVAL_MS: u64 = 30_000;
pub const DEFAULT_RECONCILE_MIN_INTERVAL_MS: u64 = 1_000;
pub const DEFAULT_UNAVAILABLE_RECOVERY_WAIT_MS: u64 = 30_000;

fn default_min_in_sync_replicas() -> u32 {
    DEFAULT_MIN_IN_SYNC_REPLICAS
}
fn default_replica_lag_time_max_ms() -> u64 {
    DEFAULT_REPLICA_LAG_TIME_MAX_MS
}
fn default_replica_fetch_max_bytes() -> u64 {
    DEFAULT_REPLICA_FETCH_MAX_BYTES
}
fn default_replica_fetch_wait_max_ms() -> u64 {
    DEFAULT_REPLICA_FETCH_WAIT_MAX_MS
}
fn default_replica_fetch_min_bytes() -> u64 {
    DEFAULT_REPLICA_FETCH_MIN_BYTES
}
fn default_replica_hw_checkpoint_interval_ms() -> u64 {
    DEFAULT_REPLICA_HW_CHECKPOINT_INTERVAL_MS
}
fn default_metadata_reconcile_interval_ms() -> u64 {
    DEFAULT_METADATA_RECONCILE_INTERVAL_MS
}
fn default_reconcile_min_interval_ms() -> u64 {
    DEFAULT_RECONCILE_MIN_INTERVAL_MS
}
fn default_unavailable_recovery_wait_ms() -> u64 {
    DEFAULT_UNAVAILABLE_RECOVERY_WAIT_MS
}

impl Default for EngineShardConfig {
    fn default() -> Self {
        Self {
            replica_num: 1,
            max_segment_size: Some(DEFAULT_MAX_SEGMENT_SIZE),
            retention_sec: DEFAULT_RETENTION_SEC,
            max_record_num: None,
            storage_type: StorageType::EngineMemory,
            min_in_sync_replicas: DEFAULT_MIN_IN_SYNC_REPLICAS,
            replica_lag_time_max_ms: DEFAULT_REPLICA_LAG_TIME_MAX_MS,
            replica_fetch_max_bytes: DEFAULT_REPLICA_FETCH_MAX_BYTES,
            replica_fetch_wait_max_ms: DEFAULT_REPLICA_FETCH_WAIT_MAX_MS,
            replica_fetch_min_bytes: DEFAULT_REPLICA_FETCH_MIN_BYTES,
            replica_hw_checkpoint_interval_ms: DEFAULT_REPLICA_HW_CHECKPOINT_INTERVAL_MS,
            metadata_reconcile_interval_ms: DEFAULT_METADATA_RECONCILE_INTERVAL_MS,
            reconcile_min_interval_ms: DEFAULT_RECONCILE_MIN_INTERVAL_MS,
            unavailable_recovery_wait_ms: DEFAULT_UNAVAILABLE_RECOVERY_WAIT_MS,
            unclean_leader_election_enable: false,
        }
    }
}

impl EngineShardConfig {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_isr_config_values() {
        let c = EngineShardConfig::default();
        assert_eq!(c.min_in_sync_replicas, DEFAULT_MIN_IN_SYNC_REPLICAS);
        assert_eq!(c.replica_lag_time_max_ms, DEFAULT_REPLICA_LAG_TIME_MAX_MS);
        assert_eq!(c.replica_fetch_max_bytes, DEFAULT_REPLICA_FETCH_MAX_BYTES);
        assert_eq!(
            c.replica_fetch_wait_max_ms,
            DEFAULT_REPLICA_FETCH_WAIT_MAX_MS
        );
        assert_eq!(c.replica_fetch_min_bytes, DEFAULT_REPLICA_FETCH_MIN_BYTES);
        assert_eq!(
            c.replica_hw_checkpoint_interval_ms,
            DEFAULT_REPLICA_HW_CHECKPOINT_INTERVAL_MS
        );
        assert!(!c.unclean_leader_election_enable);
    }

    #[test]
    fn encode_decode_roundtrip_with_isr_config() {
        let c = EngineShardConfig {
            replica_num: 3,
            min_in_sync_replicas: 2,
            replica_lag_time_max_ms: 12_345,
            ..Default::default()
        };
        let decoded = EngineShardConfig::decode(&c.encode().unwrap()).unwrap();
        assert_eq!(decoded.replica_num, 3);
        assert_eq!(decoded.min_in_sync_replicas, 2);
        assert_eq!(decoded.replica_lag_time_max_ms, 12_345);
    }
}
