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

#![allow(clippy::result_large_err)]

use std::str::FromStr;

pub mod driver;
pub mod journal;
pub mod memory;
pub mod message_expire;
pub mod minio;
pub mod mysql;
pub mod rocksdb;
pub mod s3;
pub mod storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Journal,
    Memory,
    Mysql,
    RocksDB,
    MinIO,
}

impl FromStr for StorageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "journal" => Ok(StorageType::Journal),
            "memory" => Ok(StorageType::Memory),
            "mysql" => Ok(StorageType::Mysql),
            "rocksdb" => Ok(StorageType::RocksDB),
            "minio" => Ok(StorageType::MinIO),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::StorageType;
    use std::str::FromStr;

    #[test]
    fn storage_type_from_str() {
        assert_eq!(
            StorageType::from_str("journal").unwrap(),
            StorageType::Journal
        );
        assert_eq!(
            StorageType::from_str("memory").unwrap(),
            StorageType::Memory
        );
        assert_eq!(StorageType::from_str("mysql").unwrap(), StorageType::Mysql);
        assert_eq!(
            StorageType::from_str("rocksdb").unwrap(),
            StorageType::RocksDB
        );
        assert_eq!(StorageType::from_str("minio").unwrap(), StorageType::MinIO);
    }
}
