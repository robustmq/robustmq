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
use crate::raft::manager::MultiRaftManager;
use crate::{controller::call_broker::mqtt::BrokerCallManager, core::cache::CacheManager};
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_common::UnRegisterNodeRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeHeartbeatData {
    pub node_id: u64,
    pub time: u64,
}

pub struct BrokerHeartbeat {
    timeout_ms: u64,
    cluster_cache: Arc<CacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    client_pool: Arc<ClientPool>,
    mqtt_call_manager: Arc<BrokerCallManager>,
}

impl BrokerHeartbeat {
    pub fn new(
        timeout_ms: u64,
        cluster_cache: Arc<CacheManager>,
        raft_manager: Arc<MultiRaftManager>,
        client_pool: Arc<ClientPool>,
        mqtt_call_manager: Arc<BrokerCallManager>,
    ) -> Self {
        BrokerHeartbeat {
            timeout_ms,
            cluster_cache,
            raft_manager,
            client_pool,
            mqtt_call_manager,
        }
    }

    pub async fn start(&self) {
        for node in self.cluster_cache.node_list.iter() {
            if let Some(heart_data) = self.cluster_cache.get_broker_heart(node.node_id) {
                let now_time = now_second();
                if now_time - heart_data.time >= self.timeout_ms / 1000 {
                    let req = UnRegisterNodeRequest {
                        node_id: node.node_id,
                    };

                    if let Err(e) = un_register_node_by_req(
                        &self.cluster_cache,
                        &self.raft_manager,
                        &self.client_pool,
                        &self.mqtt_call_manager,
                        req,
                    )
                    .await
                    {
                        error!("Heartbeat timeout, failed to delete node {} , error message :{},now time:{},report time:{}",
                             node.node_id,e,now_time,heart_data.time);
                        continue;
                    }

                    info!(
                            "Heartbeat of the Node times out and is deleted from the cluster. Node ID: {}, node IP: {},now time:{},report time:{}, diff:{}, time_ms:{}",
                            node.node_id, node.node_ip, now_time, heart_data.time, (now_time - heart_data.time), self.timeout_ms
                        );
                }
            } else {
                self.cluster_cache.report_broker_heart(node.node_id);
            }
        }
    }
}
