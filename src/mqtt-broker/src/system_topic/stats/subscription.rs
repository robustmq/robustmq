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
    record_mqtt_subscriptions_shared_group_get,
};
use grpc_clients::pool::ClientPool;
use serde::Serialize;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

use crate::system_topic::SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS;

/// Subscription statistics published as a single JSON payload to
/// `$SYS/brokers/stats/subscriptions`.
#[derive(Debug, Serialize)]
pub(crate) struct BrokerSubscriptionsStats {
    // Total active subscriptions (exclusive + shared)
    pub count: i64,
    // Peak total subscriptions since broker start
    pub max: i64,
    // Exclusive (non-shared) subscriptions only
    pub exclusive_count: i64,
    // Shared subscriptions only
    pub shared_count: i64,
    // Number of distinct shared subscription groups
    pub shared_group_count: i64,
}

impl BrokerSubscriptionsStats {
    pub(crate) fn collect() -> Self {
        let exclusive = record_mqtt_subscriptions_exclusive_get();
        let shared = record_mqtt_subscriptions_shared_get();
        let total = exclusive + shared;
        let shared_group = record_mqtt_subscriptions_shared_group_get();
        BrokerSubscriptionsStats {
            count: total,
            max: total,
            exclusive_count: exclusive,
            shared_count: shared,
            shared_group_count: shared_group,
        }
    }
}

pub(crate) async fn report_broker_stat_sub_options(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    let stats = BrokerSubscriptionsStats::collect();
    let payload = serde_json::to_string(&stats).unwrap_or_default();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS,
        || async move { payload },
    )
    .await;
}
