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

use metadata_struct::journal::segment::JournalSegment;
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;

use crate::core::error::PlacementCenterError;
use crate::journal::cache::JournalCacheManager;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use crate::storage::journal::shard::ShardStorage;
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Clone)]
pub struct DataRouteJournal {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    engine_cache: Arc<JournalCacheManager>,
}

impl DataRouteJournal {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        engine_cache: Arc<JournalCacheManager>,
    ) -> Self {
        DataRouteJournal {
            rocksdb_engine_handler,
            engine_cache,
        }
    }

    pub async fn set_shard(&self, value: Vec<u8>) -> Result<Vec<u8>, PlacementCenterError> {
        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());

        let shard_info = serde_json::from_slice::<JournalShard>(&value)?;
        shard_storage.save(&shard_info)?;

        self.engine_cache.set_shard(&shard_info);

        Ok(value)
    }

    pub async fn delete_shard(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let shard_info = serde_json::from_slice::<JournalShard>(&value)?;

        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        shard_storage.delete(
            &shard_info.cluster_name,
            &shard_info.namespace,
            &shard_info.shard_name,
        )?;

        self.engine_cache.remove_shard(
            &shard_info.cluster_name,
            &shard_info.namespace,
            &shard_info.shard_name,
        );

        Ok(())
    }

    pub async fn set_segment(&self, value: Vec<u8>) -> Result<Vec<u8>, PlacementCenterError> {
        let segment = serde_json::from_slice::<JournalSegment>(&value)?;

        let storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(segment.clone())?;

        self.engine_cache.set_segment(&segment);

        Ok(value)
    }

    pub async fn delete_segment(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let segment = serde_json::from_slice::<JournalSegment>(&value)?;

        let storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
            segment.segment_seq,
        )?;

        self.engine_cache.remove_segment(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
            segment.segment_seq,
        );
        Ok(())
    }

    pub async fn set_segment_meta(&self, value: Vec<u8>) -> Result<Vec<u8>, PlacementCenterError> {
        let meta = serde_json::from_slice::<JournalSegmentMetadata>(&value)?;

        let storage = SegmentMetadataStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(meta.clone())?;

        self.engine_cache.set_segment_meta(&meta);

        Ok(value)
    }

    pub async fn delete_segment_meta(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let meta = serde_json::from_slice::<JournalSegmentMetadata>(&value)?;

        let storage = SegmentMetadataStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(
            &meta.cluster_name,
            &meta.namespace,
            &meta.shard_name,
            meta.segment_seq,
        )?;

        self.engine_cache.remove_segment_meta(
            &meta.cluster_name,
            &meta.namespace,
            &meta.shard_name,
            meta.segment_seq,
        );
        Ok(())
    }
}
