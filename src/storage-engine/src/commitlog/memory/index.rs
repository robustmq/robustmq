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

use crate::commitlog::memory::engine::{MemoryStorageEngine, ShardState};
use common_base::tools::now_second;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use std::collections::HashMap;
use std::sync::Arc;

impl MemoryStorageEngine {
    /// Write indexes for a single message. Prefer [`batch_save_index`] in the
    /// write hot path where multiple messages are indexed at once.
    pub fn save_index(&self, shard_name: &str, offset: u64, msg: &AdapterWriteRecord) {
        let shard = self.get_or_create_shard(shard_name);
        Self::index_one(&shard, offset, msg);
    }

    /// Write indexes for a batch of `(offset, msg)` pairs into an already-
    /// resolved `ShardState`. Avoids repeated shard lookups and merges tag
    /// accumulations before touching `tag_index`, reducing DashMap contention
    /// when the same tag appears more than once in the batch.
    pub fn batch_save_index(
        shard: &Arc<ShardState>,
        entries: &[(u64, &AdapterWriteRecord)],
    ) {
        // Collect key → offset (last writer wins, consistent with single-write behaviour).
        // Collect tag → Vec<offset> for bulk extend.
        let mut tag_batch: HashMap<&str, Vec<u64>> = HashMap::new();

        let now = now_second();

        for &(offset, msg) in entries {
            if let Some(key) = &msg.key {
                shard.key_index.insert(key.clone(), offset);
            }

            if let Some(tags) = &msg.tags {
                for tag in tags.iter() {
                    tag_batch.entry(tag.as_str()).or_default().push(offset);
                }
            }

            if now > 0 && offset.is_multiple_of(5000) {
                shard.timestamp_index.entry(now).or_insert(offset);
            }
        }

        for (tag, offsets) in tag_batch {
            shard
                .tag_index
                .entry(tag.to_owned())
                .or_default()
                .extend(offsets);
        }
    }

    pub fn remove_indexes(&self, shard_key: &str) {
        self.shards.remove(shard_key);
    }

    fn index_one(shard: &Arc<ShardState>, offset: u64, msg: &AdapterWriteRecord) {
        if let Some(key) = &msg.key {
            shard.key_index.insert(key.clone(), offset);
        }

        if let Some(tags) = &msg.tags {
            for tag in tags.iter() {
                shard.tag_index.entry(tag.clone()).or_default().push(offset);
            }
        }

        let now = now_second();
        if now > 0 && offset.is_multiple_of(5000) {
            shard.timestamp_index.entry(now).or_insert(offset);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_build_memory_engine;
    use common_base::uuid::unique_id;

    #[tokio::test]
    async fn test_index_operations() {
        let engine = test_build_memory_engine();
        let shard_name = unique_id();
        let msg = AdapterWriteRecord {
            key: Some("test_key".to_string()),
            tags: Some(vec!["test_tag".to_string()]),
            ..Default::default()
        };
        engine.save_index(&shard_name, 100, &msg);
        let shard = engine.shards.get(&shard_name).unwrap();
        assert!(shard.key_index.contains_key("test_key"));
        assert!(shard.tag_index.contains_key("test_tag"));
        drop(shard);
        engine.remove_indexes(&shard_name);
        assert!(!engine.shards.contains_key(&shard_name));
    }
}
