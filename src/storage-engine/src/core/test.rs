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
use crate::core::record::{StorageEngineRecord, StorageEngineRecordMetadata};
use crate::segment::index::engine::{column_family_list, storage_data_fold};
use crate::segment::manager::{create_local_segment, SegmentFileManager};
use crate::segment::SegmentIdentity;
use broker_core::cache::BrokerCacheManager;
use bytes::Bytes;
use common_base::tools::{now_second, unique_id};
use common_config::broker::{default_broker_config, init_broker_conf_by_config};
use common_config::config::BrokerConfig;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::{EngineSegment, Replica};
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[allow(dead_code)]
pub fn test_build_rocksdb_sgement() -> (Arc<RocksDBEngine>, SegmentIdentity) {
    let data_fold = test_build_data_fold();
    let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
        &storage_data_fold(&data_fold),
        10000,
        column_family_list(),
    ));

    let segment_iden = test_build_segment();
    (rocksdb_engine_handler, segment_iden)
}

#[allow(dead_code)]
pub fn test_build_segment() -> SegmentIdentity {
    let shard_name = "s1".to_string();
    let segment_no = 10;

    SegmentIdentity {
        shard_name,
        segment: segment_no,
    }
}

#[allow(dead_code)]
pub fn test_build_data_fold() -> Vec<String> {
    let data_fold = vec![format!("/tmp/tests/{}", unique_id())];
    data_fold
}

#[allow(dead_code)]
pub fn test_init_conf() {
    init_broker_conf_by_config(default_broker_config());
}

#[allow(dead_code)]
pub async fn test_init_segment() -> (
    SegmentIdentity,
    Arc<StorageCacheManager>,
    Arc<SegmentFileManager>,
    String,
    Arc<RocksDBEngine>,
) {
    test_init_conf();
    let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();
    let fold = test_build_data_fold().first().unwrap().to_string();
    let segment_file_manager = Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

    let segment = EngineSegment {
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment,
        replicas: vec![Replica {
            replica_seq: 0,
            node_id: 1,
            fold: fold.clone(),
        }],
        ..Default::default()
    };
    let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
    let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
    create_local_segment(&cache_manager, &segment_file_manager, &segment)
        .await
        .unwrap();

    let segment_meta = EngineSegmentMetadata {
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment,
        ..Default::default()
    };
    cache_manager.set_segment_meta(segment_meta);

    (
        segment_iden,
        cache_manager,
        segment_file_manager,
        fold,
        rocksdb_engine_handler,
    )
}

#[allow(dead_code)]
pub fn test_init_client_pool() -> Arc<ClientPool> {
    Arc::new(ClientPool::new(10))
}

#[allow(dead_code)]
pub async fn test_base_write_data(
    len: u64,
) -> (
    SegmentIdentity,
    Arc<StorageCacheManager>,
    Arc<SegmentFileManager>,
    String,
    Arc<RocksDBEngine>,
) {
    let (segment_iden, cache_manager, segment_file_manager, fold, rocksdb_engine_handler) =
        test_init_segment().await;

    let mut data_list = Vec::new();

    for i in 0..len {
        data_list.push(StorageEngineRecord {
            metadata: StorageEngineRecordMetadata {
                offset: 1000 + i,
                key: None,
                tags: None,
                shard: segment_iden.shard_name.to_string(),
                segment: segment_iden.segment,
                create_t: now_second(),
            },
            data: Bytes::from(format!("data-{i}")),
        });
    }

    // let res = write_(
    //     &cache_manager,
    //     &rocksdb_engine_handler,
    //     &segment_file_manager,
    //     &segment_iden,
    //     &data_list,
    // )
    // .await;

    // assert!(res.is_ok());

    (
        segment_iden,
        cache_manager,
        segment_file_manager,
        fold,
        rocksdb_engine_handler,
    )
}
