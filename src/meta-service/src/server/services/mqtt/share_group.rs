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

use crate::core::cache::MetaCacheManager;
use crate::core::error::MetaServiceError;
use crate::core::group_leader::get_group_leader;
use crate::raft::manager::MultiRaftManager;
use protocol::meta::meta_service_mqtt::{
    GetShareSubLeaderReply, GetShareSubLeaderRequest, SubLeaderInfo,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn get_share_sub_leader_by_req(
    cache_manager: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetShareSubLeaderRequest,
) -> Result<GetShareSubLeaderReply, MetaServiceError> {
    let mut results = Vec::new();
    for group_name in req.group_list.iter() {
        let leader_broker = get_group_leader(
            raft_manager,
            cache_manager,
            rocksdb_engine_handler,
            &req.tenant,
            group_name,
        )
        .await?;

        let leader = match cache_manager.get_broker_node(leader_broker) {
            Some(node) => SubLeaderInfo {
                group_name: group_name.clone(),
                broker_id: node.node_id,
                broker_addr: node.node_ip,
                extend_info: node.extend.encode()?,
            },
            None => return Err(MetaServiceError::NoAvailableBrokerNode),
        };
        results.push(leader);
    }
    Ok(GetShareSubLeaderReply { leader: results })
}
