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

use super::heartbeat::BrokerHeartbeat;
use crate::core::cache::MetaCacheManager;
use crate::raft::manager::MultiRaftManager;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use node_call::NodeCallManager;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct ClusterController {
    cluster_cache: Arc<MetaCacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    client_pool: Arc<ClientPool>,
    node_call_manager: Arc<NodeCallManager>,
}

impl ClusterController {
    pub fn new(
        cluster_cache: Arc<MetaCacheManager>,
        raft_manager: Arc<MultiRaftManager>,
        client_pool: Arc<ClientPool>,
        node_call_manager: Arc<NodeCallManager>,
    ) -> ClusterController {
        ClusterController {
            cluster_cache,
            raft_manager,
            client_pool,
            node_call_manager,
        }
    }

    pub async fn start_node_heartbeat_check(&self, stop_send: &broadcast::Sender<bool>) {
        let config = broker_config();
        let heartbeat = BrokerHeartbeat::new(
            config.meta_runtime.heartbeat_timeout_ms,
            self.cluster_cache.clone(),
            self.raft_manager.clone(),
            self.client_pool.clone(),
            self.node_call_manager.clone(),
        );

        let ac_fn = async || -> ResultCommonError {
            heartbeat.start().await;
            Ok(())
        };

        loop_select_ticket(
            ac_fn,
            config.meta_runtime.heartbeat_check_time_ms,
            stop_send,
        )
        .await;
    }
}
