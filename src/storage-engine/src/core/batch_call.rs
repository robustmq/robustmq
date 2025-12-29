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

use crate::{
    clients::{manager::ClientConnectionManager, packet::read_resp_parse},
    core::{cache::StorageCacheManager, error::StorageEngineError},
};
use futures::future::join_all;
use metadata_struct::storage::storage_record::StorageRecord;
use protocol::storage::{codec::StorageEnginePacket, protocol::ReadReq};
use std::{sync::Arc, time::Duration};
use tracing::warn;

pub async fn call_read_data_by_all_node(
    cache_manager: &Arc<StorageCacheManager>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    target_broker_id: u64,
    read_req: ReadReq,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let remote_nodes = get_remote_leader_nodes(cache_manager, &read_req, target_broker_id);

    if remote_nodes.is_empty() {
        return Ok(Vec::new());
    }

    let tasks: Vec<_> = remote_nodes
        .into_iter()
        .map(|node_id| {
            let client_conn = client_connection_manager.clone();
            let req = read_req.clone();
            async move {
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    read_by_remote(&client_conn, node_id, req),
                )
                .await
                {
                    Ok(result) => result,
                    Err(_) => {
                        warn!("Read timeout from node {}", node_id);
                        Err(StorageEngineError::CommonErrorStr(format!(
                            "Timeout reading from node {}",
                            node_id
                        )))
                    }
                }
            }
        })
        .collect();

    let results = join_all(tasks).await;

    let mut all_records: Vec<StorageRecord> = results
        .into_iter()
        .filter_map(|r| {
            if let Err(e) = &r {
                warn!("Failed to read from remote node: {}", e);
            }
            r.ok()
        })
        .flatten()
        .collect();

    all_records.sort_by_key(|r| r.metadata.offset);

    Ok(all_records)
}

fn get_remote_leader_nodes(
    cache_manager: &Arc<StorageCacheManager>,
    read_req: &ReadReq,
    target_broker_id: u64,
) -> Vec<u64> {
    let mut nodes: Vec<u64> = read_req
        .body
        .messages
        .iter()
        .flat_map(|msg| cache_manager.get_segment_leader_nodes(&msg.shard_name))
        .filter(|node_id| *node_id != target_broker_id)
        .collect();

    nodes.sort_unstable();
    nodes.dedup();
    nodes
}

pub fn merge_records(
    mut local_records: Vec<StorageRecord>,
    mut remote_records: Vec<StorageRecord>,
) -> Vec<StorageRecord> {
    local_records.append(&mut remote_records);
    local_records.sort_by_key(|r| r.metadata.offset);
    local_records
}

async fn read_by_remote(
    client_connection_manager: &Arc<ClientConnectionManager>,
    target_broker_id: u64,
    read_req: ReadReq,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let resp = client_connection_manager
        .write_send(target_broker_id, StorageEnginePacket::ReadReq(read_req))
        .await?;

    match resp {
        StorageEnginePacket::ReadResp(resp) => Ok(read_resp_parse(&resp)?),
        packet => Err(StorageEngineError::ReceivedPacketError(
            target_broker_id,
            format!("Expected ReadResp, got {:?}", packet),
        )),
    }
}
