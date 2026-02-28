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
use common_metrics::mqtt::statistics::{
    record_mqtt_subscriptions_exclusive_get, record_mqtt_subscriptions_shared_get,
};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// routes
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT: &str = "$SYS/brokers/stats/routes/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX: &str = "$SYS/brokers/stats/routes/max";

pub(crate) async fn report_broker_stat_routes(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    // In MQTT, routes ≈ total active subscriptions (each subscription creates a routing entry)
    let routes = record_mqtt_subscriptions_exclusive_get() + record_mqtt_subscriptions_shared_get();

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT,
        || async move { routes.to_string() },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX,
        || async move { routes.to_string() },
    )
    .await;
}
