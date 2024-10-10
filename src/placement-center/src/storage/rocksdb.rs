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

pub const DB_COLUMN_FAMILY_CLUSTER: &str = "cluster";

pub fn column_family_list() -> Vec<String> {
    let mut list = Vec::new();
    list.push(DB_COLUMN_FAMILY_CLUSTER.to_string());
    list
}

pub fn storage_data_fold(path: &str) -> String {
    format!("{}/_data", path)
}

pub fn storage_raft_fold(path: &str) -> String {
    format!("{}/_raft", path)
}
