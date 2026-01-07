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
    pub storage_type: StorageType,
    pub engine_config: Option<StorageDriverEngineConfig>,
    pub memory_config: Option<StorageDriverMemoryConfig>,
    pub minio_config: Option<StorageDriverMinIoConfig>,
    pub mysql_config: Option<StorageDriverMySQLConfig>,
    pub rocksdb_config: Option<StorageDriverRocksDBConfig>,
    pub s3_config: Option<StorageDriverS3Config>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageType {
    #[default]
    EngineMemory,
    EngineSegment,
    EngineRocksDB,
    Mysql,
    MinIO,
    S3,
}

impl FromStr for StorageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EngineSegment" => Ok(StorageType::EngineSegment),
            "EngineMemory" => Ok(StorageType::EngineMemory),
            "EngineRocksDB" => Ok(StorageType::EngineRocksDB),
            "MinIO" => Ok(StorageType::MinIO),
            "S3" => Ok(StorageType::S3),
            "Mysql" => Ok(StorageType::Mysql),
            _ => Err(()),
        }
    }
}
