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

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common_config::broker::broker_config;
use grpc_clients::meta::kafka::call::get_coordinator_leader;
use kafka_protocol::error::ResponseError;
use protocol::meta::meta_service_kafka::GetCoordinatorLeaderRequest;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

// Every group request checks whether this node is the coordinator; cache the
// leader briefly so heartbeats don't turn into one meta grpc call each. A stale
// answer self-corrects: clients react to NOT_COORDINATOR by re-locating.
const COORDINATOR_CACHE_TTL: Duration = Duration::from_secs(3);
static COORDINATOR_LEADER: Mutex<Option<(u64, Instant)>> = Mutex::new(None);

pub async fn coordinator_node_id(sdm: &Arc<StorageDriverManager>) -> Option<u64> {
    if let Some((node_id, fetched_at)) = *COORDINATOR_LEADER.lock().unwrap() {
        if fetched_at.elapsed() < COORDINATOR_CACHE_TTL {
            return Some(node_id);
        }
    }

    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();
    let reply = get_coordinator_leader(client_pool, &addrs, GetCoordinatorLeaderRequest {})
        .await
        .map_err(|e| warn!("Kafka: failed to get coordinator leader: {}", e))
        .ok()?;
    if !reply.has_leader {
        return None;
    }
    *COORDINATOR_LEADER.lock().unwrap() = Some((reply.leader_node_id, Instant::now()));
    Some(reply.leader_node_id)
}

pub async fn is_coordinator_node(sdm: &Arc<StorageDriverManager>) -> bool {
    coordinator_node_id(sdm).await == Some(broker_config().broker_id)
}

// The coordinator's (node_id, host, port) as seen by Kafka clients.
pub async fn resolve_group_coordinator(
    sdm: &Arc<StorageDriverManager>,
) -> Result<(i32, String, i32), i16> {
    let node_id = coordinator_node_id(sdm)
        .await
        .ok_or_else(|| ResponseError::CoordinatorNotAvailable.code())?;

    let node = sdm
        .broker_cache
        .node_lists
        .get(&node_id)
        .map(|n| n.clone())
        .ok_or_else(|| ResponseError::CoordinatorNotAvailable.code())?;
    let (host, port) = split_host_port(&node.extend.kafka.tcp_addr)
        .ok_or_else(|| ResponseError::CoordinatorNotAvailable.code())?;

    Ok((node.node_id as i32, host, port))
}

pub(crate) fn split_host_port(addr: &str) -> Option<(String, i32)> {
    let (host, port) = addr.rsplit_once(':')?;
    let port = port.parse::<i32>().ok()?;
    Some((host.to_string(), port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_host_port_parses_valid_addr() {
        assert_eq!(
            split_host_port("127.0.0.1:9092"),
            Some(("127.0.0.1".to_string(), 9092))
        );
    }

    #[test]
    fn split_host_port_rejects_invalid_input() {
        assert_eq!(split_host_port("no-port-here"), None);
        assert_eq!(split_host_port("127.0.0.1:abc"), None);
    }
}
