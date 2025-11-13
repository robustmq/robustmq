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

// metadata service
pub const DB_COLUMN_FAMILY_META_RAFT: &str = "meta_raft";
pub const DB_COLUMN_FAMILY_META_DATA: &str = "meta_data";
pub const DB_COLUMN_FAMILY_META_METADATA: &str = "meta_metadata";

// broker service
pub const DB_COLUMN_FAMILY_BROKER: &str = "broker";

// journal engine
pub const DB_COLUMN_FAMILY_JOURNAL: &str = "journal";

pub fn column_family_list() -> Vec<String> {
    vec![
        DB_COLUMN_FAMILY_META_RAFT.to_string(),
        DB_COLUMN_FAMILY_META_DATA.to_string(),
        DB_COLUMN_FAMILY_META_METADATA.to_string(),
        DB_COLUMN_FAMILY_BROKER.to_string(),
        DB_COLUMN_FAMILY_JOURNAL.to_string(),
    ]
}

pub fn storage_data_fold(path: &str) -> String {
    let mut result = String::with_capacity(path.len() + 6);
    result.push_str(path);
    result.push_str("/_data");
    result
}

pub fn storage_raft_fold(path: &str) -> String {
    let mut result = String::with_capacity(path.len() + 6);
    result.push_str(path);
    result.push_str("/_raft");
    result
}

#[cfg(test)]
mod tests {
    use crate::storage::family::{column_family_list, storage_data_fold, storage_raft_fold};

    #[tokio::test]
    async fn column_family_list_test() {
        let list = column_family_list();
        assert_eq!(list.len(), 3);
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
