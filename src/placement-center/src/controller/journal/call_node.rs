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

use std::sync::Arc;

use grpc_clients::journal::inner::call::journal_inner_update_cache;
use grpc_clients::poll::ClientPool;
use log::{debug, error, info};
use metadata_struct::journal::segment::JournalSegment;
use metadata_struct::journal::shard::JournalShard;
use metadata_struct::placement::node::BrokerNode;
use protocol::journal_server::journal_inner::{
    JournalUpdateCacheActionType, JournalUpdateCacheResourceType, UpdateJournalCacheRequest,
};

use crate::cache::placement::PlacementCacheManager;

pub fn update_cache_by_add_journal_node(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
    node: BrokerNode,
) {
    tokio::spawn(async move {
        let data = match serde_json::to_vec(&node) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        call_journal_update_cache(
            &cluster_name,
            &placement_cache_manager,
            &client_poll,
            JournalUpdateCacheActionType::Add,
            JournalUpdateCacheResourceType::JournalNode,
            data,
        )
        .await;
    });
}

pub fn update_cache_by_delete_journal_node(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
    node: BrokerNode,
) {
    tokio::spawn(async move {
        let data = match serde_json::to_vec(&node) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        call_journal_update_cache(
            &cluster_name,
            &placement_cache_manager,
            &client_poll,
            JournalUpdateCacheActionType::Delete,
            JournalUpdateCacheResourceType::JournalNode,
            data,
        )
        .await;
    });
}

pub fn update_cache_by_add_shard(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
    shard_info: JournalShard,
) {
    tokio::spawn(async move {
        let data = match serde_json::to_vec(&shard_info) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        call_journal_update_cache(
            &cluster_name,
            &placement_cache_manager,
            &client_poll,
            JournalUpdateCacheActionType::Add,
            JournalUpdateCacheResourceType::Shard,
            data,
        )
        .await;
    });
}

pub fn update_cache_by_delete_shard(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
    shard_info: JournalShard,
) {
    tokio::spawn(async move {
        let data = match serde_json::to_vec(&shard_info) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        call_journal_update_cache(
            &cluster_name,
            &placement_cache_manager,
            &client_poll,
            JournalUpdateCacheActionType::Delete,
            JournalUpdateCacheResourceType::Shard,
            data,
        )
        .await;
    });
}

pub fn update_cache_by_add_segment(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
    segment_info: JournalSegment,
) {
    tokio::spawn(async move {
        let data = match serde_json::to_vec(&segment_info) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        call_journal_update_cache(
            &cluster_name,
            &placement_cache_manager,
            &client_poll,
            JournalUpdateCacheActionType::Add,
            JournalUpdateCacheResourceType::Segment,
            data,
        )
        .await;
    });
}

pub fn update_cache_by_delete_segment(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
    segment_info: JournalSegment,
) {
    tokio::spawn(async move {
        let data = match serde_json::to_vec(&segment_info) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        call_journal_update_cache(
            &cluster_name,
            &placement_cache_manager,
            &client_poll,
            JournalUpdateCacheActionType::Delete,
            JournalUpdateCacheResourceType::Segment,
            data,
        )
        .await;
    });
}

pub async fn call_journal_update_cache(
    cluster_name: &str,
    placement_cache_manager: &Arc<PlacementCacheManager>,
    client_poll: &Arc<ClientPool>,
    action_type: JournalUpdateCacheActionType,
    resource_type: JournalUpdateCacheResourceType,
    data: Vec<u8>,
) {
    info!("{:?},{:?}", action_type, resource_type);
    for addr in placement_cache_manager.get_broker_node_addr_by_cluster(cluster_name) {
        let request = UpdateJournalCacheRequest {
            cluster_name: cluster_name.to_string(),
            action_type: action_type.into(),
            resource_type: resource_type.into(),
            data: data.clone(),
        };
        match journal_inner_update_cache(client_poll.clone(), vec![addr], request).await {
            Ok(resp) => {
                debug!("Calling Journal Engine returns information:{:?}", resp);
            }
            Err(e) => {
                error!("Calling Journal Engine to update cache failed,{}", e);
            }
        };
    }
}
