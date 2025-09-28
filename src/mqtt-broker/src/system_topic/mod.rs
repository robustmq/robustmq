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
use crate::handler::topic::try_init_topic;
use crate::storage::message::MessageStorage;
use crate::system_topic::packet::bytes::{
    SYSTEM_TOPIC_BROKERS_METRICS_BYTES_RECEIVED, SYSTEM_TOPIC_BROKERS_METRICS_BYTES_SENT,
};
use crate::system_topic::packet::messages::{
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED, SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED, SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED,
    SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT,
};
use crate::system_topic::packet::packets::{
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH, SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ, SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED, SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK, SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK,
    SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE,
};
use crate::system_topic::stats::client::{
    SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX,
};
use crate::system_topic::stats::route::{
    report_broker_stat_routes, SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT,
    SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX,
};
use crate::system_topic::stats::subscription::{
    SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_MAX,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_MAX,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX,
};
use crate::system_topic::sysmon::SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE;
use common_base::error::ResultCommonError;
use common_base::tools::{get_local_ip, loop_select};
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::sync::broadcast;
use tracing::error;

// Cluster status information
pub const SYSTEM_TOPIC_BROKERS: &str = "$SYS/brokers";
pub const SYSTEM_TOPIC_BROKERS_VERSION: &str = "$SYS/brokers/${node}/version";
pub const SYSTEM_TOPIC_BROKERS_UPTIME: &str = "$SYS/brokers/${node}/uptime";
pub const SYSTEM_TOPIC_BROKERS_DATETIME: &str = "$SYS/brokers/${node}/datetime";
pub const SYSTEM_TOPIC_BROKERS_SYSDESCR: &str = "$SYS/brokers/${node}/sysdescr";

// Event
pub const SYSTEM_TOPIC_BROKERS_CONNECTED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/connected";
pub const SYSTEM_TOPIC_BROKERS_DISCONNECTED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/disconnected";
pub const SYSTEM_TOPIC_BROKERS_SUBSCRIBED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/subscribed";
pub const SYSTEM_TOPIC_BROKERS_UNSUBSCRIBED: &str =
    "$SYS/brokers/${node}/clients/${clientid}/unsubscribed";

// System alarm
pub const SYSTEM_TOPIC_BROKERS_ALARMS_ALERT: &str = "$SYS/brokers/${node}/alarms/alert";
pub const SYSTEM_TOPIC_BROKERS_ALARMS_CLEAR: &str = "$SYS/brokers/${node}/alarms/clear";
// system symon
pub const SYSTEM_TOPIC_BROKERS_SYSMON_LONG_GC: &str = "$SYS/brokers/${node}/sysmon/long_gc";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_LONG_SCHEDULE: &str =
    "$SYS/brokers/${node}/sysmon/long_schedule";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_LARGE_HEAP: &str = "$SYS/brokers/${node}/sysmon/large_heap";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_BUSY_PORT: &str = "$SYS/brokers/${node}/sysmon/busy_port";
pub const SYSTEM_TOPIC_BROKERS_SYSMON_BUSY_DIST_PORT: &str =
    "$SYS/brokers/${node}/sysmon/busy_dist_port";

pub mod broker;
pub mod event;
pub mod packet;
pub mod stats;
pub mod sysmon;

pub struct SystemTopic {
    pub metadata_cache: Arc<MQTTCacheManager>,
    pub message_storage_adapter: ArcStorageAdapter,
    pub client_pool: Arc<ClientPool>,
}

impl SystemTopic {
    pub fn new(
        metadata_cache: Arc<MQTTCacheManager>,
        message_storage_adapter: ArcStorageAdapter,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        SystemTopic {
            metadata_cache,
            message_storage_adapter,
            client_pool,
        }
    }

    pub async fn start_thread(&self, stop_send: broadcast::Sender<bool>) {
        self.try_init_system_topic().await;
        let ac_fn = async || -> ResultCommonError {
            report_broker_info(
                &self.client_pool,
                &self.metadata_cache,
                &self.message_storage_adapter,
            )
            .await;

            report_stats_info(
                &self.client_pool,
                &self.metadata_cache,
                &self.message_storage_adapter,
            )
            .await;

            report_packet_info(
                &self.client_pool,
                &self.metadata_cache,
                &self.message_storage_adapter,
            )
            .await;

            report_broker_stat_routes(
                &self.client_pool,
                &self.metadata_cache,
                &self.message_storage_adapter,
            )
            .await;

            Ok(())
        };

        loop_select(ac_fn, 1, &stop_send).await;
    }

    pub async fn try_init_system_topic(&self) {
        let results = self.get_all_system_topic();
        for topic_name in results {
            let new_topic_name = replace_topic_name(topic_name);
            if let Err(e) = try_init_topic(
                &new_topic_name,
                &self.metadata_cache,
                &self.message_storage_adapter,
                &self.client_pool,
            )
            .await
            {
                if e.to_string().contains("already exist") {
                    return;
                }
                panic!("Initializing system topic {new_topic_name} Failed, error message :{e}");
            }
        }
    }

