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
use crate::controller::journal::call_node::JournalInnerCallManager;
use crate::controller::mqtt::call_broker::MQTTInnerCallManager;
use crate::core::cache::CacheManager;
use crate::raft::manager::MultiRaftManager;

use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct ClusterController {
    cluster_cache: Arc<CacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    stop_send: broadcast::Sender<bool>,
    client_pool: Arc<ClientPool>,
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
}

impl ClusterController {
    pub fn new(
        cluster_cache: Arc<CacheManager>,
        raft_manager: Arc<MultiRaftManager>,
        stop_send: broadcast::Sender<bool>,
        client_pool: Arc<ClientPool>,
        journal_call_manager: Arc<JournalInnerCallManager>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
    ) -> ClusterController {
        ClusterController {
            cluster_cache,
            raft_manager,
            stop_send,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
        }
    }

    // Start the heartbeat detection thread of the Storage Engine node
    pub async fn start_node_heartbeat_check(&self) {
        let config = broker_config();
        let heartbeat = BrokerHeartbeat::new(
            config.meta_runtime.heartbeat_timeout_ms,
            self.cluster_cache.clone(),
            self.raft_manager.clone(),
            self.client_pool.clone(),
            self.journal_call_manager.clone(),
            self.mqtt_call_manager.clone(),
        );

        let ac_fn = async || -> ResultCommonError {
            heartbeat.start().await;
            Ok(())
        };

        loop_select_ticket(
            ac_fn,
            config.meta_runtime.heartbeat_check_time_ms / 1000,
            &self.stop_send,
        )
        .await;
    }
}
