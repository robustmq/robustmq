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

use common_base::tools::unique_id;
use protocol::placement_center::generate::common::ClusterType;

#[warn(dead_code)]
pub fn pc_addr() -> String {
    return "http://127.0.0.1:1228".to_string();
}

#[warn(dead_code)]
pub fn shard_name() -> String {
    return "test1".to_string();
}

#[warn(dead_code)]
pub fn shard_replica() -> u32 {
    return 1;
}

#[warn(dead_code)]
pub fn cluster_type() -> i32 {
    return ClusterType::JournalServer.into();
}

#[warn(dead_code)]
pub fn cluster_name() -> String {
    return unique_id();
}

#[warn(dead_code)]
pub fn node_id() -> u64 {
    return 4;
}

#[warn(dead_code)]
pub fn node_ip() -> String {
    return "127.0.0.4".to_string();
}

#[warn(dead_code)]
pub fn extend_info() -> String {
    return "extend info".to_string();
}
