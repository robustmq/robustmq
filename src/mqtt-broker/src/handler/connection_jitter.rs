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

use crate::handler::cache::CacheManager;
use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::tools::convert_seconds;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

pub struct UpdateConnectionJitterCache {
    stop_send: broadcast::Sender<bool>,
    cache_manager: Arc<CacheManager>,
}

impl UpdateConnectionJitterCache {
    pub fn new(stop_send: broadcast::Sender<bool>, cache_manager: Arc<CacheManager>) -> Self {
        Self {
            stop_send,
            cache_manager,
        }
    }

    pub async fn start_update(&self) {
        loop {
            let mut stop_rx = self.stop_send.subscribe();
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            info!("{}","Connection Jitter cache updating thread stopped successfully.");
                            break;
                        }
                    }
                }
                _ = self.update_connection_jitter_cache()=>{
                }
            }
        }
    }

    async fn update_connection_jitter_cache(&self) {
        let config = self.cache_manager.get_connection_jitter_config().clone();
        let window_time = config.window_time;
        let window_time_2_seconds = convert_seconds(window_time, TimeUnit::Minutes) as u64;
        match self
            .cache_manager
            .acl_metadata
            .remove_connection_jitter_conditions(config)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("Updating Connection Jitter cache norm exception");
                error!("{}", e);
            }
        }
        sleep(Duration::from_secs(window_time_2_seconds)).await;
    }
}
