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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::handler::dynamic_config::{save_cluster_dynamic_config, ClusterDynamicConfig};
use crate::observability::metrics::event_metrics;
use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::tools::{convert_seconds, now_second};
use common_config::broker::config::MqttFlappingDetect;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use protocol::broker_mqtt::broker_mqtt_admin::EnableFlappingDetectRequest;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error, info};

pub struct UpdateFlappingDetectCache {
    stop_send: broadcast::Sender<bool>,
    cache_manager: Arc<CacheManager>,
}

#[derive(Clone, Debug)]
pub struct FlappingDetectCondition {
    pub client_id: String,
    pub before_last_window_connections: u64,
    pub first_request_time: u64,
}

impl UpdateFlappingDetectCache {
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
                            info!("{}","Flapping detect cache updating thread stopped successfully.");
                            break;
                        }
                    }
                }
                _ = self.update_flapping_detect_cache()=>{
                }
            }
        }
    }

    async fn update_flapping_detect_cache(&self) {
        let config = self.cache_manager.get_flapping_detect_config().clone();
        let window_time = config.window_time as u64;
        match self
            .cache_manager
            .acl_metadata
            .remove_flapping_detect_conditions(config)
            .await
        {
            Ok(_) => {
                debug!("Updating Flapping detect cache norm exception");
            }
            Err(e) => {
                error!("{}", e);
            }
        }
        sleep(Duration::from_secs(convert_seconds(
            window_time,
            TimeUnit::Minutes,
        )))
        .await;
    }
}

pub fn check_flapping_detect(client_id: String, cache_manager: &Arc<CacheManager>) {
    // get metric
    let current_counter = event_metrics::get_client_connection_counter(client_id.clone());
    let current_request_time = now_second();

    // get flapping detect info
    let flapping_detect_condition = if let Some(flapping_detect_info) = cache_manager
        .acl_metadata
        .get_flapping_detect_condition(client_id.clone())
    {
        flapping_detect_info
    } else {
        FlappingDetectCondition {
            client_id: client_id.clone(),
            before_last_window_connections: current_counter + 1,
            first_request_time: current_request_time,
        }
    };

    debug!(
        "get a flapping_detect_condition: {:?}",
        flapping_detect_condition.clone()
    );

    // incr metric
    event_metrics::incr_client_connection_counter(client_id.clone());

    let config = cache_manager.get_flapping_detect_config();
    let current_counter = event_metrics::get_client_connection_counter(client_id.clone());
    debug!("get current_counter : {current_counter} by client_id: {client_id}");

    if is_within_window_time(
        current_request_time,
        flapping_detect_condition.first_request_time,
        config.window_time as u64,
    ) && is_exceed_max_client_connections(
        current_counter,
        flapping_detect_condition.before_last_window_connections,
        config.max_client_connections,
    ) {
        debug!("add a new client_id: {client_id} into blacklist.");
        add_blacklist_4_connection_jitter(cache_manager, config, client_id);
    }

    cache_manager
        .acl_metadata
        .add_flapping_detect_condition(flapping_detect_condition);
}

fn add_blacklist_4_connection_jitter(
    cache_manager: &Arc<CacheManager>,
    config: MqttFlappingDetect,
    client_id: String,
) {
    let client_id_blacklist = MqttAclBlackList {
        blacklist_type: MqttAclBlackListType::ClientId,
        resource_name: client_id,
        end_time: now_second() + convert_seconds(config.ban_time as u64, TimeUnit::Minutes),
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

    current_request_time - first_request_time < window_time_seconds
}

fn is_exceed_max_client_connections(
    current_time: u64,
    connect_times: u64,
    max_client_connections: u64,
) -> bool {
    current_time - connect_times >= max_client_connections
}

pub async fn enable_flapping_detect(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: EnableFlappingDetectRequest,
) -> ResultMqttBrokerError {
    let connection_jitter = MqttFlappingDetect {
        enable: request.is_enable,
        window_time: request.window_time,
        max_client_connections: request.max_client_connections as u64,
        ban_time: request.ban_time,
    };

    save_cluster_dynamic_config(
        client_pool,
        ClusterDynamicConfig::MqttFlappingDetect,
        connection_jitter.encode(),
    )
    .await?;

    cache_manager.update_flapping_detect_config(connection_jitter);

    Ok(())
}
