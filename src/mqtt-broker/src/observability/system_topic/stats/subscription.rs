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

use crate::handler::cache::CacheManager;
use crate::observability::system_topic::report_system_data;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

// subscribe
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/suboptions/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/suboptions/max";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscribers/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_MAX: &str =
    "$SYS/brokers/${node}/stats/subscribers/max";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/count";

pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/subscriptions/max";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/count";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/max";

pub(crate) async fn report_broker_stat_suboptions_count<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT,
        || async {
            "".to_string()
            // metadata_cache.get_suboptions_count().to_string()
        },
    )
    .await;
}

pub(crate) async fn report_broker_stat_suboptions_max<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX,
        || async {
            "".to_string()
            // metadata_cache.get_suboptions_max().to_string()
        },
    )
    .await;
}

pub(crate) async fn report_broker_stat_subscribers_count<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT,
        || async {
            "".to_string()
            // metadata_cache.get_subscribers_count().to_string()
        },
    )
    .await;
}

pub(crate) async fn report_broker_stat_subscribers_max<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_MAX,
        || async {
            "".to_string()
            // metadata_cache.get_subscribers_count().to_string()
        },
    )
    .await;
}

pub(crate) async fn report_broker_stat_subscriptions_count<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT,
        || async {
            "".to_string()
            // metadata_cache.get_subscriptions_count().to_string()
        },
    )
    .await;
}
pub(crate) async fn report_broker_stat_subscriptions_max<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_MAX,
        || async {
            "".to_string()
            // metadata_cache.get_subscriptions_count().to_string()
        },
    )
    .await;
}

pub(crate) async fn report_broker_stat_subscriptions_shared_count<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT,
        || async {
            "".to_string()
            // metadata_cache.get_shared_subscriptions_count().to_string()
        },
    )
    .await;
}

pub(crate) async fn report_broker_stat_subscriptions_shared_max<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX,
        || async {
            "".to_string()
            // metadata_cache.get_shared_subscriptions_max().to_string()
        },
    )
    .await;
}
