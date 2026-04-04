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

use crate::core::cache::MQTTCacheManager;
use crate::core::tool::ResultMqttBrokerError;
use crate::storage::local::LocalStorage;
use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::error::ResultCommonError;
use common_base::tools::{convert_seconds, loop_select_ticket, now_second};
use common_config::config::MqttFlappingDetect;
use common_metrics::mqtt::event;
use common_security::manager::SecurityManager;
use metadata_struct::auth::blacklist::{EnumBlackListType, SecurityBlackList};
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BanLog {
    pub tenant: String,
    pub ban_type: String,
    pub resource_name: String,
    pub ban_source: String,
    pub end_time: u64,
    pub create_time: u64,
}

#[derive(Clone, Debug)]
pub struct FlappingDetectCondition {
    pub tenant: String,
    pub client_id: String,
    pub before_last_window_connections: u64,
    pub first_request_time: u64,
}

pub async fn clean_flapping_detect(
    cache_manager: Arc<MQTTCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        let config = cache_manager
            .node_cache
            .get_cluster_config()
            .mqtt_flapping_detect;
        cache_manager
            .remove_flapping_detect_conditions(config)
            .await;
        Ok(())
    };

    loop_select_ticket(ac_fn, 10000, &stop_send).await;
}

pub async fn check_flapping_detect(
    tenant: &str,
    client_id: String,
    cache_manager: &Arc<MQTTCacheManager>,
    security_manager: &Arc<SecurityManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> ResultMqttBrokerError {
    // get metric
    let current_counter = event::get_client_connection_counter(tenant, client_id.clone());
    let current_request_time = now_second();

    // get flapping detect info
    let flapping_detect_condition = if let Some(flapping_detect_info) =
        cache_manager.get_flapping_detect_condition(tenant, &client_id)
    {
        flapping_detect_info
    } else {
        FlappingDetectCondition {
            tenant: tenant.to_string(),
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
    event::incr_client_connection_counter(tenant, client_id.clone());

    let config = cache_manager
        .node_cache
        .get_cluster_config()
        .mqtt_flapping_detect;
    let current_counter = event::get_client_connection_counter(tenant, client_id.clone());
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
        add_blacklist_4_connection_jitter(
            tenant,
            security_manager,
            rocksdb_engine_handler,
            config,
            client_id,
        )
        .await?;
    }

    cache_manager.add_flapping_detect_condition(flapping_detect_condition);
    Ok(())
}

async fn add_blacklist_4_connection_jitter(
    tenant: &str,
    security_manager: &Arc<SecurityManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    config: MqttFlappingDetect,
    client_id: String,
) -> ResultMqttBrokerError {
    let end_time = now_second() + convert_seconds(config.ban_time as u64, TimeUnit::Minutes);
    let client_id_blacklist = SecurityBlackList {
        name: format!("flapping-ban-{}-{}", tenant, client_id),
        tenant: tenant.to_string(),
        blacklist_type: EnumBlackListType::ClientId,
        resource_name: client_id.clone(),
        end_time,
        desc: "Ban due to connection jitter ".to_string(),
    };

    security_manager.metadata.add_blacklist(client_id_blacklist);

    let local_storage = LocalStorage::new(rocksdb_engine_handler.clone());
    let log = BanLog {
        tenant: tenant.to_string(),
        ban_source: "flapping_detect".to_string(),
        ban_type: "client_id".to_string(),
        resource_name: client_id.clone(),
        end_time,
        create_time: now_second(),
    };
    local_storage.save_ban_log(log).await?;
    Ok(())
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
