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

use metadata_struct::mqtt::connection::MQTTConnection;

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
        .mqtt_runtime
        .limit
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
        .mqtt_runtime
        .limit
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
        .mqtt_runtime
        .limit
        .cluster
        .max_topics as usize;
    if count > limit_count {
        return true;
    }

    // tenant
    if let Some(ten) = cache_manager.node_cache.get_tenant(tenant) {
        let count = cache_manager.node_cache.topic_count_by_tenant(tenant);
        if count > ten.config.max_topics as usize {
            return true;
        }
    }

    false
}

pub fn qos_flight_message_num_limit(
    cache_manager: &Arc<MQTTCacheManager>,
    connection: &MQTTConnection,
) -> bool {
    let len = cache_manager
        .pkid_manager
        .get_qos_pkid_data_len_by_client_id(&connection.client_id);
    len > connection.client_max_receive_maximum as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        pkid_manager::{PkidAckEnum, ReceiveQosPkidData},
        tool::test_build_mqtt_cache_manager,
    };
    use common_config::{broker::default_broker_config, storage::StorageType};
    use metadata_struct::{
        mqtt::{connection::MQTTConnection, session::MqttSession},
        tenant::{Tenant, TenantConfig},
        topic::Topic,
    };

    fn connection(client_id: &str, tenant: &str, max_receive: u16) -> MQTTConnection {
        MQTTConnection {
            client_id: client_id.to_string(),
            tenant: tenant.to_string(),
            client_max_receive_maximum: max_receive,
            ..Default::default()
        }
    }

    fn tenant(name: &str, max_connections: u64, max_sessions: u64, max_topics: u64) -> Tenant {
        Tenant {
            tenant_name: name.to_string(),
            config: TenantConfig {
                max_connections_per_node: max_connections,
                max_sessions,
                max_topics,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn session(tenant: &str, client_id: &str) -> MqttSession {
        MqttSession::new(
            tenant.to_string(),
            client_id.to_string(),
            60,
            false,
            None,
            true,
        )
    }

    #[tokio::test]
    async fn connection_limit_checks_node_and_tenant_quota() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let mut config = default_broker_config();
        config.mqtt_runtime.limit.cluster.max_connections_per_node = 1;
        cache_manager.node_cache.set_cluster_config(config);

        cache_manager.add_connection(1, connection("client-1", "tenant-a", 1));
        assert!(!connection_total_num_limit(&cache_manager, "tenant-a").await);

        cache_manager.add_connection(2, connection("client-2", "tenant-a", 1));
        assert!(connection_total_num_limit(&cache_manager, "tenant-a").await);

        let cache_manager = test_build_mqtt_cache_manager().await;
        let mut config = default_broker_config();
        config.mqtt_runtime.limit.cluster.max_connections_per_node = 10;
        cache_manager.node_cache.set_cluster_config(config);
        cache_manager
            .node_cache
            .add_tenant(tenant("tenant-a", 1, 10, 10));
        cache_manager.add_connection(1, connection("client-1", "tenant-a", 1));
        assert!(!connection_total_num_limit(&cache_manager, "tenant-a").await);
        cache_manager.add_connection(2, connection("client-2", "tenant-a", 1));
        assert!(connection_total_num_limit(&cache_manager, "tenant-a").await);
        assert!(!connection_total_num_limit(&cache_manager, "tenant-b").await);
    }

    #[tokio::test]
    async fn session_limit_checks_node_and_tenant_quota() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let mut config = default_broker_config();
        config.mqtt_runtime.limit.cluster.max_sessions = 1;
        cache_manager.node_cache.set_cluster_config(config);

        cache_manager.add_session("client-1", &session("tenant-a", "client-1"));
        assert!(!session_total_num_limit(&cache_manager, "tenant-a").await);
        cache_manager.add_session("client-2", &session("tenant-a", "client-2"));
        assert!(session_total_num_limit(&cache_manager, "tenant-a").await);

        let cache_manager = test_build_mqtt_cache_manager().await;
        let mut config = default_broker_config();
        config.mqtt_runtime.limit.cluster.max_sessions = 10;
        cache_manager.node_cache.set_cluster_config(config);
        cache_manager
            .node_cache
            .add_tenant(tenant("tenant-a", 10, 1, 10));
        cache_manager.add_session("client-1", &session("tenant-a", "client-1"));
        assert!(!session_total_num_limit(&cache_manager, "tenant-a").await);
        cache_manager.add_session("client-2", &session("tenant-a", "client-2"));
        assert!(session_total_num_limit(&cache_manager, "tenant-a").await);
        assert!(!session_total_num_limit(&cache_manager, "tenant-b").await);
    }

    #[tokio::test]
    async fn topic_limit_checks_node_and_tenant_quota() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let mut config = default_broker_config();
        config.mqtt_runtime.limit.cluster.max_topics = 1;
        cache_manager.node_cache.set_cluster_config(config);
        cache_manager.node_cache.add_topic(&Topic::new(
            "tenant-a",
            "topic-1",
            StorageType::EngineRocksDB,
        ));
        assert!(!topic_total_num_limit(&cache_manager, "tenant-a").await);
        cache_manager.node_cache.add_topic(&Topic::new(
            "tenant-a",
            "topic-2",
            StorageType::EngineRocksDB,
        ));
        assert!(topic_total_num_limit(&cache_manager, "tenant-a").await);

        let cache_manager = test_build_mqtt_cache_manager().await;
        let mut config = default_broker_config();
        config.mqtt_runtime.limit.cluster.max_topics = 10;
        cache_manager.node_cache.set_cluster_config(config);
        cache_manager
            .node_cache
            .add_tenant(tenant("tenant-a", 10, 10, 1));
        cache_manager.node_cache.add_topic(&Topic::new(
            "tenant-a",
            "topic-1",
            StorageType::EngineRocksDB,
        ));
        assert!(!topic_total_num_limit(&cache_manager, "tenant-a").await);
        cache_manager.node_cache.add_topic(&Topic::new(
            "tenant-a",
            "topic-2",
            StorageType::EngineRocksDB,
        ));
        assert!(topic_total_num_limit(&cache_manager, "tenant-a").await);
        assert!(!topic_total_num_limit(&cache_manager, "tenant-b").await);
    }

    #[test]
    fn qos_flight_limit_uses_strictly_greater_than_client_quota() {
        let cache_manager = futures::executor::block_on(test_build_mqtt_cache_manager());
        let connection = connection("client-1", "tenant-a", 2);
        assert!(!qos_flight_message_num_limit(&cache_manager, &connection));

        cache_manager.pkid_manager.add_qos_pkid_data(
            "client-1",
            ReceiveQosPkidData {
                ack_enum: PkidAckEnum::PubAck,
                pkid: 1,
                create_time: 0,
            },
        );
        cache_manager.pkid_manager.add_qos_pkid_data(
            "client-1",
            ReceiveQosPkidData {
                ack_enum: PkidAckEnum::PubRec,
                pkid: 2,
                create_time: 0,
            },
        );
        assert!(!qos_flight_message_num_limit(&cache_manager, &connection));

        cache_manager.pkid_manager.add_qos_pkid_data(
            "client-1",
            ReceiveQosPkidData {
                ack_enum: PkidAckEnum::PubComp,
                pkid: 3,
                create_time: 0,
            },
        );
        assert!(qos_flight_message_num_limit(&cache_manager, &connection));
    }
}
