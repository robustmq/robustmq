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
}

impl Default for StorageDriverMemoryConfig {
    fn default() -> Self {
        Self {
            max_records_per_shard: 1000,
            max_shard_size_limit: 10_000_000,
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
        }
    }

    pub fn test_mode() -> Self {
        Self {
            max_records_per_shard: 10_000,
            max_shard_size_limit: 100_000,
        }
    }

    pub fn production_mode() -> Self {
        Self {
            max_records_per_shard: 100_000,
            max_shard_size_limit: 1_000_000,
        }
    }

    pub fn minimal_memory_mode() -> Self {
        Self {
            max_records_per_shard: 1_000,
            max_shard_size_limit: 5_000,
        }
    }
}
