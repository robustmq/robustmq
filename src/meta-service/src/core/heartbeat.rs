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

use crate::core::{cache::MetaCacheManager, cluster::remove_node};
use crate::raft::manager::MultiRaftManager;
use common_base::tools::now_second;
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
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
    cluster_cache: Arc<MetaCacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    node_call_manager: Arc<NodeCallManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl BrokerHeartbeat {
    pub fn new(
        timeout_ms: u64,
        cluster_cache: Arc<MetaCacheManager>,
        raft_manager: Arc<MultiRaftManager>,
        node_call_manager: Arc<NodeCallManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        BrokerHeartbeat {
            timeout_ms,
            cluster_cache,
            raft_manager,
            node_call_manager,
            rocksdb_engine_handler,
        }
    }

    pub async fn start(&self) {
        // Collect decisions first to release the DashMap iter lock before any .await.
        // Holding iter() across an .await that triggers remove() on the same map causes
        // a deadlock: the read shard lock is never released, so write lock can't proceed.
        let actions = self.collect_expired_nodes();
        self.process_expired_nodes(actions).await;
    }

    // Step 1: snapshot node states while holding iter() lock, then release immediately.
    fn collect_expired_nodes(&self) -> Vec<NodeAction> {
        let now_time = now_second();
        self.cluster_cache
            .node_list
            .iter()
            .map(|entry| {
                let node_id = entry.node_id;
                let node_ip = entry.node_ip.clone();
                if let Some(heart_data) = self.cluster_cache.get_broker_heart(node_id) {
                    NodeAction {
                        node_id,
                        node_ip,
                        expired: now_time - heart_data.time >= self.timeout_ms / 1000,
                        now_time,
                        report_time: heart_data.time,
                    }
                } else {
                    NodeAction {
                        node_id,
                        node_ip,
                        expired: false,
                        now_time,
                        report_time: 0,
                    }
                }
            })
            .collect()
    }

    // Step 2: iter() lock already released; safe to .await and mutate node_list.
    async fn process_expired_nodes(&self, actions: Vec<NodeAction>) {
        for action in actions {
            if action.report_time == 0 {
                self.cluster_cache.report_broker_heart(action.node_id);
                continue;
            }

            if action.expired {
                if let Err(e) = remove_node(
                    &self.cluster_cache,
                    &self.raft_manager,
                    &self.rocksdb_engine_handler,
                    &self.node_call_manager,
                    action.node_id,
                )
                .await
                {
                    error!(
                        "Heartbeat timeout, failed to delete node {} , error message :{},now time:{},report time:{}",
                        action.node_id, e, action.now_time, action.report_time
                    );
                    continue;
                }

                info!(
                    "Heartbeat of the Node times out and is deleted from the cluster. Node ID: {}, node IP: {},now time:{},report time:{}, diff:{}, time_ms:{}",
                    action.node_id, action.node_ip, action.now_time, action.report_time,
                    (action.now_time - action.report_time), self.timeout_ms
                );
            }
        }
    }
}

struct NodeAction {
    node_id: u64,
    node_ip: String,
    expired: bool,
    now_time: u64,
    report_time: u64,
}
