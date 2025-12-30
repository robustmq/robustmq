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

use crate::{core::error::StorageEngineError, memory::engine::MemoryStorageEngine};
use dashmap::DashMap;
use metadata_struct::storage::adapter_offset::{AdapterConsumerGroupOffset, AdapterOffsetStrategy};
use std::collections::HashMap;

impl MemoryStorageEngine {
    pub async fn get_offset_by_group(
        &self,
        group_name: &str,
        _strategy: AdapterOffsetStrategy,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, StorageEngineError> {
        let Some(group_map) = self.group_data.get(group_name) else {
            return Ok(Vec::new());
        };

        let offsets = group_map
            .iter()
            .map(|entry| AdapterConsumerGroupOffset {
                group: group_name.to_string(),
                shard_name: entry.key().clone(),
                offset: *entry.value(),
                ..Default::default()
            })
            .collect();

        Ok(offsets)
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), StorageEngineError> {
        if offset.is_empty() {
            return Ok(());
        }

        let group_map = self
            .group_data
            .entry(group_name.to_string())
            .or_insert_with(|| DashMap::with_capacity(offset.len()));

        for (shard_name, offset_val) in offset.iter() {
            group_map.insert(shard_name.clone(), *offset_val);
        }

        Ok(())
    }
}
