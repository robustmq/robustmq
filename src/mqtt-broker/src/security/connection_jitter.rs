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

use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::tools::{convert_seconds, now_second};
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use metadata_struct::mqtt::cluster::MqttClusterDynamicConnectionJitter;
use std::sync::Arc;

use crate::handler::cache::CacheManager;

#[derive(Clone)]
pub struct ConnectionJitterCondition {
    pub client_id: String,
    connect_times: u32,
    first_request_time: u64,
}

pub fn check_connection_jitter(client_id: &str, cache_manager: &Arc<CacheManager>) {
    // todo ig prometheus to get metric
    let mut counter = 0;
    counter += 1;

    let mut connection_jitter_condition = if let Some(connection_jitter_condition) = cache_manager
        .acl_metadata
        .get_connection_jitter_condition(client_id)
    {
        connection_jitter_condition
    } else {
        ConnectionJitterCondition {
            client_id: client_id.to_string(),
            connect_times: counter,
            first_request_time: now_second(),
        }
    };

    let config = cache_manager.get_connection_jitter_config();
    let current_request_time = now_second();

    if is_within_window_time(
        current_request_time,
        connection_jitter_condition.first_request_time,
        config.window_time,
    ) {
        if is_exceed_max_client_connections(
            counter,
            connection_jitter_condition.connect_times,
            config.max_client_connections,
        ) {
            add_blacklist_4_connection_jitter(cache_manager, config);
        } else {
            connection_jitter_condition.connect_times = counter;
        }
    }

    connection_jitter_condition.connect_times = counter;
    connection_jitter_condition.first_request_time = now_second();

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
        end_time: now_second() + convert_seconds(config.ban_time, TimeUnit::Minutes) as u64,
        desc: "Ban due to connection jitter ".to_string(),
    };

    cache_manager.add_blacklist(client_id_blacklist);
}

fn is_within_window_time(
    current_request_time: u64,
    first_request_time: u64,
    window_time: u32,
) -> bool {
    let window_time_seconds = convert_seconds(window_time, TimeUnit::Minutes);

    current_request_time - first_request_time < window_time_seconds as u64
}

fn is_exceed_max_client_connections(
    current_time: u32,
    connect_times: u32,
    max_client_connections: u32,
) -> bool {
    current_time - connect_times > max_client_connections
}
