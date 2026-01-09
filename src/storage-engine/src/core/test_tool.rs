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

use super::cache::StorageCacheManager;
use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::offset::CommitLogOffset;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::segment::create_local_segment;
use crate::filesegment::offset::FileSegmentOffset;
use crate::filesegment::SegmentIdentity;
use broker_core::cache::BrokerCacheManager;
use common_base::tools::{now_second, unique_id};
use common_config::broker::{default_broker_config, init_broker_conf_by_config};
use common_config::config::BrokerConfig;
use common_config::storage::StorageType;
use metadata_struct::storage::segment::{EngineSegment, Replica, SegmentStatus};
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::{EngineShard, EngineShardConfig, EngineShardStatus};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::test::test_rocksdb_instance;
use std::sync::Arc;

#[allow(dead_code)]
pub fn test_build_segment() -> SegmentIdentity {
    SegmentIdentity {
        shard_name: unique_id(),
        segment: 0,
    }
}

#[allow(dead_code)]
pub fn test_build_data_fold() -> Vec<String> {
    vec![format!("/tmp/tests/{}", unique_id())]
}

#[allow(dead_code)]
pub fn test_init_conf() {
    init_broker_conf_by_config(default_broker_config());
}

#[allow(dead_code)]
pub async fn test_init_segment(
    engine_storage_type: StorageType,
) -> (
    SegmentIdentity,
    Arc<StorageCacheManager>,
    String,
    Arc<RocksDBEngine>,
) {
    test_init_conf();
    let rocksdb_engine_handler = test_rocksdb_instance();
    let segment_iden = test_build_segment();
    let fold = test_build_data_fold().first().unwrap().to_string();
    let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(BrokerCacheManager::new(
        BrokerConfig::default(),
    ))));

    let shard = EngineShard {
        shard_uid: unique_id(),
        shard_name: segment_iden.shard_name.clone(),
        start_segment_seq: 0,
        active_segment_seq: 0,
        last_segment_seq: 0,
        status: EngineShardStatus::Run,
        config: EngineShardConfig {
            retention_sec: 10,
            storage_type: engine_storage_type,
            ..Default::default()
        },
        create_time: now_second(),
    };
    cache_manager.set_shard(shard);

    let segment = EngineSegment {
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment,
        replicas: vec![Replica {
            replica_seq: 0,
            node_id: 1,
            fold: fold.clone(),
        }],
        leader: 1,
        leader_epoch: 0,
        status: SegmentStatus::Write,
        isr: vec![1],
    };

    create_local_segment(&cache_manager, &segment)
        .await
        .unwrap();

    cache_manager.set_segment_meta(EngineSegmentMetadata {
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment,
        start_offset: 0,
        ..Default::default()
    });
    cache_manager.sort_offset_index(&segment_iden.shard_name);

    if engine_storage_type == StorageType::EngineMemory
        || engine_storage_type == StorageType::EngineRocksDB
    {
        let commit_offset =
            CommitLogOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
        commit_offset
            .save_earliest_offset(&segment_iden.shard_name, 0)
            .unwrap();
        commit_offset
            .save_latest_offset(&segment_iden.shard_name, 0)
            .unwrap();
    }

    if engine_storage_type == StorageType::EngineSegment {
        let commit_offset =
            FileSegmentOffset::new(rocksdb_engine_handler.clone(), cache_manager.clone());
        commit_offset
            .segment_offset
            .save_start_offset(&segment_iden, 0)
            .unwrap();
        commit_offset
            .segment_offset
            .save_end_offset(&segment_iden, 0)
            .unwrap();

        commit_offset
            .segment_offset
            .save_start_timestamp(&segment_iden, 0)
            .unwrap();
        commit_offset
            .segment_offset
            .save_end_timestamp(&segment_iden, 0)
            .unwrap();
    }

    (segment_iden, cache_manager, fold, rocksdb_engine_handler)
}

pub fn test_build_rocksdb_engine() -> RocksDBStorageEngine {
    let db = test_rocksdb_instance();
    let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(BrokerCacheManager::new(
        BrokerConfig::default(),
    ))));

    RocksDBStorageEngine::new(cache_manager, db)
}

pub fn test_build_memory_engine() -> MemoryStorageEngine {
    let db = test_rocksdb_instance();
    let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(BrokerCacheManager::new(
        BrokerConfig::default(),
    ))));
    let config = common_config::storage::memory::StorageDriverMemoryConfig::default();

    MemoryStorageEngine::new(db, cache_manager, config)
}
