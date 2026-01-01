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

use crate::{
    core::{error::StorageEngineError, shard::StorageEngineRunType},
    memory::engine::MemoryStorageEngine,
};
use dashmap::DashMap;
use metadata_struct::storage::adapter_offset::AdapterConsumerGroupOffset;
use std::collections::HashMap;

impl MemoryStorageEngine {
    pub async fn get_offset_by_group(
        &self,
        group_name: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, StorageEngineError> {
        match self.engine_type {
            StorageEngineRunType::Standalone => {
                if let Some(group_map) = self.group_data.get(group_name) {
                    Ok(Self::build_offset_list(group_name, &group_map))
                } else {
                    Ok(Vec::new())
                }
            }
            StorageEngineRunType::EngineStorage => {
                match self.group_data.entry(group_name.to_string()) {
                    dashmap::mapref::entry::Entry::Occupied(entry) => {
                        Ok(Self::build_offset_list(group_name, entry.get()))
                    }
                    dashmap::mapref::entry::Entry::Vacant(entry) => {
                        let data = self.offset_manager.get_offset(group_name).await?;
                        let capacity = data.len().max(8);
                        let group_info = DashMap::with_capacity(capacity);
                        for item in &data {
                            group_info.insert(item.shard_name.clone(), item.offset);
                        }
                        entry.insert(group_info);
                        Ok(data)
                    }
                }
            }
        }
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), StorageEngineError> {
        if offset.is_empty() {
            return Ok(());
        }

        if self.engine_type == StorageEngineRunType::EngineStorage {
            self.offset_manager
                .commit_offset(group_name, offset)
                .await?;
        }

        let capacity = offset.len().max(8);
        let group_map = self
            .group_data
            .entry(group_name.to_string())
            .or_insert_with(|| DashMap::with_capacity(capacity));

        for (shard_name, &offset_val) in offset.iter() {
            group_map.insert(shard_name.clone(), offset_val);
        }

        Ok(())
    }

    fn build_offset_list(
        group_name: &str,
        group_map: &DashMap<String, u64>,
    ) -> Vec<AdapterConsumerGroupOffset> {
        group_map
            .iter()
            .map(|entry| AdapterConsumerGroupOffset {
                group: group_name.to_string(),
                shard_name: entry.key().clone(),
                offset: *entry.value(),
                ..Default::default()
            })
            .collect()
    }
}
