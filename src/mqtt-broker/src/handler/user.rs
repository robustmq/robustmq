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
use log::{error, info};
use metadata_struct::mqtt::user::MqttUser;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

use crate::security::AuthDriver;
use crate::storage::user::UserStorage;

use super::cache::CacheManager;

pub struct UpdateUserCache {
    stop_send: broadcast::Sender<bool>,
    auth_driver: Arc<AuthDriver>,
}

impl UpdateUserCache {
    pub fn new(stop_send: broadcast::Sender<bool>, auth_driver: Arc<AuthDriver>) -> Self {
        UpdateUserCache {
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
                            info!("{}","User cache updating thread stopped successfully.");
                            break;
                        }
                    }
                }
                _ = self.update_user_cache()=>{
                }
            }
        }
    }

    async fn update_user_cache(&self) {
        match self.auth_driver.update_user_cache().await {
            Ok(_) => {}
            Err(e) => {
                println!("Updating user info normal exception");
                error!("{}", e);
            }
        };
        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn init_system_user(cache_manager: &Arc<CacheManager>, client_pool: &Arc<ClientPool>) {
    let conf = broker_mqtt_conf();
    let system_user_info = MqttUser {
        username: conf.system.default_user.clone(),
        password: conf.system.default_password.clone(),
        is_superuser: true,
    };
    let user_storage = UserStorage::new(client_pool.clone());
    match user_storage.save_user(system_user_info.clone()).await {
        Ok(_) => {
            cache_manager.add_user(system_user_info);
        }
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }
}
