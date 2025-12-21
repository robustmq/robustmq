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

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    storage::engine::StorageDriverEngineConfig, storage::memory::StorageDriverMemoryConfig,
    storage::minio::StorageDriverMinIoConfig, storage::mysql::StorageDriverMySQLConfig,
    storage::rocksdb::StorageDriverRocksDBConfig, storage::s3::StorageDriverS3Config,
};

pub mod engine;
pub mod memory;
pub mod minio;
pub mod mysql;
pub mod rocksdb;
pub mod s3;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct StorageAdapterConfig {
    pub storage_type: StorageAdapterType,
    pub engine_config: Option<StorageDriverEngineConfig>,
    pub memory_config: Option<StorageDriverMemoryConfig>,
    pub minio_config: Option<StorageDriverMinIoConfig>,
    pub mysql_config: Option<StorageDriverMySQLConfig>,
    pub rocksdb_config: Option<StorageDriverRocksDBConfig>,
    pub s3_config: Option<StorageDriverS3Config>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageAdapterType {
    // Engine,
    #[default]
    Memory,
    // Mysql,
    RocksDB,
    MinIO,
    S3,
}

impl FromStr for StorageAdapterType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // "engine" => Ok(StorageAdapterType::Engine),
            "memory" => Ok(StorageAdapterType::Memory),
            // "mysql" => Ok(StorageAdapterType::Mysql),
            "rocksdb" => Ok(StorageAdapterType::RocksDB), // "rocksdb" is an alias for backward compatibility
            "minio" => Ok(StorageAdapterType::MinIO),
            "s3" => Ok(StorageAdapterType::S3),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::storage::StorageAdapterType;

    #[test]
    fn storage_type_from_str() {
        // assert_eq!(
        //     StorageAdapterType::from_str("engine").unwrap(),
        //     StorageAdapterType::Engine
        // );
        assert_eq!(
            StorageAdapterType::from_str("memory").unwrap(),
            StorageAdapterType::Memory
        );
        // assert_eq!(
        //     StorageAdapterType::from_str("mysql").unwrap(),
        //     StorageAdapterType::Mysql
        // );
        assert_eq!(
            StorageAdapterType::from_str("rocksdb").unwrap(),
            StorageAdapterType::RocksDB
        );
        assert_eq!(
            StorageAdapterType::from_str("minio").unwrap(),
            StorageAdapterType::MinIO
        );
    }
}
