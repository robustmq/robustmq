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
pub struct StorageDriverRocksDBConfig {
    pub data_path: String,
    pub max_open_files: i32,
    pub block_cache_size: usize,
    pub write_buffer_size: usize,
    pub max_write_buffer_number: i32,
}

impl Default for StorageDriverRocksDBConfig {
    fn default() -> Self {
        Self {
            data_path: "./robustmq_data/rocksdb".to_string(),
            max_open_files: 1000,
            block_cache_size: 512 * 1024 * 1024,
            write_buffer_size: 128 * 1024 * 1024,
            max_write_buffer_number: 4,
        }
    }
}

impl StorageDriverRocksDBConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.data_path.is_empty() {
            return Err("data_path cannot be empty".to_string());
        }

        if self.max_open_files < -1 {
            return Err(format!(
                "max_open_files ({}) must be >= -1 (-1 means unlimited)",
                self.max_open_files
            ));
        }

        if self.block_cache_size < 8 * 1024 * 1024 {
            return Err(format!(
                "block_cache_size ({}) must be at least 8MB",
                self.block_cache_size
            ));
        }

        if self.write_buffer_size < 8 * 1024 * 1024 {
            return Err(format!(
                "write_buffer_size ({}) must be at least 8MB",
                self.write_buffer_size
            ));
        }

        if self.max_write_buffer_number < 2 {
            return Err(format!(
                "max_write_buffer_number ({}) must be at least 2",
                self.max_write_buffer_number
            ));
        }

        Ok(())
    }
}
