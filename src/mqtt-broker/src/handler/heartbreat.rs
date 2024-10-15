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

use grpc_clients::poll::ClientPool;
use log::{debug, error};
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

use crate::storage::cluster::ClusterStorage;

pub async fn report_heartbeat(client_poll: Arc<ClientPool>, stop_send: broadcast::Sender<bool>) {
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
            _ = report(client_poll.clone()) => {

            }
        }
    }
}

async fn report(client_poll: Arc<ClientPool>) {
    let cluster_storage = ClusterStorage::new(client_poll);
    match cluster_storage.heartbeat().await {
        Ok(()) => {}
        Err(e) => {
            error!("{}", e.to_string());
        }
    }
    sleep(Duration::from_secs(3)).await;
}
