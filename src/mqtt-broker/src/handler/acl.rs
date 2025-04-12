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

use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{error, info};

use crate::security::AuthDriver;

pub struct UpdateAclCache {
    stop_send: broadcast::Sender<bool>,
    auth_driver: Arc<AuthDriver>,
}

impl UpdateAclCache {
    pub fn new(stop_send: broadcast::Sender<bool>, auth_driver: Arc<AuthDriver>) -> Self {
        UpdateAclCache {
            stop_send,
            auth_driver,
        }
    }

    pub async fn start_update(&self) {
        loop {
            let mut stop_rx = self.stop_send.subscribe();
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            info!("{}","Acl cache updating thread stopped successfully.");
                            break;
                        }
                    }
                }
                _ = self.update_acl_cache()=>{
                }
            }
        }
    }

    async fn update_acl_cache(&self) {
        match self.auth_driver.update_acl_cache().await {
            Ok(_) => {}
            Err(e) => {
                println!("Updating acl info normal exception");
                error!("{}", e);
            }
        };
        sleep(Duration::from_secs(5)).await;
    }
}
