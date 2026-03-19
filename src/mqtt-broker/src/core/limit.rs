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
use std::sync::Arc;

pub async fn connection_total_num_limit(
    cache_manager: &Arc<MQTTCacheManager>,
    tenant: &str,
) -> bool {
    // node
    let count = cache_manager.get_connection_count();
    let limit_count = cache_manager
        .node_cache
        .get_cluster_config()
        .await
        .mqtt_limit
        .cluster
        .max_connections_per_node as usize;
    if count > limit_count {
        return true;
    }

    // tenant
    if let Some(ten) = cache_manager.node_cache.get_tenant(tenant) {
        let count = cache_manager.get_connection_count_by_tenant(tenant);
        if count > ten.config.max_connections_per_node as usize {
            return true;
        }
    }

    false
}

pub async fn session_total_num_limit(cache_manager: &Arc<MQTTCacheManager>, tenant: &str) -> bool {
    // cluster
    let count = cache_manager.session_count();
    let limit_count = cache_manager
        .node_cache
        .get_cluster_config()
        .await
        .mqtt_limit
        .cluster
        .max_sessions as usize;
    if count > limit_count {
        return true;
    }

    // tenant
    if let Some(ten) = cache_manager.node_cache.get_tenant(tenant) {
        let count = cache_manager.session_count_by_tenant(tenant);
        if count > ten.config.max_sessions as usize {
            return true;
        }
    }

    false
}

pub async fn topic_total_num_limit(cache_manager: &Arc<MQTTCacheManager>, tenant: &str) -> bool {
    // cluster
    let count = cache_manager.node_cache.topic_count();
    let limit_count = cache_manager
        .node_cache
        .get_cluster_config()
        .await
        .mqtt_limit
        .cluster
        .max_topics as usize;
    if count > limit_count {
        return true;
    }

    // tenant
    if let Some(ten) = cache_manager.node_cache.get_tenant(tenant) {
        let count = cache_manager.session_count_by_tenant(tenant);
        if count > ten.config.max_topics as usize {
            return true;
        }
    }

    false
}

pub async fn qos2_flight_num_limit(cache_manager: &Arc<MQTTCacheManager>, tenant: &str) -> bool {
    // cluster
    let count = cache_manager.node_cache.topic_count();
    let limit_count = cache_manager
        .node_cache
        .get_cluster_config()
        .await
        .mqtt_limit
        .cluster
        .max_mqtt_qos2_num as usize;
    if count > limit_count {
        return true;
    }

    // tenant
    if let Some(ten) = cache_manager.node_cache.get_tenant(tenant) {
        let count = cache_manager.session_count_by_tenant(tenant);
        if count > ten.config.max_mqtt_qos2_num as usize {
            return true;
        }
    }

    false
}
