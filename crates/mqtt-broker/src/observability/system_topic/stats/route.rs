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
use crate::observability::system_topic::report_system_data;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;

// routes
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT: &str =
    "$SYS/brokers/${node}/stats/routes/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX: &str =
    "$SYS/brokers/${node}/stats/routes/max";

pub(crate) async fn report_broker_stat_routes(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT,
        || async {
            "".to_string()
            // metadata_cache.get_routes_count().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX,
        || async {
            "".to_string()
            // metadata_cache.get_routes_max().to_string()
        },
    )
    .await;
}
