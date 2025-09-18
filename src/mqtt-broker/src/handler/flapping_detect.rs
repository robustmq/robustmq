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

use crate::handler::cache::MQTTCacheManager;
use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::error::ResultCommonError;
use common_base::tools::{convert_seconds, loop_select, now_second};
use common_config::config::MqttFlappingDetect;
use common_metrics::mqtt::event_metrics;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct FlappingDetectCondition {
    pub client_id: String,
    pub before_last_window_connections: u64,
    pub first_request_time: u64,
}

pub async fn clean_flapping_detect(
    cache_manager: Arc<MQTTCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        let config = cache_manager.get_flapping_detect_config().clone();
        cache_manager
            .acl_metadata
            .remove_flapping_detect_conditions(config)
            .await;
        Ok(())
    };

    loop_select(ac_fn, 10, &stop_send).await;
}

pub fn check_flapping_detect(client_id: String, cache_manager: &Arc<MQTTCacheManager>) {
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
    cache_manager: &Arc<MQTTCacheManager>,
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
