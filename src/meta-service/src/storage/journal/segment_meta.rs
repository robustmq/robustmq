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

use crate::storage::engine_meta::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_meta,
};
use common_base::error::common::CommonError;
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use rocksdb_engine::RocksDBEngine;

use crate::storage::keys::{
    key_all_segment_metadata, key_segment_metadata, key_segment_metadata_cluster_prefix,
    key_segment_metadata_namespace_prefix, key_segment_metadata_shard_prefix,
};

pub struct SegmentMetadataStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SegmentMetadataStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SegmentMetadataStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, segment: JournalSegmentMetadata) -> Result<(), CommonError> {
        let shard_key = key_segment_metadata(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
            segment.segment_seq,
        );
        engine_save_by_meta(self.rocksdb_engine_handler.clone(), shard_key, segment)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<Option<JournalSegmentMetadata>, CommonError> {
        let shard_key: String =
            key_segment_metadata(cluster_name, namespace, shard_name, segment_seq);

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)? {
            return Ok(Some(serde_json::from_str::<JournalSegmentMetadata>(
                &data.data,
            )?));
        }

        Ok(None)
    }

    pub fn all_segment(&self) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_all_segment_metadata();
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_cluster(
        &self,
        cluster_name: &str,
    ) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_segment_metadata_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_namespace(
        &self,
        cluster_name: &str,
        namespace: &str,
    ) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_segment_metadata_namespace_prefix(cluster_name, namespace);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_shard(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_segment_metadata_shard_prefix(cluster_name, namespace, shard_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<(), CommonError> {
        let shard_key = key_segment_metadata(cluster_name, namespace, shard_name, segment_seq);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
    use rocksdb_engine::RocksDBEngine;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn create_test_instance() -> SegmentMetadataStorage {
        // Create a temporary directory for the database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let rocksdb_engine = RocksDBEngine::new(db_path, 0, vec!["meta".to_string()]);
        let rocksdb_engine_handler = Arc::new(rocksdb_engine);
        SegmentMetadataStorage::new(rocksdb_engine_handler)
    }

    fn create_test_segment(seq: u32) -> JournalSegmentMetadata {
        JournalSegmentMetadata {
            cluster_name: "test-cluster".to_string(),
            namespace: "test-namespace".to_string(),
            shard_name: "test-shard".to_string(),
            segment_seq: seq,
            start_offset: seq as i64 * 100,
            end_offset: seq as i64 * 100 + 99,
            start_timestamp: seq as i64 * 1000,
            end_timestamp: seq as i64 * 1000 + 999,
        }
    }

    #[test]
    fn test_save_and_get() {
        let storage = create_test_instance();
        let segment = create_test_segment(1);

        // Test save functionality
        storage
            .save(segment.clone())
            .expect("Failed to save segment metadata");

        // Test get functionality
        let retrieved = storage
            .get(
                &segment.cluster_name,
                &segment.namespace,
                &segment.shard_name,
                segment.segment_seq,
            )
            .expect("Failed to get segment metadata")
            .expect("Segment metadata does not exist");

        // Verify all fields
        assert_eq!(retrieved.cluster_name, segment.cluster_name);
        assert_eq!(retrieved.namespace, segment.namespace);
        assert_eq!(retrieved.shard_name, segment.shard_name);
        assert_eq!(retrieved.segment_seq, segment.segment_seq);
        assert_eq!(retrieved.start_offset, segment.start_offset);
        assert_eq!(retrieved.end_offset, segment.end_offset);
        assert_eq!(retrieved.start_timestamp, segment.start_timestamp);
        assert_eq!(retrieved.end_timestamp, segment.end_timestamp);
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = create_test_instance();

        // Test getting non-existent metadata
        let result = storage
            .get("nonexistent", "nonexistent", "nonexistent", 998)
            .expect("Failed to perform get operation");

        assert!(result.is_none(), "Expected None but got Some");
    }

    #[test]
    fn test_list_operations() {
        let storage = create_test_instance();

        // Create multiple test segments
        let segments = vec![
            create_test_segment(1),
            create_test_segment(2),
            create_test_segment(3),
        ];

        // Save all segments
        for segment in &segments {
            storage
                .save(segment.clone())
                .expect("Failed to save segment metadata");
        }

        // Test all_segment
        let all_segments = storage.all_segment().expect("Failed to get all segments");
        assert_eq!(all_segments.len(), segments.len());

        // Test list_by_cluster
        let cluster_segments = storage
            .list_by_cluster(&segments[0].cluster_name)
            .expect("Failed to list segments by cluster");
        assert_eq!(cluster_segments.len(), segments.len());

        // Test list_by_namespace
        let namespace_segments = storage
            .list_by_namespace(&segments[0].cluster_name, &segments[0].namespace)
            .expect("Failed to list segments by namespace");
        assert_eq!(namespace_segments.len(), segments.len());

        // Test list_by_shard
        let shard_segments = storage
            .list_by_shard(
                &segments[0].cluster_name,
                &segments[0].namespace,
                &segments[0].shard_name,
            )
            .expect("Failed to list segments by shard");
        assert_eq!(shard_segments.len(), segments.len());
    }

    #[test]
    fn test_delete() {
        let storage = create_test_instance();
        let segment = create_test_segment(1);

        // Save segment
        storage
            .save(segment.clone())
            .expect("Failed to save segment metadata");

        // Verify segment is saved
        assert!(storage
            .get(
                &segment.cluster_name,
                &segment.namespace,
                &segment.shard_name,
                segment.segment_seq,
            )
            .expect("Failed to get segment metadata")
            .is_some());

        // Delete segment
        storage
            .delete(
                &segment.cluster_name,
                &segment.namespace,
                &segment.shard_name,
                segment.segment_seq,
            )
            .expect("Failed to delete segment metadata");

        // Verify segment is deleted
        assert!(storage
            .get(
                &segment.cluster_name,
                &segment.namespace,
                &segment.shard_name,
                segment.segment_seq,
            )
            .expect("Failed to get segment metadata")
            .is_none());
    }
}
