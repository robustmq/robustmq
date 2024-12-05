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

pub mod journal;
pub mod memory;
// pub mod mysql;
// pub mod rocksdb;
pub mod storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Journal,
    Memory,
    Mysql,
    Placement,
    RocksDB,
}

impl FromStr for StorageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "journal" => Ok(StorageType::Journal),
            "memory" => Ok(StorageType::Memory),
            "mysql" => Ok(StorageType::Mysql),
            "placement" => Ok(StorageType::Placement),
            "rocksdb" => Ok(StorageType::RocksDB),
            _ => Err(()),
        }
    }
}

// pub fn storage_is_journal(storage_type: &str) -> bool {

// }

// pub fn storage_is_memory(storage_type: &str) -> bool {
//     let st = format!("{:?}", StorageType::Memory).to_lowercase();
//     st == storage_type
// }

// pub fn storage_is_placement(storage_type: &str) -> bool {
//     let st = format!("{:?}", StorageType::Placement).to_lowercase();
//     st == storage_type
// }

// pub fn storage_is_mysql(storage_type: &str) -> bool {
//     let st = format!("{:?}", StorageType::Mysql).to_lowercase();
//     st == storage_type
// }

// pub fn storage_is_rocksdb(storage_type: &str) -> bool {
//     let st = format!("{:?}", StorageType::RocksDB).to_lowercase();
//     st == storage_type
// }

#[cfg(test)]
mod tests {
    // use crate::{storage_is_journal, storage_is_memory, storage_is_mysql, storage_is_rocksdb};

    // #[tokio::test]
    // async fn storage_type_test() {
    //     assert!(storage_is_journal("journal"));
    //     assert!(storage_is_memory("memory"));
    //     assert!(storage_is_mysql("mysql"));
    //     assert!(storage_is_rocksdb("rocksdb"));
    // }

    use std::str::FromStr;

    use crate::StorageType;

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
            StorageType::from_str("placement").unwrap(),
            StorageType::Placement
        );
        assert_eq!(
            StorageType::from_str("rocksdb").unwrap(),
            StorageType::RocksDB
        );
    }
}
