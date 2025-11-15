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

use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use crate::storage::journal::shard::ShardStorage;
use bytes::Bytes;
use metadata_struct::journal::segment::JournalSegment;
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct DataRouteJournal {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<CacheManager>,
}

impl DataRouteJournal {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        DataRouteJournal {
            rocksdb_engine_handler,
            cache_manager,
        }
    }

    pub async fn set_shard(&self, value: Bytes) -> Result<Bytes, MetaServiceError> {
        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());

        let shard_info = JournalShard::decode(&value)?;
        shard_storage.save(&shard_info)?;

        self.cache_manager.set_shard(&shard_info);

        Ok(value)
    }

    pub async fn delete_shard(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let shard_info = JournalShard::decode(&value)?;

        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        shard_storage.delete(
            &shard_info.cluster_name,
            &shard_info.namespace,
            &shard_info.shard_name,
        )?;

        self.cache_manager.remove_shard(
            &shard_info.cluster_name,
            &shard_info.namespace,
            &shard_info.shard_name,
        );

        Ok(())
    }

    pub async fn set_segment(&self, value: Bytes) -> Result<Bytes, MetaServiceError> {
        let segment = JournalSegment::decode(&value)?;

        let storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(segment.clone())?;

        self.cache_manager.set_segment(&segment);

        Ok(value)
    }

    pub async fn delete_segment(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let segment = JournalSegment::decode(&value)?;

        let storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
            segment.segment_seq,
        )?;

        self.cache_manager.remove_segment(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
            segment.segment_seq,
        );
        Ok(())
    }

    pub async fn set_segment_meta(&self, value: Bytes) -> Result<Bytes, MetaServiceError> {
        let meta = JournalSegmentMetadata::decode(&value)?;

        let storage = SegmentMetadataStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(meta.clone())?;

        self.cache_manager.set_segment_meta(&meta);

        Ok(value)
    }

    pub async fn delete_segment_meta(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let meta = JournalSegmentMetadata::decode(&value)?;

        let storage = SegmentMetadataStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(
            &meta.cluster_name,
            &meta.namespace,
            &meta.shard_name,
            meta.segment_seq,
        )?;

        self.cache_manager.remove_segment_meta(
            &meta.cluster_name,
            &meta.namespace,
            &meta.shard_name,
            meta.segment_seq,
        );
        Ok(())
    }
}
