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

use crate::commitlog::memory::engine::{MemoryShardData, MemoryStorageEngine};
use common_base::tools::now_second;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use std::collections::HashMap;
use std::sync::Arc;

impl MemoryStorageEngine {
    pub fn batch_save_index(shard: &Arc<MemoryShardData>, entries: &[(u64, &AdapterWriteRecord)]) {
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
}
