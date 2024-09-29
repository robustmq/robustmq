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
    cache::placement::PlacementCacheManager,
    raft::apply::{RaftMachineApply, StorageData, StorageDataType},
};
use common_base::tools::now_second;
use log::{error, info};
use prost::Message;
use protocol::placement_center::generate::{common::ClusterType, placement::UnRegisterNodeRequest};
use std::{sync::Arc, thread::sleep, time::Duration};

pub struct BrokerHeartbeat {
    timeout_ms: u64,
    check_time_ms: u64,
    cluster_cache: Arc<PlacementCacheManager>,
    placement_center_storage: Arc<RaftMachineApply>,
}

impl BrokerHeartbeat {
    pub fn new(
        timeout_ms: u64,
        check_time_ms: u64,
        cluster_cache: Arc<PlacementCacheManager>,
        placement_center_storage: Arc<RaftMachineApply>,
    ) -> Self {
        return BrokerHeartbeat {
            timeout_ms,
            check_time_ms,
            cluster_cache,
            placement_center_storage,
        };
    }

    pub async fn start(&mut self) {
        for (cluster_name, node_list) in self.cluster_cache.node_list.clone() {
            for (node_id, node) in node_list.clone() {
                if !self
                    .cluster_cache
                    .node_heartbeat
                    .contains_key(&cluster_name)
                {
                    continue;
                }
                if let Some(cluster_heartbeat) =
                    self.cluster_cache.node_heartbeat.get(&cluster_name)
                {
                    if let Some(time) = cluster_heartbeat.get(&node_id) {
                        if now_second() - time.clone() >= self.timeout_ms / 1000 {
                            let cluster_name = node.cluster_name.clone();
                            if let Some(_) = self.cluster_cache.cluster_list.get(&cluster_name) {
                                let mut req = UnRegisterNodeRequest::default();
                                req.node_id = node.node_id;
                                req.cluster_name = node.cluster_name.clone();
                                req.cluster_type = ClusterType::JournalServer.into();
                                let pcs = self.placement_center_storage.clone();
                                let data = StorageData::new(
                                    StorageDataType::ClusterUngisterNode,
                                    UnRegisterNodeRequest::encode_to_vec(&req),
                                );
                                tokio::spawn(async move {
                                    match pcs
                                        .apply_propose_message(
                                            data,
                                            "heartbeat_remove_node".to_string(),
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            info!(
                                                   "The heartbeat of the Storage Engine node times out and is deleted from the cluster. Node ID: {}, node IP: {}.",
                                                    node.node_id,
                                                    node.node_ip);
                                        }
                                        Err(e) => {
                                            error!("{}", e);
                                        }
                                    }
                                });
                            }
                        }
                    } else {
                        self.cluster_cache
                            .report_heart_by_broker_node(&cluster_name, node_id, now_second());
                    }
                } else {
                    self.cluster_cache
                        .report_heart_by_broker_node(&cluster_name, node_id, now_second());
                }
            }
        }
        sleep(Duration::from_millis(self.check_time_ms));
    }
}
