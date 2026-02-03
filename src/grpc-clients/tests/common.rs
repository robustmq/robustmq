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

use common_base::uuid::unique_id;

#[allow(dead_code)]
pub fn pc_addr() -> String {
    "http://127.0.0.1:1228".to_string()
}

#[allow(dead_code)]
pub fn shard_name() -> String {
    "test1".to_string()
}

#[allow(dead_code)]
pub fn shard_replica() -> u32 {
    1
}

#[allow(dead_code)]
pub fn cluster_name() -> String {
    unique_id()
}

#[allow(dead_code)]
pub fn namespace() -> String {
    unique_id()
}

#[allow(dead_code)]
pub fn node_id() -> u64 {
    4
}

#[allow(dead_code)]
pub fn node_ip() -> String {
    "127.0.0.4".to_string()
}

#[allow(dead_code)]
pub fn extend_info() -> Vec<u8> {
    Vec::new()
}

#[allow(dead_code)]
pub fn producer_id() -> String {
    "producer id".to_string()
}

#[allow(dead_code)]
pub fn seq_num() -> u64 {
    4
}

pub fn get_placement_addr() -> String {
    "127.0.0.1:1228".to_string()
}
