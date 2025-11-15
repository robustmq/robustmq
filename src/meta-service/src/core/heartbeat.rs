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

use super::cluster::un_register_node_by_req;
use crate::controller::journal::call_node::JournalInnerCallManager;
use crate::raft::manager::MultiRaftManager;
use crate::{controller::mqtt::call_broker::MQTTInnerCallManager, core::cache::CacheManager};
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_inner::UnRegisterNodeRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeHeartbeatData {
    pub cluster_name: String,
    pub node_id: u64,
    pub time: u64,
}

pub struct BrokerHeartbeat {
    timeout_ms: u64,
    cluster_cache: Arc<CacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    client_pool: Arc<ClientPool>,
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
}

impl BrokerHeartbeat {
    pub fn new(
        timeout_ms: u64,
        cluster_cache: Arc<CacheManager>,
        raft_manager: Arc<MultiRaftManager>,
        client_pool: Arc<ClientPool>,
        journal_call_manager: Arc<JournalInnerCallManager>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
    ) -> Self {
        BrokerHeartbeat {
            timeout_ms,
            cluster_cache,
            raft_manager,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
        }
    }

    pub async fn start(&self) {
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
                        let req = UnRegisterNodeRequest {
                            node_id: node.node_id,
                            cluster_name: cluster_name.to_string(),
                        };

                        if let Err(e) = un_register_node_by_req(
                            &self.cluster_cache,
                            &self.raft_manager,
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

                        info!(
                            "Heartbeat of the Node times out and is deleted from the cluster. Node ID: {}, node IP: {},now time:{},report time:{}, diff:{}, time_ms:{}",
                            node.node_id, node.node_ip, now_time, heart_data.time, (now_time - heart_data.time), self.timeout_ms
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
