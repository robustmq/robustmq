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

use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::loop_select_ticket,
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

use crate::{cache::BrokerCacheManager, cluster::ClusterStorage};

pub async fn register_node(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<BrokerCacheManager>,
) -> ResultCommonError {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let config = broker_config();
    let node = cluster_storage.register_node(cache_manager, config).await?;
    cache_manager.add_node(node);
    Ok(())
}

pub async fn report_heartbeat(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<BrokerCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        let cluster_storage = ClusterStorage::new(client_pool.clone());
        let config = broker_config();

        // Send heartbeat with 1 second timeout
        match timeout(Duration::from_secs(3), cluster_storage.heartbeat()).await {
            Ok(Ok(())) => {
                debug!("Heartbeat report success for node {}", config.broker_id);
            }
            Ok(Err(e)) => {
                // Heartbeat failed
                if e.to_string().contains("Node") && e.to_string().contains("does not exist") {
                    warn!(
                        "Node {} does not exist in Meta Service, attempting to re-register",
                        config.broker_id
                    );
                    if let Err(register_err) = register_node(client_pool, cache_manager).await {
                        error!(
                            "Failed to re-register node {} after heartbeat failure: {}",
                            config.broker_id, register_err
                        );
                    } else {
                        info!("Node {} successfully re-registered", config.broker_id);
                    }
                }
                error!(
                    "Heartbeat failed for node {} ({}:{}): {}",
                    config.broker_id,
                    config.broker_ip.as_deref().unwrap_or("unknown"),
                    config.grpc_port,
                    e
                );
            }
            Err(_) => {
                error!(
                    "Heartbeat timeout (3s) for node {} ({}:{}), Meta Service may be unresponsive",
                    config.broker_id,
                    config.broker_ip.as_deref().unwrap_or("unknown"),
                    config.grpc_port
                );
            }
        }
        Ok(())
    };

    loop_select_ticket(ac_fn, 3000, &stop_send).await;
}

#[derive(Deserialize, Serialize, Debug)]
struct MetaServiceStatus {
    pub running_state: serde_json::Value,
    pub current_leader: u64,
}

impl MetaServiceStatus {
    /// A state machine is ready when:
    /// 1. `running_state` is `{"Ok": ...}` (no error)
    /// 2. `current_leader != 0` (a leader has been elected)
    fn is_ready(&self) -> bool {
        let running_ok = self
            .running_state
            .as_object()
            .map(|m| m.contains_key("Ok"))
            .unwrap_or(false);
        running_ok && self.current_leader != 0
    }
}

pub async fn check_meta_service_status(client_pool: Arc<ClientPool>) {
    let fun = async move || -> Result<Option<bool>, CommonError> {
        let cluster_storage = ClusterStorage::new(client_pool.clone());
        let data = cluster_storage.meta_cluster_status().await?;

        // The status JSON is a map of raft shard name -> MetaServiceStatus,
        // e.g. {"metadata_0": {...}, "offset_3": {...}, "data_7": {...}}.
        // The cluster is ready only when every shard is ready.
        let shard_statuses: std::collections::BTreeMap<String, MetaServiceStatus> =
            serde_json::from_str(&data)?;

        if shard_statuses.is_empty() {
            return Ok(None);
        }

        let not_ready: Vec<String> = shard_statuses
            .iter()
            .filter(|(_, s)| !s.is_ready())
            .map(|(name, s)| {
                format!(
                    "{}(running_state={}, leader={})",
                    name, s.running_state, s.current_leader
                )
            })
            .collect();

        if not_ready.is_empty() {
            let summary: Vec<String> = shard_statuses
                .iter()
                .map(|(name, s)| format!("{}→node{}", name, s.current_leader))
                .collect();
            info!(
                "Meta Service cluster is ready. All raft shards have elected a leader: [{}]",
                summary.join(", ")
            );
            return Ok(Some(true));
        }

        let total = shard_statuses.len();
        let ready = total - not_ready.len();
        info!(
            "Meta Service cluster not ready yet: {}/{} shards ready. Not ready: {:?}",
            ready, total, not_ready
        );
        Ok(None)
    };

    loop {
        match fun().await {
            Ok(Some(true)) => break,
            Ok(_) => {
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                info!("Meta Service cluster is not yet ready: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
