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

use crate::common::tool::loop_select;
use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::cluster::ClusterStorage;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error, info};

pub async fn register_node(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
) -> ResultMqttBrokerError {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let config = broker_config();
    let node = cluster_storage.register_node(cache_manager, config).await?;
    cache_manager.add_node(node);
    Ok(())
}

pub async fn report_heartbeat(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultMqttBrokerError {
        let cluster_storage = ClusterStorage::new(client_pool.clone());
        if let Err(e) = cluster_storage.heartbeat().await {
            if e.to_string().contains("Node") && e.to_string().contains("does not exist") {
                if let Err(e) = register_node(client_pool, cache_manager).await {
                    error!("{}", e);
                }
            }
            error!("{}", e);
        } else {
            debug!("heartbeat report success");
        }
        Ok(())
    };

    loop_select(ac_fn, 3, &stop_send).await;
}

#[derive(Deserialize, Serialize)]
struct PlacementCenterStatus {
    pub current_leader: u32,
}

pub async fn check_placement_center_status(client_pool: Arc<ClientPool>) {
    let fun = async move || -> Result<Option<bool>, MqttBrokerError> {
        let cluster_storage = ClusterStorage::new(client_pool.clone());
        let data = cluster_storage.place_cluster_status().await?;
        let status = serde_json::from_str::<PlacementCenterStatus>(&data)?;
        if status.current_leader > 0 {
            info!(
                "Placement Center cluster is in normal condition. current leader node is {}.",
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
