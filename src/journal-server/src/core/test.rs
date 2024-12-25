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

use common_base::config::journal_server::{
    init_journal_server_conf_by_config, journal_server_conf, JournalServerConfig,
};
use common_base::tools::unique_id;
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::segment::{JournalSegment, Replica, SegmentConfig};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use rocksdb_engine::RocksDBEngine;

use super::cache::CacheManager;
use crate::index::engine::{column_family_list, storage_data_fold};
use crate::segment::manager::{try_create_local_segment, SegmentFileManager};
use crate::segment::SegmentIdentity;

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

pub fn test_build_segment() -> SegmentIdentity {
    let namespace = unique_id();
    let shard_name = "s1".to_string();
    let segment_no = 10;

    SegmentIdentity {
        namespace: namespace.clone(),
        shard_name: shard_name.clone(),
        segment_seq: segment_no,
    }
}

pub fn test_build_data_fold() -> Vec<String> {
    let data_fold = vec![format!("/tmp/tests/{}", unique_id())];
    data_fold
}

pub fn test_init_conf() {
    init_journal_server_conf_by_config(JournalServerConfig {
        node_id: 1,
        cluster_name: unique_id(),
        ..Default::default()
    });
}

pub async fn test_init_segment() -> (
    SegmentIdentity,
    Arc<CacheManager>,
    Arc<SegmentFileManager>,
    String,
    Arc<RocksDBEngine>,
) {
    test_init_conf();
    let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();
    let fold = test_build_data_fold().first().unwrap().to_string();
    let segment_file_manager = Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

    let segment = JournalSegment {
        namespace: segment_iden.namespace.clone(),
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment_seq,
        replicas: vec![Replica {
            replica_seq: 0,
            node_id: 1,
            fold: fold.clone(),
        }],
        config: SegmentConfig {
            max_segment_size: 1024 * 1024 * 1024,
        },
        ..Default::default()
    };

    try_create_local_segment(&segment_file_manager, &rocksdb_engine_handler, &segment)
        .await
        .unwrap();

    let cache_manager = Arc::new(CacheManager::new());
    cache_manager.set_segment(segment);

    let conf = journal_server_conf();
    let segment_meta = JournalSegmentMetadata {
        cluster_name: conf.cluster_name.clone(),
        namespace: segment_iden.namespace.clone(),
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment_seq,
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

pub fn test_init_client_pool() -> Arc<ClientPool> {
    Arc::new(ClientPool::new(10))
}
