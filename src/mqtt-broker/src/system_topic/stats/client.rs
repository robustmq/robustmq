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
use serde::Serialize;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

use crate::system_topic::SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS;

/// Connection/session statistics published as a single JSON payload to
/// `$SYS/brokers/stats/connections`.
#[derive(Debug, Serialize)]
pub(crate) struct BrokerConnectionsStats {
    // Currently active connections
    pub count: i64,
    // Peak connection count since broker start (approximated by session count,
    // as sessions persist beyond disconnection)
    pub max: i64,
    // Total persisted sessions (includes disconnected clients with clean_session=false)
    pub sessions: i64,
}

impl BrokerConnectionsStats {
    pub(crate) fn collect() -> Self {
        BrokerConnectionsStats {
            count: record_mqtt_connections_get(),
            max: record_mqtt_sessions_get(),
            sessions: record_mqtt_sessions_get(),
        }
    }
}

pub(crate) async fn report_broker_stat_connections(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    let stats = BrokerConnectionsStats::collect();
    let payload = serde_json::to_string(&stats).unwrap_or_default();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS,
        || async move { payload },
    )
    .await;
}
