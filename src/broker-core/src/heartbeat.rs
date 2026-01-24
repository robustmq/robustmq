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
    pub current_leader: u32,
}

pub async fn check_meta_service_status(client_pool: Arc<ClientPool>) {
    let fun = async move || -> Result<Option<bool>, CommonError> {
        let cluster_storage = ClusterStorage::new(client_pool.clone());
        let data = cluster_storage.meta_cluster_status().await?;
        let status = serde_json::from_str::<MetaServiceStatus>(&data)?;
        if status.current_leader > 0 {
            info!(
                "Meta Service cluster is in normal condition. current leader node is {}.",
                status.current_leader
            );
            return Ok(Some(true));
        };
        Ok(None)
    };

    loop {
        match fun().await {
            Ok(bol) => {
                if let Some(b) = bol {
                    if b {
                        break;
                    }
                }

                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                debug!(" cluster is not yet ready. Error message: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
