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
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// subscribe
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT: &str =
    "$SYS/brokers/stats/suboptions/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX: &str =
    "$SYS/brokers/stats/suboptions/max";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT: &str =
    "$SYS/brokers/stats/subscribers/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_MAX: &str =
    "$SYS/brokers/stats/subscribers/max";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT: &str =
    "$SYS/brokers/stats/subscriptions/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_MAX: &str =
    "$SYS/brokers/stats/subscriptions/max";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT: &str =
    "$SYS/brokers/stats/subscriptions/shared/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX: &str =
    "$SYS/brokers/stats/subscriptions/shared/max";

pub(crate) async fn report_broker_stat_sub_options(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    // Total subscriptions = exclusive + shared
    let exclusive = record_mqtt_subscriptions_exclusive_get();
    let shared = record_mqtt_subscriptions_shared_get();
    let total_subs = exclusive + shared;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT,
        || async move { total_subs.to_string() },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX,
        || async move { total_subs.to_string() },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT,
        || async move { exclusive.to_string() },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_MAX,
        || async move { exclusive.to_string() },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT,
        || async move { total_subs.to_string() },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_MAX,
        || async move { total_subs.to_string() },
    )
    .await;

    let shared_count = record_mqtt_subscriptions_shared_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT,
        || async move { shared_count.to_string() },
    )
    .await;

    let shared_group = record_mqtt_subscriptions_shared_group_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX,
        || async move { shared_group.to_string() },
    )
    .await;
}
