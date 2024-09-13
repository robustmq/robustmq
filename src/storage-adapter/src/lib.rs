// Copyright 2023 RobustMQ Team
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


pub mod journal;
pub mod memory;
pub mod mysql;
pub mod placement;
pub mod storage;
pub mod local_rocksdb;

#[derive(Debug)]
pub enum StorageType {
    Journal,
    Memory,
    Mysql,
    Placement,
}

pub fn storage_is_journal(storage_type: &String) -> bool {
    let st = format!("{:?}", StorageType::Journal).to_lowercase();
    return st == storage_type.clone();
}

pub fn storage_is_memory(storage_type: &String) -> bool {
    let st = format!("{:?}", StorageType::Memory).to_lowercase();
    return st == storage_type.clone();
}

pub fn storage_is_placement(storage_type: &String) -> bool {
    let st = format!("{:?}", StorageType::Placement).to_lowercase();
    return st == storage_type.clone();
}

pub fn storage_is_mysql(storage_type: &String) -> bool {
    let st = format!("{:?}", StorageType::Mysql).to_lowercase();
    return st == storage_type.clone();
}

#[cfg(test)]
mod tests {
    use crate::{storage_is_journal, storage_is_memory, storage_is_mysql};

    #[tokio::test]
    async fn storage_type_test() {
        assert!(storage_is_journal(&"journal".to_string()));
        assert!(storage_is_memory(&"memory".to_string()));
        assert!(storage_is_mysql(&"mysql".to_string()));
    }
}
