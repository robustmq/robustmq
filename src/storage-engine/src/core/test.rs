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
use crate::core::segment::create_local_segment;
use crate::segment::write::{WriteChannelDataRecord, WriteManager};
use crate::segment::SegmentIdentity;
use broker_core::cache::BrokerCacheManager;
use bytes::Bytes;
use common_base::tools::unique_id;
use common_config::broker::{default_broker_config, init_broker_conf_by_config};
use common_config::config::BrokerConfig;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::{EngineSegment, Replica};
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::test::test_rocksdb_instance;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;

#[allow(dead_code)]
pub fn test_build_segment() -> SegmentIdentity {
    SegmentIdentity {
        shard_name: unique_id(),
        segment: 10,
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
pub async fn test_init_segment() -> (
    SegmentIdentity,
    Arc<StorageCacheManager>,
    String,
    Arc<RocksDBEngine>,
) {
    test_init_conf();
    let rocksdb_engine_handler = test_rocksdb_instance();
    let segment_iden = test_build_segment();
    let fold = test_build_data_fold().first().unwrap().to_string();

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

    let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(BrokerCacheManager::new(
        BrokerConfig::default(),
    ))));

    create_local_segment(&cache_manager, &segment)
        .await
        .unwrap();

    cache_manager.set_segment_meta(EngineSegmentMetadata {
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment,
        ..Default::default()
    });

    (segment_iden, cache_manager, fold, rocksdb_engine_handler)
}

#[allow(dead_code)]
pub async fn test_base_write_data(
    len: u64,
) -> (
    SegmentIdentity,
    Arc<StorageCacheManager>,
    String,
    Arc<RocksDBEngine>,
) {
    let (segment_iden, cache_manager, fold, rocksdb_engine_handler) = test_init_segment().await;

    use crate::segment::offset::save_shard_offset;
    save_shard_offset(
        &rocksdb_engine_handler,
        &segment_iden.shard_name,
        segment_iden.segment,
        0,
    )
    .unwrap();

    let client_poll = Arc::new(ClientPool::new(100));

    let write_manager = WriteManager::new(
        rocksdb_engine_handler.clone(),
        cache_manager.clone(),
        client_poll.clone(),
        3,
    );

    let (stop_send, _) = broadcast::channel(2);
    write_manager.start(stop_send.clone());

    sleep(Duration::from_millis(100)).await;

    let mut data_list = Vec::new();
    for i in 0..len {
        data_list.push(WriteChannelDataRecord {
            pkid: i,
            header: None,
            key: Some(format!("key-{}", i)),
            tags: Some(vec![format!("tag-{}", i)]),
            value: Bytes::from(format!("data-{i}")),
        });
    }
    write_manager.write(&segment_iden, data_list).await.unwrap();

    stop_send.send(true).ok();
    sleep(Duration::from_millis(100)).await;

    (segment_iden, cache_manager, fold, rocksdb_engine_handler)
}
