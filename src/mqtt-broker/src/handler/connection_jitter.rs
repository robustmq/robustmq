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
use crate::handler::error::MqttBrokerError;
use crate::observability::metrics::event_metrics;
use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::tools::{convert_seconds, now_second};
use log::{error, info};
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use metadata_struct::mqtt::cluster::MqttClusterDynamicConnectionJitter;
use protocol::broker_mqtt::broker_mqtt_admin::EnableConnectionJitterRequest;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

pub struct UpdateConnectionJitterCache {
    stop_send: broadcast::Sender<bool>,
    cache_manager: Arc<CacheManager>,
}

#[derive(Clone, Debug)]
pub struct ConnectionJitterCondition {
    pub client_id: String,
    pub connect_times: u64,
    pub first_request_time: u64,
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
        let window_time = config.window_time as u64;
        let window_time_2_seconds = convert_seconds(window_time, TimeUnit::Minutes);
        match self
            .cache_manager
            .acl_metadata
            .remove_connection_jitter_conditions(config)
            .await
        {
            Ok(_) => {
                info!("Updating Connection Jitter cache norm exception");
            }
            Err(e) => {
                error!("{}", e);
            }
        }
        sleep(Duration::from_secs(window_time_2_seconds)).await;
    }
}

pub fn check_connection_jitter(client_id: String, cache_manager: &Arc<CacheManager>) {
    // get metric
    let current_counter = event_metrics::get_client_connection_counter(client_id.clone());
    let current_request_time = now_second();

    // get connection jitter info
    let connection_jitter_condition = if let Some(connection_jitter_condition) = cache_manager
        .acl_metadata
        .get_connection_jitter_condition(client_id.clone())
    {
        connection_jitter_condition
    } else {
        ConnectionJitterCondition {
            client_id: client_id.clone(),
            connect_times: current_counter,
            first_request_time: current_request_time,
        }
    };

    // incr metric
    event_metrics::incr_client_connection_counter(client_id.clone());

    let config = cache_manager.get_connection_jitter_config();
    let current_counter = event_metrics::get_client_connection_counter(client_id.clone());

    if is_within_window_time(
        current_request_time,
        connection_jitter_condition.first_request_time,
        config.window_time as u64,
    ) {
        if is_exceed_max_client_connections(
            current_counter,
            connection_jitter_condition.connect_times,
            config.max_client_connections,
        ) {
            add_blacklist_4_connection_jitter(cache_manager, config);
        }
    }

    cache_manager
        .acl_metadata
        .add_connection_jitter_condition(connection_jitter_condition);
}

fn add_blacklist_4_connection_jitter(
    cache_manager: &Arc<CacheManager>,
    config: MqttClusterDynamicConnectionJitter,
) {
    let client_id_blacklist = MqttAclBlackList {
        blacklist_type: MqttAclBlackListType::ClientId,
        resource_name: "client_id".to_string(),
        end_time: now_second() + convert_seconds(config.ban_time as u64, TimeUnit::Minutes) as u64,
        desc: "Ban due to connection jitter ".to_string(),
    };

    cache_manager.add_blacklist(client_id_blacklist);
}

fn is_within_window_time(
    current_request_time: u64,
    first_request_time: u64,
    window_time: u64,
) -> bool {
    let window_time_seconds = convert_seconds(window_time, TimeUnit::Minutes);

    current_request_time - first_request_time < window_time_seconds as u64
}

fn is_exceed_max_client_connections(
    current_time: u64,
    connect_times: u64,
    max_client_connections: u64,
) -> bool {
    current_time - connect_times > max_client_connections
}

pub async fn enable_connection_jitter(
    cache_manager: &Arc<CacheManager>,
    request: EnableConnectionJitterRequest,
) -> Result<(), MqttBrokerError> {
    let connection_jitter = MqttClusterDynamicConnectionJitter {
        enable: request.is_enable,
        window_time: request.window_time,
        max_client_connections: request.max_client_connections as u64,
        ban_time: request.ban_time,
    };

    cache_manager
        .set_connection_jitter_config(connection_jitter)
        .await?;

    Ok(())
}
