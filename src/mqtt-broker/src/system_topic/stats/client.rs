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
use crate::system_topic::report_system_data;
use common_metrics::mqtt::statistics::{record_mqtt_connections_get, record_mqtt_sessions_get};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// Stats
// connections
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT: &str =
    "$SYS/brokers/stats/connections/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX: &str =
    "$SYS/brokers/stats/connections/max";

pub(crate) async fn report_broker_stat_connections(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    let current = record_mqtt_connections_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT,
        || async move { current.to_string() },
    )
    .await;

    // max is approximated by the session count (sessions persist beyond disconnection)
    let max = record_mqtt_sessions_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX,
        || async move { max.to_string() },
    )
    .await;
}
