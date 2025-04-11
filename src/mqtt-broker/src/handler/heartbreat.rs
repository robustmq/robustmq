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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use log::{debug, error};
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout};

use super::error::MqttBrokerError;
use crate::storage::cluster::ClusterStorage;

pub async fn register_node(client_pool: &Arc<ClientPool>) -> Result<(), MqttBrokerError> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let config = broker_mqtt_conf();
    cluster_storage.register_node(config).await?;
    Ok(())
}

pub async fn report_heartbeat(client_pool: &Arc<ClientPool>, stop_send: broadcast::Sender<bool>) {
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
            val = timeout(Duration::from_secs(10),report(client_pool)) => {
                if let Err(e) = val{
                    error!("Broker heartbeat report timeout, error message:{}",e);
                }
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn report(client_pool: &Arc<ClientPool>) {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    if let Err(e) = cluster_storage.heartbeat().await {
        if e.to_string().contains("Node") && e.to_string().contains("does not exist") {
            if let Err(e) = register_node(client_pool).await {
                error!("{}", e);
            }
        }
        error!("{}", e);
    } else {
        debug!("heartbeat report success");
    }
}
