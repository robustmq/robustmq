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

pub use rocksdb_engine::RocksDBEngine;

pub const DB_COLUMN_FAMILY_META: &str = "meta";
pub const DB_COLUMN_FAMILY_META_RAFT_LOG: &str = "meta_raft_log";
pub const DB_COLUMN_FAMILY_META_RAFT_STORE: &str = "meta_raft_store";
pub const DB_COLUMN_FAMILY_BROKER: &str = "broker";

pub fn column_family_list() -> Vec<String> {
    vec![
        DB_COLUMN_FAMILY_META.to_string(),
        DB_COLUMN_FAMILY_BROKER.to_string(),
    ]
}

pub fn raft_column_family_list() -> Vec<String> {
    vec![
        DB_COLUMN_FAMILY_META_RAFT_LOG.to_string(),
        DB_COLUMN_FAMILY_META_RAFT_STORE.to_string(),
    ]
}

pub fn storage_data_fold(path: &str) -> String {
    format!("{path}/_data")
}

pub fn storage_raft_fold(path: &str) -> String {
    format!("{path}/_raft")
}

#[cfg(test)]
mod tests {
    use crate::rocksdb::{column_family_list, storage_data_fold, storage_raft_fold};

    #[tokio::test]
    async fn column_family_list_test() {
        let list = column_family_list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "meta");
    }

    #[tokio::test]
    async fn storage_data_fold_test() {
        let path = "/tmp/test";
        let fold = storage_data_fold(path);
        assert_eq!(fold, "/tmp/test/_data");
    }

    #[tokio::test]
    async fn storage_raft_fold_test() {
        let path = "/tmp/test";
        let fold = storage_raft_fold(path);
        assert_eq!(fold, "/tmp/test/_raft");
    }
}
