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

use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use log::{error, info};
use metadata_struct::placement::node::str_to_cluster_type;
use protocol::placement_center::placement_center_inner::UnRegisterNodeRequest;
use serde::{Deserialize, Serialize};

use super::cluster::un_register_node_by_req;
use crate::core::cache::PlacementCacheManager;
use crate::journal::controller::call_node::JournalInnerCallManager;
use crate::mqtt::controller::call_broker::MQTTInnerCallManager;
use crate::route::apply::RaftMachineApply;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeHeartbeatData {
    pub cluster_name: String,
    pub node_id: u64,
    pub time: u64,
}

pub struct BrokerHeartbeat {
    timeout_ms: u64,
    cluster_cache: Arc<PlacementCacheManager>,
    raft_machine_apply: Arc<RaftMachineApply>,
    client_pool: Arc<ClientPool>,
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
}

impl BrokerHeartbeat {
    pub fn new(
        timeout_ms: u64,
        cluster_cache: Arc<PlacementCacheManager>,
        raft_machine_apply: Arc<RaftMachineApply>,
        client_pool: Arc<ClientPool>,
        journal_call_manager: Arc<JournalInnerCallManager>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
    ) -> Self {
        BrokerHeartbeat {
            timeout_ms,
            cluster_cache,
            raft_machine_apply,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
        }
    }

    pub async fn start(&mut self) {
        for cluster_name in self.cluster_cache.get_all_cluster_name() {
            for node in self.cluster_cache.get_broker_node_by_cluster(&cluster_name) {
                if let Some(heart_data) = self
                    .cluster_cache
                    .get_broker_heart(&cluster_name, node.node_id)
                {
                    let now_time = now_second();
                    if now_time - heart_data.time >= self.timeout_ms / 1000
                        && self.cluster_cache.get_cluster(&cluster_name).is_some()
                    {
                        let cluster_type = str_to_cluster_type(&node.cluster_type).unwrap();
                        let req = UnRegisterNodeRequest {
                            node_id: node.node_id,
                            cluster_name: cluster_name.to_string(),
                            cluster_type: cluster_type.into(),
                        };

                        if let Err(e) = un_register_node_by_req(
                            &self.cluster_cache,
                            &self.raft_machine_apply,
                            &self.client_pool,
                            &self.journal_call_manager,
                            &self.mqtt_call_manager,
                            req,
                        )
                        .await
                        {
                            error!("Heartbeat timeout, failed to delete node {} in cluster {}, error message :{},now time:{},report time:{}",
                             node.node_id,cluster_name,e,now_time,heart_data.time);
                            continue;
                        }

                        self.cluster_cache
                            .remove_broker_heart(&cluster_name, node.node_id);
                        info!(
                            "Heartbeat of the Node times out and is deleted from the cluster. Node ID: {}, node IP: {},now time:{},report time:{}",
                            node.node_id, node.node_ip, now_time, heart_data.time
                        );
                    }
                } else {
                    self.cluster_cache
                        .report_broker_heart(&cluster_name, node.node_id);
                }
            }
        }
    }
}
