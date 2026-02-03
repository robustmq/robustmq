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

use crate::driver::StorageDriverManager;
use axum::async_trait;
use broker_core::cache::BrokerCacheManager;
use common_base::error::common::CommonError;
use common_base::uuid::unique_id;
use common_config::config::BrokerConfig;
use common_config::storage::memory::StorageDriverMemoryConfig;
use common_config::storage::StorageType;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::storage::adapter_offset::{
    AdapterConsumerGroupOffset, AdapterOffsetStrategy, AdapterShardInfo,
};
use metadata_struct::storage::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
use metadata_struct::storage::storage_record::StorageRecord;
use rocksdb_engine::test::test_rocksdb_instance;
use std::{collections::HashMap, sync::Arc};
use storage_engine::clients::manager::ClientConnectionManager;
use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
use storage_engine::commitlog::offset::CommitLogOffset;
use storage_engine::commitlog::rocksdb::engine::RocksDBStorageEngine;
use storage_engine::core::cache::StorageCacheManager;
use storage_engine::filesegment::write::WriteManager;
use storage_engine::group::OffsetManager;
use storage_engine::handler::adapter::{StorageEngineHandler, StorageEngineHandlerParams};

#[async_trait]
pub trait StorageAdapter {
    async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError>;

    async fn list_shard(&self, shard: Option<String>) -> Result<Vec<EngineShard>, CommonError>;

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError>;

    async fn write(
        &self,
        shard: &str,
        data: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, CommonError>;

    async fn batch_write(
        &self,
        shard: &str,
        data: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError>;

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError>;

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError>;

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<StorageRecord>, CommonError>;

    async fn delete_by_key(&self, shard: &str, key: &str) -> Result<(), CommonError>;

    async fn delete_by_offset(&self, shard: &str, offset: u64) -> Result<(), CommonError>;

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, CommonError>;

    async fn get_offset_by_group(
        &self,
        group_name: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError>;

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError>;

    async fn close(&self) -> Result<(), CommonError>;
}

pub async fn test_build_storage_driver_manager() -> Result<Arc<StorageDriverManager>, CommonError> {
    let rocksdb_engine_handler = test_rocksdb_instance();

    let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
    let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));

    let memory_storage_engine = Arc::new(MemoryStorageEngine::new(
        rocksdb_engine_handler.clone(),
        cache_manager.clone(),
        StorageDriverMemoryConfig::default(),
    ));

    let client_pool = Arc::new(ClientPool::new(8));
    let client_connection_manager =
        Arc::new(ClientConnectionManager::new(cache_manager.clone(), 5));
    let offset_manager = Arc::new(OffsetManager::new(
        client_pool.clone(),
        rocksdb_engine_handler.clone(),
        true,
    ));
    let write_manager = Arc::new(WriteManager::new(
        rocksdb_engine_handler.clone(),
        cache_manager.clone(),
        client_pool.clone(),
        4,
    ));

    let rocksdb_storage_engine = Arc::new(RocksDBStorageEngine::new(
        cache_manager.clone(),
        rocksdb_engine_handler.clone(),
    ));

    let params = StorageEngineHandlerParams {
        cache_manager: cache_manager.clone(),
        client_pool: client_pool.clone(),
        memory_storage_engine,
        rocksdb_engine_handler: rocksdb_engine_handler.clone(),
        client_connection_manager,
        offset_manager: offset_manager.clone(),
        write_manager,
        rocksdb_storage_engine,
    };

    let engine_adapter_handler = Arc::new(StorageEngineHandler::new(params));
    let driver = StorageDriverManager::new(offset_manager, engine_adapter_handler).await?;
    Ok(Arc::new(driver))
}

pub fn test_add_topic(storage_driver_manager: &Arc<StorageDriverManager>, topic_name: &str) {
    let topic = Topic::build_by_name(topic_name);
    storage_driver_manager
        .broker_cache
        .add_topic(topic_name, &topic);

    let shard_name = Topic::build_storage_name(&topic.topic_id, 0);

    storage_driver_manager
        .engine_storage_handler
        .cache_manager
        .set_shard(EngineShard {
            shard_uid: unique_id(),
            shard_name: shard_name.clone(),
            config: EngineShardConfig {
                storage_type: StorageType::EngineMemory,
                ..Default::default()
            },
            ..Default::default()
        });

    storage_driver_manager
        .engine_storage_handler
        .cache_manager
        .set_segment(&EngineSegment {
            shard_name: shard_name.clone(),
            segment_seq: 0,
            leader: 1,
            ..Default::default()
        });

    storage_driver_manager
        .engine_storage_handler
        .cache_manager
        .set_segment_meta(EngineSegmentMetadata {
            shard_name: shard_name.clone(),
            segment_seq: 0,
            ..Default::default()
        });
    let commit_offset = CommitLogOffset::new(
        storage_driver_manager
            .engine_storage_handler
            .cache_manager
            .clone(),
        storage_driver_manager
            .engine_storage_handler
            .rocksdb_engine_handler
            .clone(),
    );
    commit_offset.save_earliest_offset(&shard_name, 0).unwrap();
    commit_offset.save_latest_offset(&shard_name, 0).unwrap();
}
