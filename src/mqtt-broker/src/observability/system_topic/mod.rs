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

use std::sync::Arc;
use std::time::Duration;

use crate::handler::cache::CacheManager;
use crate::handler::topic::try_init_topic;
use crate::observability::system_topic::packet::bytes::{
    SYSTEM_TOPIC_BROKERS_METRICS_BYTES_RECEIVED, SYSTEM_TOPIC_BROKERS_METRICS_BYTES_SENT,
};
use crate::observability::system_topic::packet::messages::{
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
use crate::observability::system_topic::packet::packets::{
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
use crate::observability::system_topic::stats::client::{
    SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX,
};
use crate::observability::system_topic::stats::route::{
    SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT, SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX,
};
use crate::observability::system_topic::stats::subscription::{
    SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_MAX,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT, SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_MAX,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT,
    SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX,
};
use crate::observability::system_topic::sysmon::{
    SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE, SYSTEM_TOPIC_BROKERS_ALARMS_DEACTIVATE,
};
use crate::storage::message::MessageStorage;
use common_base::tools::get_local_ip;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error};

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
pub mod warn;

pub struct SystemTopic<S> {
    pub metadata_cache: Arc<CacheManager>,
    pub message_storage_adapter: Arc<S>,
    pub client_pool: Arc<ClientPool>,
}

impl<S> SystemTopic<S>
where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        message_storage_adapter: Arc<S>,
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
        let mut stop_rx = stop_send.subscribe();
        loop {
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("System topic thread stopped successfully");
                            break;
                        }
                    }
                }
                _ = self.report_info()=>{}
            }
            sleep(Duration::from_secs(60)).await;
        }
    }

    pub async fn report_info(&self) {
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
        report_alarm_info(
            &self.client_pool,
            &self.metadata_cache,
            &self.message_storage_adapter,
        )
        .await;
    }

    pub async fn try_init_system_topic(&self) {
        let results = self.get_all_system_topic();
        for topic_name in results {
            let new_topic_name = replace_topic_name(topic_name);
            match try_init_topic(
                &new_topic_name,
                &self.metadata_cache,
                &self.message_storage_adapter,
                &self.client_pool,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    panic!(
                        "Initializing system topic {} Failed, error message :{}",
                        new_topic_name, e
                    );
                }
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
            SYSTEM_TOPIC_BROKERS_ALARMS_DEACTIVATE.to_string(),
        ]
    }
}

pub(crate) async fn report_alarm_info<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    sysmon::st_check_system_alarm(client_pool, metadata_cache, message_storage_adapter).await;
}

pub(crate) async fn report_broker_info<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    broker::report_cluster_status(client_pool, metadata_cache, message_storage_adapter).await;
    broker::report_broker_version(client_pool, metadata_cache, message_storage_adapter).await;
    broker::report_broker_time(client_pool, metadata_cache, message_storage_adapter).await;
    broker::report_broker_sysdescr(client_pool, metadata_cache, message_storage_adapter).await;
}

pub(crate) async fn report_packet_info<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    // bytes
    packet::bytes::report_broker_metrics_bytes_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::bytes::report_broker_metrics_bytes_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    // connect
    packet::packets::report_broker_metrics_packets_connect(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_connack(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    // publish
    packet::packets::report_broker_metrics_packets_publish_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_publish_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_puback_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_puback_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_puback_missed(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubrec_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubrec_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubrec_missed(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubrel_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubrel_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubrel_missed(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubcomp_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubcomp_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pubcomp_missed(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_subscribe(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_suback(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_unsubscribe(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_unsuback(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pingreq(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_pingresp(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_disconnect_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_disconnect_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::packets::report_broker_metrics_packets_auth(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    // messages
    packet::messages::report_broker_metrics_messages_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_expired(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_retained(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_dropped(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_forward(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos0_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos0_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos1_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos1_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos2_received(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos2_sent(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos2_expired(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    packet::messages::report_broker_metrics_messages_qos2_dropped(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
}

pub(crate) async fn report_stats_info<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    // client
    stats::client::report_broker_stat_connections_count(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::client::report_broker_stat_connections_max(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;

    // route
    stats::route::report_broker_stat_routes_count(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::route::report_broker_stat_routes_max(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;

    // subscription
    stats::subscription::report_broker_stat_suboptions_count(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::subscription::report_broker_stat_suboptions_max(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::subscription::report_broker_stat_subscribers_count(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::subscription::report_broker_stat_subscribers_max(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::subscription::report_broker_stat_subscriptions_count(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::subscription::report_broker_stat_subscriptions_max(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::subscription::report_broker_stat_subscriptions_shared_count(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::subscription::report_broker_stat_subscriptions_shared_max(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;

    //topics
    stats::topics::report_broker_stat_topics_count(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
    stats::topics::report_broker_stat_topics_max(
        client_pool,
        metadata_cache,
        message_storage_adapter,
    )
    .await;
}

pub(crate) async fn report_system_data<S, F, Fut>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
    topic_const: &str,
    data_generator: F,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
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

pub(crate) async fn write_topic_data<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: String,
    record: Record,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
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
    use crate::handler::cache::CacheManager;
    use crate::observability::system_topic::write_topic_data;
    use crate::storage::message::cluster_name;
    use common_base::tools::{get_local_ip, unique_id};
    use common_config::mqtt::init_broker_mqtt_conf_by_path;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::adapter::read_config::ReadConfig;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::topic::MqttTopic;
    use std::sync::Arc;
    use storage_adapter::memory::MemoryStorageAdapter;
    use storage_adapter::storage::StorageAdapter;

    #[tokio::test]
    async fn test_write_topic_data() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let cache_manger = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let topic_name = format!("$SYS/brokers/{}-test", unique_id());
        let mqtt_topic = MqttTopic::new(unique_id(), cluster_name(), topic_name.clone());
        cache_manger.add_topic(&topic_name, &mqtt_topic);

        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());
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
        let expected_name = format!("$SYS/brokers/{}/version", local_ip);
        assert!(replaced_name.contains("brokers/"));
        assert!(!replaced_name.contains("${node}"));
        assert_eq!(expected_name, replaced_name)
    }

    #[tokio::test]
    async fn test_report_system_data() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let client_pool = Arc::new(ClientPool::new(3));
        let cache_manger = Arc::new(CacheManager::new(client_pool.clone(), cluster_name()));
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let topic_name = format!("$SYS/brokers/{}-test", unique_id());
        let mqtt_topic = MqttTopic::new(unique_id(), cluster_name(), topic_name.clone());
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