    fn get_all_system_topic(&self) -> Vec<String> {
        vec![
            // broker
            SYSTEM_TOPIC_BROKERS.to_string(),
            SYSTEM_TOPIC_BROKERS_VERSION.to_string(),
            SYSTEM_TOPIC_BROKERS_UPTIME.to_string(),
            SYSTEM_TOPIC_BROKERS_DATETIME.to_string(),
            SYSTEM_TOPIC_BROKERS_SYSDESCR.to_string(),
            // stats
            SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_MAX.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_MAX.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT.to_string(),
            SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX.to_string(),
            // bytes
            SYSTEM_TOPIC_BROKERS_METRICS_BYTES_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_BYTES_SENT.to_string(),
            // messages
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED.to_string(),
            // packets
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT.to_string(),
            SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH.to_string(),
            // ALARM
            SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE.to_string(),
        ]
    }
}

pub(crate) async fn report_broker_info(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    broker::report_cluster_status(client_pool, metadata_cache, message_storage_adapter).await;
    broker::report_broker_version(client_pool, metadata_cache, message_storage_adapter).await;
    broker::report_broker_time(client_pool, metadata_cache, message_storage_adapter).await;
    broker::report_broker_sysdescr(client_pool, metadata_cache, message_storage_adapter).await;
}

pub(crate) async fn report_packet_info(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    // bytes
    packet::bytes::report_broker_metrics_bytes(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;

    packet::packets::report_broker_metrics_packets(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    // connect

    // messages
    packet::messages::report_broker_metrics_messages(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
}

pub(crate) async fn report_stats_info(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    // client
    stats::client::report_broker_stat_connections(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;

    // subscription
    stats::subscription::report_broker_stat_sub_options(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;

    //topics
    stats::topics::report_broker_stat_topics(client_pool, metadata_cache, message_storage_adapter)
        .await;
}

pub(crate) async fn report_system_data<F, Fut>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
    topic_const: &str,
    data_generator: F,
) where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = String>,
{
    let topic_name = replace_topic_name(topic_const.to_string());
    let data = data_generator().await;

    if let Some(record) = MqttMessage::build_system_topic_message(topic_name.clone(), data) {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_pool,
            topic_name,
            record,
        )
        .await;
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
    message_storage_adapter: &ArcStorageAdapter,
    metadata_cache: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: String,
    record: Record,
) {
    match try_init_topic(
        &topic_name,
        &metadata_cache.clone(),
        &message_storage_adapter.clone(),
        &client_pool.clone(),
    )
    .await
    {
        Ok(topic) => {
            let message_storage = MessageStorage::new(message_storage_adapter.clone());
            match message_storage
                .append_topic_message(&topic.topic_id, vec![record])
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "Message written to system subject {} Error, error message :{}",
                        topic_name,
                        e.to_string()
                    );
                }
            }
        }
        Err(e) => {
            if e.to_string().contains("already exist") {
                return;
            }

            error!(
                "Initializing system topic {} Failed, error message :{}",
                topic_name,
                e.to_string()
            );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::storage::message::cluster_name;
    use crate::system_topic::write_topic_data;
    use common_base::tools::{get_local_ip, unique_id};
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::adapter::read_config::ReadConfig;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use std::sync::Arc;
    use storage_adapter::storage::build_memory_storage_driver;

    #[tokio::test]
    async fn test_write_topic_data() {
        init_broker_conf_by_config(default_broker_config());
        let client_pool = Arc::new(ClientPool::new(3));
        let cache_manger = test_build_mqtt_cache_manager();
        let topic_name = format!("$SYS/brokers/{}-test", unique_id());
        let mqtt_topic = MQTTTopic::new(unique_id(), cluster_name(), topic_name.clone());
        cache_manger.add_topic(&topic_name, &mqtt_topic);

        let message_storage_adapter = build_memory_storage_driver();
        let data = "test_write_topic_data".to_string();

        let topic_message =
            MqttMessage::build_system_topic_message(topic_name.clone(), data).unwrap();

        write_topic_data(
            &message_storage_adapter,
            &cache_manger,
            &client_pool,
            topic_name.clone(),
            topic_message.clone(),
        )
        .await;

        let topic = cache_manger.get_topic_by_name(&topic_name).unwrap();

        let read_config = ReadConfig {
            max_record_num: 1,
            max_size: 1024 * 1024 * 1024,
        };
        let results = message_storage_adapter
            .read_by_offset(
                cluster_name().to_owned(),
                topic.topic_id.to_owned(),
                0,
                read_config,
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(topic_message.data, results[0].data)
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
        let cache_manger = test_build_mqtt_cache_manager();
        let message_storage_adapter = build_memory_storage_driver();
        let topic_name = format!("$SYS/brokers/{}-test", unique_id());
        let mqtt_topic = MQTTTopic::new(unique_id(), cluster_name(), topic_name.clone());
        cache_manger.add_topic(&topic_name, &mqtt_topic);
        let expect_data = "test_data".to_string();
        super::report_system_data(
            &client_pool,
            &cache_manger,
            &message_storage_adapter,
            &topic_name,
            || async { expect_data.clone() },
        )
        .await;

        let mqtt_topic = cache_manger.get_topic_by_name(&topic_name).unwrap();

        let read_config = ReadConfig {
            max_record_num: 1,
            max_size: 1024 * 1024 * 1024,
        };
        let results = message_storage_adapter
            .read_by_offset(
                cluster_name().to_owned(),
                mqtt_topic.topic_id.clone(),
                0,
                read_config,
            )
            .await
            .unwrap();

        let except_message =
            MqttMessage::build_system_topic_message(topic_name.clone(), expect_data).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data, except_message.data);
        assert_eq!(results[0].crc_num, except_message.crc_num);
    }
}
