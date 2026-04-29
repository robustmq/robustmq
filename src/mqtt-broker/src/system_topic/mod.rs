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
use crate::core::error::MqttBrokerError;
use crate::core::topic::try_init_topic;
use crate::storage::message::MessageStorage;
use crate::system_topic::stats::route::report_broker_stat_routes;
use bytes::Bytes;
use common_base::error::ResultCommonError;
use common_base::tools::{get_local_ip, loop_select_ticket, now_millis};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::tenant::DEFAULT_TENANT;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::warn;

// Cluster status information
pub const SYSTEM_TOPIC_BROKERS: &str = "$SYS/brokers";
// Broker runtime status (version, uptime, datetime, sysdescr) as a single JSON payload
pub(crate) const SYSTEM_TOPIC_BROKERS_INFO: &str = "$SYS/brokers/info";

// Metrics topics
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_BYTES: &str = "$SYS/brokers/metrics/bytes";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES: &str = "$SYS/brokers/metrics/messages";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS: &str = "$SYS/brokers/metrics/packets";

// Stats topics
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS: &str = "$SYS/brokers/stats/connections";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_ROUTES: &str = "$SYS/brokers/stats/routes";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS: &str =
    "$SYS/brokers/stats/subscriptions";
pub(crate) const SYSTEM_TOPIC_BROKERS_STATS_TOPICS: &str = "$SYS/brokers/stats/topics";

pub mod broker;
pub mod packet;
pub mod stats;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemTopicEnvelope<T> {
    pub node_id: u64,
    pub node_ip: String,
    pub ts: u128,
    pub value: T,
}

pub struct SystemTopic {
    pub metadata_cache: Arc<MQTTCacheManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub client_pool: Arc<ClientPool>,
}

impl SystemTopic {
    pub fn new(
        metadata_cache: Arc<MQTTCacheManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        SystemTopic {
            metadata_cache,
            storage_driver_manager,
            client_pool,
        }
    }

    pub async fn start_thread(&self, stop_send: broadcast::Sender<bool>) {
        let ac_fn = async || -> ResultCommonError {
            report_broker_info(
                &self.client_pool,
                &self.metadata_cache,
                &self.storage_driver_manager,
            )
            .await;

            report_stats_info(
                &self.client_pool,
                &self.metadata_cache,
                &self.storage_driver_manager,
            )
            .await;

            report_packet_info(
                &self.client_pool,
                &self.metadata_cache,
                &self.storage_driver_manager,
            )
            .await;

            report_broker_stat_routes(
                &self.client_pool,
                &self.metadata_cache,
                &self.storage_driver_manager,
            )
            .await;

            Ok(())
        };

        let interval_ms = broker_config().mqtt_system_monitor.system_topic_interval_ms;
        loop_select_ticket(ac_fn, interval_ms, &stop_send).await;
    }
}

pub(crate) async fn report_broker_info(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    broker::report_cluster_status(client_pool, metadata_cache, storage_driver_manager).await;
    broker::report_broker_info_metrics(client_pool, metadata_cache, storage_driver_manager).await;
}

pub(crate) async fn report_packet_info(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    // bytes
    packet::bytes::report_broker_metrics_bytes(client_pool, metadata_cache, storage_driver_manager)
        .await;

    packet::packets::report_broker_metrics_packets(
        client_pool,
        metadata_cache,
        storage_driver_manager,
    )
    .await;
    // connect

    // messages
    packet::messages::report_broker_metrics_messages(
        client_pool,
        metadata_cache,
        storage_driver_manager,
    )
    .await;
}

pub(crate) async fn report_stats_info(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    // client
    stats::client::report_broker_stat_connections(
        client_pool,
        metadata_cache,
        storage_driver_manager,
    )
    .await;

    // subscription
    stats::subscription::report_broker_stat_sub_options(
        client_pool,
        metadata_cache,
        storage_driver_manager,
    )
    .await;

    //topics
    stats::topics::report_broker_stat_topics(client_pool, metadata_cache, storage_driver_manager)
        .await;
}

pub(crate) fn build_system_topic_payload<T: Serialize>(
    value: T,
) -> Result<String, serde_json::Error> {
    let payload = SystemTopicEnvelope {
        node_id: broker_config().broker_id,
        node_ip: get_local_ip(),
        ts: now_millis(),
        value,
    };
    serde_json::to_string(&payload)
}

pub(crate) async fn report_system_data<F, Fut, T>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    topic_const: &str,
    data_generator: F,
) where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
    T: Serialize,
{
    let topic_name = replace_topic_name(topic_const.to_string());
    let value = data_generator().await;
    let data = match build_system_topic_payload(value) {
        Ok(data) => data,
        Err(e) => {
            warn!(
                "Failed to serialize system topic payload for topic {}: {:?}",
                topic_name, e
            );
            return;
        }
    };

    if let Err(e) = write_topic_data(
        storage_driver_manager,
        metadata_cache,
        client_pool,
        topic_name.clone(),
        Bytes::from(data),
    )
    .await
    {
        warn!(
            "Failed to write system topic data to topic {}: {:?}",
            topic_name, e
        );
    }
}

