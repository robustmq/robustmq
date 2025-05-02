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

// Stats
// connections
pub const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/connections/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/connections/max";
// subscribe
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/suboptions/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/suboptions/max";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscribers/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/max";
// topics
pub const SYSTEM_TOPIC_BROKERS_STATS_TOPICS_COUNT: &str = "$SYS/brokers/${node}/stats/topics/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_TOPICS_MAX: &str = "$SYS/brokers/${node}/stats/topics/max";
// routes
pub const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT: &str = "$SYS/brokers/${node}/stats/routes/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX: &str = "$SYS/brokers/${node}/stats/routes/max";

/*
pub(crate) async fn report_broker_stat_connections_count<S>(
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
        SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT,
        || async { metadata_cache.get_connection_count().to_string() },
    )
    .await;
}

pub(crate) async fn report_broker_stat_connections_max<S>(
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
        SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX,
        ||  async {
            metadata_cache.get_connection_max().to_string()
        },
    ).await;
}


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
        ||  async {
            metadata_cache.get_suboptions_count().to_string()
        },
    ).await;
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
        ||  async {
            metadata_cache.get_suboptions_max().to_string()
        },
    ).await;
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
        ||  async {
            metadata_cache.get_subscribers_count().to_string()
        },
    ).await;
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
        ||  async {
            metadata_cache.get_subscriptions_count().to_string()
        },
    ).await;
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
        ||  async {
            metadata_cache.get_shared_subscriptions_count().to_string()
        },
    ).await;
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
        ||  async {
            metadata_cache.get_shared_subscriptions_max().to_string()
        },
    ).await;
}


pub(crate) async fn report_broker_stat_topics_count<S>(
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
        SYSTEM_TOPIC_BROKERS_STATS_TOPICS_COUNT,
        ||  async {
            metadata_cache.get_topics_count().to_string()
        },
    ).await;
}

pub(crate) async fn report_broker_stat_topics_max<S>(
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
        SYSTEM_TOPIC_BROKERS_STATS_TOPICS_MAX,
        ||  async {
            metadata_cache.get_topics_max().to_string()
        },
    ).await;
}

pub(crate) async fn report_broker_stat_routes_count<S>(
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
        SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT,
        ||  async {
            metadata_cache.get_routes_count().to_string()
        },
    ).await;
}

pub(crate) async fn report_broker_stat_routes_max<S>(
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
        SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX,
        ||  async {
            metadata_cache.get_routes_max().to_string()
        },
    ).await;
}*/
