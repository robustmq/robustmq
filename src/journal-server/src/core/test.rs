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

use super::cache::CacheManager;
use crate::index::engine::{column_family_list, storage_data_fold};
use crate::segment::manager::{create_local_segment, SegmentFileManager};
use crate::segment::write::{create_write_thread, write_data};
use crate::segment::SegmentIdentity;
use common_base::tools::{now_second, unique_id};
use common_config::broker::{broker_config, default_broker_config, init_broker_conf_by_config};
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::segment::{JournalSegment, Replica, SegmentConfig};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use prost::Message;
use protocol::journal::journal_record::JournalRecord;
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
    let namespace = unique_id();
    let shard_name = "s1".to_string();
    let segment_no = 10;

    SegmentIdentity {
        namespace,
        shard_name,
        segment_seq: segment_no,
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

    let cache_manager = Arc::new(CacheManager::new());
    create_local_segment(&cache_manager, &segment_file_manager, &segment)
        .await
        .unwrap();

    let conf = broker_config();
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

#[allow(dead_code)]
pub fn test_init_client_pool() -> Arc<ClientPool> {
    Arc::new(ClientPool::new(10))
}

#[allow(dead_code)]
pub async fn test_base_write_data(
    len: u64,
) -> (
    SegmentIdentity,
    Arc<CacheManager>,
    Arc<SegmentFileManager>,
    String,
    Arc<RocksDBEngine>,
) {
    let (segment_iden, cache_manager, segment_file_manager, fold, rocksdb_engine_handler) =
        test_init_segment().await;

    let res = create_write_thread(
        &cache_manager,
        &rocksdb_engine_handler,
        &segment_file_manager,
        &segment_iden,
    )
    .await;
    assert!(res.is_ok());

    let mut data_list = Vec::new();

    let producer_id = unique_id();
    for i in 0..len {
        data_list.push(JournalRecord {
            namespace: segment_iden.namespace.clone(),
            shard_name: segment_iden.shard_name.clone(),
            segment: segment_iden.segment_seq,
            content: format!("data-{i}").encode_to_vec(),
            key: format!("key-{i}"),
            tags: vec![format!("tag-{}", i)],
            pkid: i,
            create_time: now_second(),
            producer_id: producer_id.clone(),
            ..Default::default()
        });
    }

    let res = write_data(
        &cache_manager,
        &rocksdb_engine_handler,
        &segment_file_manager,
        &segment_iden,
        data_list,
    )
    .await;

    assert!(res.is_ok());

    (
        segment_iden,
        cache_manager,
        segment_file_manager,
        fold,
        rocksdb_engine_handler,
    )
}
