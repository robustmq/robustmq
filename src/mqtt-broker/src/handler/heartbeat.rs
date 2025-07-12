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
use std::time::Duration;

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::storage::cluster::ClusterStorage;

use common_config::mqtt::broker_mqtt_conf;
use common_config::mqtt::default::default_heartbeat_timeout;
use grpc_clients::pool::ClientPool;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout};
use tracing::{debug, error};

pub async fn register_node(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
) -> ResultMqttBrokerError {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let config = broker_mqtt_conf();
    let node = cluster_storage.register_node(cache_manager, config).await?;
    cache_manager.add_node(node);
    Ok(())
}

pub async fn report_heartbeat(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    heartbeat_timeout: &String,
    stop_send: broadcast::Sender<bool>,
) {
    let actual_timeout = match humantime::parse_duration(heartbeat_timeout) {
        Ok(v) => v.as_secs(),
        Err(e) => {
            error!(
                "Failed to parse '{}' heartbeat timeout by err: {}",
                heartbeat_timeout, e
            );

            humantime::parse_duration(default_heartbeat_timeout().as_ref())
                .unwrap()
                .as_secs()
        }
    };

    loop {
        let mut stop_recv = stop_send.subscribe();
        select! {
            val = stop_recv.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        debug!("{}","Heartbeat reporting thread exited successfully");
                        break;
                    }
                }
            }
            val = timeout(Duration::from_secs(actual_timeout),report(client_pool,cache_manager)) => {
                if let Err(e) = val{
                    error!("Broker heartbeat report timeout, error message:{}",e);
                }
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn report(client_pool: &Arc<ClientPool>, cache_manager: &Arc<CacheManager>) {
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
}
