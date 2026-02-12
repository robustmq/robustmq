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

use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use common_base::error::common::CommonError;
use protocol::meta::meta_service_mqtt::{GetShareSubLeaderReply, GetShareSubLeaderRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub fn generate_group_leader(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    group_name: &str,
) -> Result<u64, CommonError> {
    if let Some(leader) = cache_manager.group_leader.get(group_name) {
        if cache_manager.node_list.contains_key(&leader.broker_id) {
            return Ok(leader.broker_id);
        }
    }

    let mut broker_ids: Vec<u64> = cache_manager
        .node_list
        .iter()
        .map(|node| node.node_id)
        .collect();

    broker_ids.sort();

    let mut target_broker_id = 0;
    let mut cur_len = 0;
    for broker_id in broker_ids {
        let size = if let Some(list) = node_sub_info.get(&broker_id) {
            list.len()
        } else {
            0
        };

        if target_broker_id == 0 {
            target_broker_id = broker_id;
            cur_len = size;
            continue;
        }

        if size < cur_len {
            target_broker_id = broker_id;
            cur_len = size;
        }
    }

    if target_broker_id == 0 {
        return Err(CommonError::ClusterNoAvailableNode);
    }

    Ok(target_broker_id)
}

pub fn get_share_sub_leader_by_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetShareSubLeaderRequest,
) -> Result<GetShareSubLeaderReply, MetaServiceError> {
    let share_sub = ShareSubLeader::new(cache_manager.clone(), rocksdb_engine_handler.clone());

    // Get leader broker ID for the shared subscription group
    let leader_broker = share_sub
        .get_leader_node(&req.group_name)
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;

    // Get broker node details from cache
    match cache_manager.get_broker_node(leader_broker) {
        Some(node) => Ok(GetShareSubLeaderReply {
            broker_id: node.node_id,
            broker_addr: node.node_ip,
            extend_info: node.extend.encode()?,
        }),
        None => Err(MetaServiceError::NoAvailableBrokerNode),
    }
}
