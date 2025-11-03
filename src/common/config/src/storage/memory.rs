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

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageDriverMemoryConfig {
    pub max_records_per_shard: usize,
    pub max_shard_size_limit: usize,
    pub initial_shard_capacity: usize,
    pub initial_group_capacity: usize,
    pub enable_tag_index: bool,
    pub enable_key_index: bool,
    pub enable_timestamp_index: bool,
}

impl Default for StorageDriverMemoryConfig {
    fn default() -> Self {
        Self {
            max_records_per_shard: 1000,
            max_shard_size_limit: 10_000_000,
            initial_shard_capacity: 16,
            initial_group_capacity: 8,
            enable_tag_index: true,
            enable_key_index: true,
            enable_timestamp_index: true,
        }
    }
}

impl StorageDriverMemoryConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_records_per_shard < 100 {
            return Err(format!(
                "max_records_per_shard ({}) must be at least 100",
                self.max_records_per_shard
            ));
        }

        if self.max_shard_size_limit < self.max_records_per_shard {
            return Err(format!(
                "max_shard_size_limit ({}) must be >= max_records_per_shard ({})",
                self.max_shard_size_limit, self.max_records_per_shard
            ));
        }

        if self.initial_shard_capacity == 0 {
            return Err("initial_shard_capacity cannot be 0".to_string());
        }

        if self.initial_shard_capacity > 1024 {
            return Err(format!(
                "initial_shard_capacity ({}) should not exceed 1024",
                self.initial_shard_capacity
            ));
        }

        if self.initial_group_capacity == 0 {
            return Err("initial_group_capacity cannot be 0".to_string());
        }

        Ok(())
    }

    pub fn estimate_memory_usage(&self, shard_count: usize, avg_record_size: usize) -> usize {
        let per_shard_memory = self.max_records_per_shard * avg_record_size;
        let total_data_memory = per_shard_memory * shard_count;
        let index_overhead = total_data_memory / 5;
        total_data_memory + index_overhead
    }

    pub fn dev_mode() -> Self {
        Self {
            max_records_per_shard: 1_000,
            max_shard_size_limit: 10_000,
            initial_shard_capacity: 4,
            initial_group_capacity: 2,
            enable_tag_index: true,
            enable_key_index: true,
            enable_timestamp_index: true,
        }
    }

    pub fn test_mode() -> Self {
        Self {
            max_records_per_shard: 10_000,
            max_shard_size_limit: 100_000,
            initial_shard_capacity: 16,
            initial_group_capacity: 8,
            enable_tag_index: true,
            enable_key_index: true,
            enable_timestamp_index: true,
        }
    }

    pub fn production_mode() -> Self {
        Self {
            max_records_per_shard: 100_000,
            max_shard_size_limit: 1_000_000,
            initial_shard_capacity: 64,
            initial_group_capacity: 32,
            enable_tag_index: true,
            enable_key_index: true,
            enable_timestamp_index: true,
        }
    }

    pub fn minimal_memory_mode() -> Self {
        Self {
            max_records_per_shard: 1_000,
            max_shard_size_limit: 5_000,
            initial_shard_capacity: 4,
            initial_group_capacity: 2,
            enable_tag_index: false,
            enable_key_index: false,
            enable_timestamp_index: false,
        }
    }
}
