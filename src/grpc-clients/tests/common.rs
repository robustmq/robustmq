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

// All meta-service nodes in the 3-node cluster started by scripts/cluster.sh /
// ig-test.sh (see config/cluster/server-{1,2,3}.toml). Read-only calls like
// ListAcl are rejected outright by any node that isn't the current metadata
// raft leader (no forwarding, unlike writes) -- passing every node lets the
// gRPC client's own round-robin retry find whichever node is actually leader,
// instead of only ever hitting node 1 and failing whenever leadership has
// moved elsewhere.
pub fn get_placement_addr() -> Vec<String> {
    vec![
        "127.0.0.1:1228".to_string(),
        "127.0.0.1:2228".to_string(),
        "127.0.0.1:3228".to_string(),
    ]
}

// Poll `check` until it returns true or the deadline (15s) elapses.
// Metadata create/delete propagates asynchronously to the node a read may hit,
// so a read issued right after a successful write can momentarily see stale state.
#[allow(dead_code)]
pub async fn wait_until<F, Fut>(mut check: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(15);
    loop {
        if check().await {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }
}
