use super::cache::StorageCacheManager;
use crate::segment::index::build::try_trigger_build_index;
use crate::segment::manager::{create_local_segment, SegmentFileManager};
use crate::segment::offset::save_shard_offset;
use crate::segment::write::{WriteChannelDataRecord, WriteManager};
use crate::segment::SegmentIdentity;
use broker_core::cache::BrokerCacheManager;
use bytes::Bytes;
use common_base::tools::unique_id;
use common_config::broker::{default_broker_config, init_broker_conf_by_config};
use common_config::config::BrokerConfig;
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
        shard_name: "s1".to_string(),
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
    Arc<SegmentFileManager>,
    String,
    Arc<RocksDBEngine>,
) {
    test_init_conf();
    let rocksdb_engine_handler = test_rocksdb_instance();
    let segment_iden = test_build_segment();
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

    let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(
        BrokerCacheManager::new(BrokerConfig::default()),
    )));

    create_local_segment(&cache_manager, &segment_file_manager, &segment)
        .await
        .unwrap();

    cache_manager.set_segment_meta(EngineSegmentMetadata {
        shard_name: segment_iden.shard_name.clone(),
        segment_seq: segment_iden.segment,
        ..Default::default()
    });

    (
        segment_iden,
        cache_manager,
        segment_file_manager,
        fold,
        rocksdb_engine_handler,
    )
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

    let write_manager = WriteManager::new(
        rocksdb_engine_handler.clone(),
        segment_file_manager.clone(),
        cache_manager.clone(),
        3,
    );

    let (stop_send, _) = broadcast::channel(2);
    write_manager.start(stop_send);

    let mut data_list = Vec::new();
    for i in 0..len {
        data_list.push(WriteChannelDataRecord {
            pkid: i,
            key: None,
            tags: None,
            value: Bytes::from(format!("data-{i}")),
        });
    }

    write_manager
        .write(&segment_iden, data_list)
        .await
        .unwrap();

    (
        segment_iden,
        cache_manager,
        segment_file_manager,
        fold,
        rocksdb_engine_handler,
    )
}

#[allow(dead_code)]
pub async fn test_write_and_build_index(
    record_count: u64,
    wait_secs: u64,
) -> (
    SegmentIdentity,
    Arc<StorageCacheManager>,
    Arc<SegmentFileManager>,
    String,
    Arc<RocksDBEngine>,
) {
    let (segment_iden, cache_manager, segment_file_manager, fold, rocksdb_engine_handler) =
        test_init_segment().await;

    save_shard_offset(&rocksdb_engine_handler, &segment_iden.shard_name, 0).unwrap();

    let write_manager = WriteManager::new(
        rocksdb_engine_handler.clone(),
        segment_file_manager.clone(),
        cache_manager.clone(),
        3,
    );

    let (stop_send, _) = broadcast::channel(2);
    write_manager.start(stop_send);

    sleep(Duration::from_millis(100)).await;

    let mut data_list = Vec::new();
    for i in 0..record_count {
        data_list.push(WriteChannelDataRecord {
            pkid: i,
            key: Some(format!("key-{}", i)),
            tags: Some(vec![format!("tag-{}", i)]),
            value: Bytes::from(format!("data-{}", i)),
        });
    }

    write_manager
        .write(&segment_iden, data_list)
        .await
        .unwrap();

    try_trigger_build_index(
        &cache_manager,
        &segment_file_manager,
        &rocksdb_engine_handler,
        &segment_iden,
    )
    .await
    .unwrap();

    sleep(Duration::from_secs(wait_secs)).await;

    (
        segment_iden,
        cache_manager,
        segment_file_manager,
        fold,
        rocksdb_engine_handler,
    )
}
