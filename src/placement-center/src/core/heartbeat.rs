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
use std::thread::sleep;
use std::time::Duration;

use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use log::{error, info};
use metadata_struct::placement::node::str_to_cluster_type;
use protocol::placement_center::placement_center_inner::UnRegisterNodeRequest;
use serde::{Deserialize, Serialize};

use super::cluster::un_register_node_by_req;
use crate::core::cache::PlacementCacheManager;
use crate::journal::controller::call_node::JournalInnerCallManager;
use crate::route::apply::RaftMachineApply;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeHeartbeatData {
    pub cluster_name: String,
    pub node_id: u64,
    pub time: u64,
}

pub struct BrokerHeartbeat {
    timeout_ms: u64,
    check_time_ms: u64,
    cluster_cache: Arc<PlacementCacheManager>,
    raft_machine_apply: Arc<RaftMachineApply>,
    client_pool: Arc<ClientPool>,
    call_manager: Arc<JournalInnerCallManager>,
}

impl BrokerHeartbeat {
    pub fn new(
        timeout_ms: u64,
        check_time_ms: u64,
        cluster_cache: Arc<PlacementCacheManager>,
        raft_machine_apply: Arc<RaftMachineApply>,
        client_pool: Arc<ClientPool>,
        call_manager: Arc<JournalInnerCallManager>,
    ) -> Self {
        BrokerHeartbeat {
            timeout_ms,
            check_time_ms,
            cluster_cache,
            raft_machine_apply,
            client_pool,
            call_manager,
        }
    }

    pub async fn start(&mut self) {
        let node_list = self.cluster_cache.node_list.clone();

        for (cluster_name, node_list) in node_list.clone() {
            for (node_id, node) in node_list {
                if let Some(heart_data) =
                    self.cluster_cache.get_broker_heart(&cluster_name, node_id)
                {
                    if now_second() - heart_data.time >= self.timeout_ms / 1000
                        && self.cluster_cache.cluster_list.get(&cluster_name).is_some()
                    {
                        let cluster_type = str_to_cluster_type(&node.cluster_type).unwrap();
                        let req = UnRegisterNodeRequest {
                            node_id,
                            cluster_name: cluster_name.to_string(),
                            cluster_type: cluster_type.into(),
                        };

                        if let Err(e) = un_register_node_by_req(
                            &self.cluster_cache,
                            &self.raft_machine_apply,
                            &self.client_pool,
                            &self.call_manager,
                            req,
                        )
                        .await
                        {
                            error!("Heartbeat timeout, failed to delete node {} in cluster {}, error message :{}", node_id,cluster_name,e);
                            continue;
                        }

                        self.cluster_cache
                            .remove_broker_heart(&cluster_name, node_id);
                        info!(
                                    "The heartbeat of the Node times out and is deleted from the cluster. Node ID: {}, node IP: {}.",
                                     node_id,
                                     node.node_ip);
                    }
                } else {
                    self.cluster_cache
                        .report_broker_heart(&cluster_name, node_id);
                }
            }
        }
        sleep(Duration::from_millis(self.check_time_ms));
    }
}