pub(crate) fn replace_topic_name(mut topic_name: String) -> String {
    if topic_name.contains("${node}") {
        let local_ip = get_local_ip();
        topic_name = topic_name.replace("${node}", &local_ip)
    }
    topic_name
}

pub(crate) async fn write_topic_data(
    storage_driver_manager: &Arc<StorageDriverManager>,
    metadata_cache: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: String,
    payload: Bytes,
) -> Result<Vec<u64>, MqttBrokerError> {
    let topic = try_init_topic(
        DEFAULT_TENANT,
        &topic_name,
        &metadata_cache.clone(),
        storage_driver_manager,
        &client_pool.clone(),
    )
    .await?;

    let record = AdapterWriteRecord::new(topic_name, payload);
    let message_storage = MessageStorage::new(storage_driver_manager.clone());
    let resp = message_storage
        .append_topic_message(DEFAULT_TENANT, &topic.topic_name, vec![record])
        .await?;
    Ok(resp)
}

#[cfg(test)]
mod test {
    use crate::core::tool::test_build_mqtt_cache_manager0;
    use crate::system_topic::write_topic_data;
    use bytes::Bytes;
    use common_base::{tools::get_local_ip, uuid::unique_id};
    use common_config::broker::{broker_config, default_broker_config, init_broker_conf_by_config};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use std::collections::HashMap;
    use std::sync::Arc;
    use storage_adapter::storage::{test_add_topic, test_build_storage_driver_manager};

    #[tokio::test]
    async fn test_write_topic_data() {
        init_broker_conf_by_config(default_broker_config());
        let client_pool = Arc::new(ClientPool::new(3));
        let topic_name = format!("$SYS/brokers/{}-test", unique_id());
        let storage_driver_manager = test_build_storage_driver_manager().await.unwrap();
        let cache_manger =
            test_build_mqtt_cache_manager0(storage_driver_manager.broker_cache.clone()).await;

        test_add_topic(&storage_driver_manager, &topic_name);

        let data = Bytes::from("test_write_topic_data");
        let resp = write_topic_data(
            &storage_driver_manager,
            &cache_manger,
            &client_pool,
            topic_name.clone(),
            data.clone(),
        )
        .await
        .unwrap();

        println!("{:?}", resp);

        let topic = storage_driver_manager
            .broker_cache
            .get_topic_by_name(DEFAULT_TENANT, &topic_name)
            .unwrap();

        let read_config = AdapterReadConfig {
            max_record_num: 1,
            max_size: 1024 * 1024 * 1024,
        };

        let results = storage_driver_manager
            .read_by_offset(
                DEFAULT_TENANT,
                &topic.topic_name,
                &HashMap::new(),
                &read_config,
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(data, results[0].data)
    }

    #[tokio::test]
    async fn test_replace_topic_name() {
        let topic_name = "$SYS/brokers/${node}/version".to_string();
        let replaced_name = super::replace_topic_name(topic_name.clone());
        let local_ip = get_local_ip();
        let expected_name = format!("$SYS/brokers/{local_ip}/version");
        assert!(replaced_name.contains("brokers/"));
        assert!(!replaced_name.contains("${node}"));
        assert_eq!(expected_name, replaced_name)
    }

    #[tokio::test]
    async fn test_report_system_data() {
        init_broker_conf_by_config(default_broker_config());
        let client_pool = Arc::new(ClientPool::new(3));
        let topic_name = format!("$SYS/brokers/{}-test", unique_id());

        let storage_driver_manager = test_build_storage_driver_manager().await.unwrap();
        let cache_manger =
            test_build_mqtt_cache_manager0(storage_driver_manager.broker_cache.clone()).await;

        test_add_topic(&storage_driver_manager, &topic_name);

        let expect_value = "test_data".to_string();
        super::report_system_data(
            &client_pool,
            &cache_manger,
            &storage_driver_manager,
            &topic_name,
            || async { expect_value.clone() },
        )
        .await;

        let mqtt_topic = cache_manger
            .node_cache
            .get_topic_by_name(DEFAULT_TENANT, &topic_name)
            .unwrap();

        let read_config = AdapterReadConfig {
            max_record_num: 1,
            max_size: 1024 * 1024 * 1024,
        };
        let results = storage_driver_manager
            .read_by_offset(
                DEFAULT_TENANT,
                &mqtt_topic.topic_name,
                &HashMap::new(),
                &read_config,
            )
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        let payload = results[0].data.clone();
        let payload: super::SystemTopicEnvelope<String> =
            serde_json::from_str(&String::from_utf8_lossy(&payload)).unwrap();
        assert_eq!(payload.value, "test_data");
        assert_eq!(payload.node_id, broker_config().broker_id);
    }
}
