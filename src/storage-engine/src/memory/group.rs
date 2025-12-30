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
    core::error::StorageEngineError,
    memory::engine::{MemoryStorageEngine, MemoryStorageType},
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
            MemoryStorageType::Standalone => {
                let group_map = self
                    .group_data
                    .entry(group_name.to_string())
                    .or_insert(DashMap::with_capacity(8));

                Ok(group_map
                    .iter()
                    .map(|entry| AdapterConsumerGroupOffset {
                        group: group_name.to_string(),
                        shard_name: entry.key().clone(),
                        offset: *entry.value(),
                        ..Default::default()
                    })
                    .collect())
            }
            MemoryStorageType::EngineStorage => {
                if let Some(group_map) = self.group_data.get(group_name) {
                    Ok(group_map
                        .iter()
                        .map(|entry| AdapterConsumerGroupOffset {
                            group: group_name.to_string(),
                            shard_name: entry.key().clone(),
                            offset: *entry.value(),
                            ..Default::default()
                        })
                        .collect())
                } else {
                    let data = self.offset_manager.get_offset(group_name).await?;
                    let group_info = DashMap::with_capacity(8);
                    for raw in data.clone() {
                        group_info.insert(raw.shard_name, raw.offset);
                    }
                    self.group_data.insert(group_name.to_string(), group_info);
                    Ok(data)
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
        let group_map = self
            .group_data
            .entry(group_name.to_string())
            .or_insert_with(|| DashMap::with_capacity(offset.len()));

        for (shard_name, offset_val) in offset.iter() {
            group_map.insert(shard_name.clone(), *offset_val);
        }

        if self.engine_type == MemoryStorageType::EngineStorage {
            self.offset_manager
                .commit_offset(group_name, offset)
                .await?;
        }

        Ok(())
    }
}
