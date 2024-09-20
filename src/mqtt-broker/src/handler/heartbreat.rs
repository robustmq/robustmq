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

use crate::storage::cluster::ClusterStorage;
use clients::poll::ClientPool;
use log::{debug, error};
use std::{sync::Arc, time::Duration};
use tokio::{select, sync::broadcast, time::sleep};

pub async fn report_heartbeat(client_poll: Arc<ClientPool>, stop_send: broadcast::Sender<bool>) {
    loop {
        let mut stop_recv = stop_send.subscribe();
        select! {
            val = stop_recv.recv() =>{
                match val{
                    Ok(flag) => {
                        if flag {
                            debug!("{}","Heartbeat reporting thread exited successfully");
                            break;
                        }
                    }
                    Err(_) => {}
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
    sleep(Duration::from_secs(5)).await;
}
