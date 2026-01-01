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
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;

impl CacheManager {
    pub fn set_shard(&self, shard: EngineShard) {
        self.shard_list.insert(shard.shard_name.clone(), shard);
    }

    pub fn remove_shard(&self, shard_name: &str) {
        self.shard_list.remove(shard_name);
        self.segment_list.remove(shard_name);
        self.segment_meta_list.remove(shard_name);
        self.wait_delete_shard_list.remove(shard_name);
    }

    pub fn get_segment(&self, shard_name: &str, segment_seq: u32) -> Option<EngineSegment> {
        self.segment_list
            .get(shard_name)?
            .get(&segment_seq)
            .map(|entry| entry.value().clone())
    }

    pub fn set_segment(&self, segment: EngineSegment) {
        let list = self
            .segment_list
            .entry(segment.shard_name.clone())
            .or_insert(DashMap::with_capacity(8));
        list.insert(segment.segment_seq, segment);
    }

    pub fn remove_segment(&self, shard_name: &str, segment_seq: u32) {
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            segment_list.remove(&segment_seq);
        }
    }

    pub fn get_segment_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegment> {
        self.segment_list
            .get(shard_name)
            .map(|segment_list| {
                segment_list
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_segment_meta(
        &self,
        shard_name: &str,
        segment_seq: u32,
    ) -> Option<EngineSegmentMetadata> {
        self.segment_meta_list
            .get(shard_name)?
            .get(&segment_seq)
            .map(|entry| entry.value().clone())
    }

    pub fn set_segment_meta(&self, meta: EngineSegmentMetadata) {
        let list_data = self
            .segment_meta_list
            .entry(meta.shard_name.clone())
            .or_insert(DashMap::with_capacity(8));
        list_data.insert(meta.segment_seq, meta);
    }

    pub fn remove_segment_meta(&self, shard_name: &str, segment_seq: u32) {
        if let Some(list) = self.segment_meta_list.get(shard_name) {
            list.remove(&segment_seq);
        }
    }

    pub fn get_segment_meta_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegmentMetadata> {
        self.segment_meta_list
            .get(shard_name)
            .map(|segment_list| {
                segment_list
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn add_wait_delete_shard(&self, shard: &str) {
        self.wait_delete_shard_list
            .insert(shard.to_string(), now_second());
    }

    pub fn remove_wait_delete_shard(&self, shard: &str) {
        self.wait_delete_shard_list.remove(shard);
    }

    pub fn add_wait_delete_segment(&self, segment: &EngineSegment) {
        self.wait_delete_segment_list
            .insert(segment.name(), segment.clone());
    }

    pub fn remove_wait_delete_segment(&self, segment: &EngineSegment) {
        self.wait_delete_segment_list.remove(&segment.name());
    }

    pub fn get_wait_delete_shard_list(&self) -> Vec<String> {
        self.wait_delete_shard_list
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn get_wait_delete_segment_list(&self) -> Vec<EngineSegment> {
        self.wait_delete_segment_list
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cache::CacheManager;
    use common_base::tools::now_second;
    use metadata_struct::storage::segment::SegmentStatus;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn test_cache_manager() -> CacheManager {
        CacheManager::new(test_rocksdb_instance())
    }

    fn test_shard() -> EngineShard {
        EngineShard {
            shard_uid: "test-uid".to_string(),
            shard_name: "test-shard".to_string(),
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            status: metadata_struct::storage::shard::EngineShardStatus::Run,
            config: metadata_struct::storage::shard::EngineShardConfig::default(),
            replica_num: 3,
            engine_type: metadata_struct::storage::shard::EngineStorageType::Segment,
            create_time: now_second(),
        }
    }

    fn test_segment(shard_name: &str, segment_seq: u32) -> EngineSegment {
        EngineSegment {
            shard_name: shard_name.to_string(),
            segment_seq,
            leader_epoch: 0,
            status: SegmentStatus::Write,
            leader: 1,
            replicas: Vec::new(),
            isr: Vec::new(),
        }
    }

    fn test_segment_meta(shard_name: &str, segment_seq: u32) -> EngineSegmentMetadata {
        EngineSegmentMetadata {
            shard_name: shard_name.to_string(),
            segment_seq,
            start_offset: 0,
            end_offset: 100,
            start_timestamp: now_second() as i64,
            end_timestamp: now_second() as i64,
        }
    }

    #[test]
    fn shard_operations_test() {
        let cache = test_cache_manager();
        let shard = test_shard();

        cache.set_shard(shard.clone());
        assert!(cache.shard_list.contains_key(&shard.shard_name));

        cache.remove_shard(&shard.shard_name);
        assert!(!cache.shard_list.contains_key(&shard.shard_name));
    }

    #[test]
    fn segment_operations_test() {
        let cache = test_cache_manager();
        let segment1 = test_segment("shard1", 0);
        let segment2 = test_segment("shard1", 1);

        cache.set_segment(segment1.clone());
        cache.set_segment(segment2.clone());

        let result = cache.get_segment("shard1", 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().segment_seq, 0);

        let list = cache.get_segment_list_by_shard("shard1");
        assert_eq!(list.len(), 2);

        cache.remove_segment("shard1", 0);
        let result = cache.get_segment("shard1", 0);
        assert!(result.is_none());

        let list = cache.get_segment_list_by_shard("shard1");
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn segment_meta_operations_test() {
        let cache = test_cache_manager();
        let meta1 = test_segment_meta("shard1", 0);
        let meta2 = test_segment_meta("shard1", 1);

        cache.set_segment_meta(meta1.clone());
        cache.set_segment_meta(meta2.clone());

        let result = cache.get_segment_meta("shard1", 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().segment_seq, 0);

        let list = cache.get_segment_meta_list_by_shard("shard1");
        assert_eq!(list.len(), 2);

        cache.remove_segment_meta("shard1", 0);
        let result = cache.get_segment_meta("shard1", 0);
        assert!(result.is_none());

        let list = cache.get_segment_meta_list_by_shard("shard1");
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn wait_delete_shard_list_test() {
        let cache = test_cache_manager();

        cache.add_wait_delete_shard("shard1");
        cache.add_wait_delete_shard("shard2");

        let list = cache.get_wait_delete_shard_list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"shard1".to_string()));

        cache.remove_wait_delete_shard("shard1");
        let list = cache.get_wait_delete_shard_list();
        assert_eq!(list.len(), 1);
        assert!(!list.contains(&"shard1".to_string()));
    }

    #[test]
    fn wait_delete_segment_list_test() {
        let cache = test_cache_manager();
        let segment1 = test_segment("shard1", 0);
        let segment2 = test_segment("shard1", 1);

        cache.add_wait_delete_segment(&segment1);
        cache.add_wait_delete_segment(&segment2);

        let list = cache.get_wait_delete_segment_list();
        assert_eq!(list.len(), 2);

        cache.remove_wait_delete_segment(&segment1);
        let list = cache.get_wait_delete_segment_list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].segment_seq, 1);
    }
}
