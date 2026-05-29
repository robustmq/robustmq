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
    error::ResultCommonError,
    task::{TaskKind, TaskSupervisor},
    tools::loop_select_ticket,
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

use crate::{cache::NodeCacheManager, cluster::ClusterStorage};

pub async fn register_node(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<NodeCacheManager>,
) -> ResultCommonError {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let config = broker_config();
    let node = cluster_storage.register_node(cache_manager, config).await?;
    cache_manager.add_node(node);
    Ok(())
}

pub async fn register_node_and_start_heartbeat(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<NodeCacheManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    stop_send: broadcast::Sender<bool>,
) {
    let config = broker_config();
    match register_node(client_pool, cache_manager).await {
        Ok(()) => {
            let raw_client_pool = client_pool.clone();
            let broker_cache = cache_manager.clone();

            task_supervisor.spawn(
                TaskKind::BrokerNodeHeartbeat.to_string(),
                Box::pin(async move {
                    report_heartbeat(&raw_client_pool, &broker_cache, stop_send).await;
                }),
            );
            info!("Node {} has been successfully registered", config.broker_id);
        }
        Err(e) => {
            error!("Node registration failed. Error message:{}", e);
        }
    }
}

pub async fn report_heartbeat(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<NodeCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let config = broker_config();
    info!(
        "Heartbeat task started for node {}, reporting every 3s",
        config.broker_id
    );

    let ac_fn = async || -> ResultCommonError {
        let cluster_storage = ClusterStorage::new(client_pool.clone());
        let config = broker_config();

        match timeout(Duration::from_secs(3), cluster_storage.heartbeat()).await {
            Ok(Ok(())) => {
                debug!("Heartbeat report success for node {}", config.broker_id);
            }
            Ok(Err(e)) => {
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
                        return Ok(());
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

pub async fn check_meta_service_status(client_pool: Arc<ClientPool>) {
    loop {
        let cluster_storage = ClusterStorage::new(client_pool.clone());
        match cluster_storage.raft_ping().await {
            Ok(()) => {
                info!("Meta Service cluster is ready");
                break;
            }
            Err(e) => {
                info!(
                    "Waiting for Meta Service cluster to be ready ({}), retrying in 1s...",
                    e
                );
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
