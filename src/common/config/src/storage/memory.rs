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
    pub max_shard_size_limit: usize,
    /// Fraction of records to evict when a shard exceeds its capacity limit.
    /// Set to 0.2 (20%) so each eviction reclaims enough headroom that the shard
    /// can accept a burst of new writes before the next eviction is needed.
    /// A smaller ratio would mean evicting just a few records at a time, causing
    /// continuous eviction loops under sustained write load.
    pub evict_ratio: f64,
}

impl Default for StorageDriverMemoryConfig {
    fn default() -> Self {
        Self {
            max_shard_size_limit: 10_000_000,
            evict_ratio: 0.2,
        }
    }
}

impl StorageDriverMemoryConfig {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn dev_mode() -> Self {
        Self {
            max_shard_size_limit: 10_000,
            evict_ratio: 0.2,
        }
    }

    pub fn test_mode() -> Self {
        Self {
            max_shard_size_limit: 100_000,
            evict_ratio: 0.2,
        }
    }

    pub fn production_mode() -> Self {
        Self {
            max_shard_size_limit: 1_000_000,
            evict_ratio: 0.2,
        }
    }

    pub fn minimal_memory_mode() -> Self {
        Self {
            max_shard_size_limit: 5_000,
            evict_ratio: 0.2,
        }
    }
}
