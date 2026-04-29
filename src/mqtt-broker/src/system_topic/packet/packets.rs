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
use common_metrics::mqtt::packets::{
    record_mqtt_total_packets_auth_get, record_mqtt_total_packets_connack_get,
    record_mqtt_total_packets_connect_get, record_mqtt_total_packets_disconnect_received_get,
    record_mqtt_total_packets_disconnect_sent_get, record_mqtt_total_packets_pingreq_get,
    record_mqtt_total_packets_pingresp_get, record_mqtt_total_packets_puback_received_get,
    record_mqtt_total_packets_puback_sent_get, record_mqtt_total_packets_pubcomp_received_get,
    record_mqtt_total_packets_pubcomp_sent_get, record_mqtt_total_packets_publish_received_get,
    record_mqtt_total_packets_publish_sent_get, record_mqtt_total_packets_pubrec_received_get,
    record_mqtt_total_packets_pubrec_sent_get, record_mqtt_total_packets_pubrel_received_get,
    record_mqtt_total_packets_pubrel_sent_get, record_mqtt_total_packets_received_get,
    record_mqtt_total_packets_sent_get, record_mqtt_total_packets_suback_get,
    record_mqtt_total_packets_subscribe_get, record_mqtt_total_packets_unsuback_get,
    record_mqtt_total_packets_unsubscribe_get,
};
use grpc_clients::pool::ClientPool;
use serde::Serialize;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

use crate::system_topic::SYSTEM_TOPIC_BROKERS_METRICS_PACKETS;

/// Aggregated packet counters published as a single JSON payload to
/// `$SYS/brokers/metrics/packets`.
#[derive(Debug, Serialize)]
pub(crate) struct BrokerPacketsMetrics {
    // Total packets received / sent across all types
    pub received: u64,
    pub sent: u64,

    // CONNECT / CONNACK
    pub connect: u64,
    pub connack: u64,

    // PUBLISH
    pub publish_received: u64,
    pub publish_sent: u64,

    // PUBACK (QoS 1 acknowledgement)
    pub puback_received: u64,
    pub puback_sent: u64,
    pub puback_missed: u64,

    // PUBREC (QoS 2 step 1)
    pub pubrec_received: u64,
    pub pubrec_sent: u64,
    pub pubrec_missed: u64,

    // PUBREL (QoS 2 step 2)
    pub pubrel_received: u64,
    pub pubrel_sent: u64,
    pub pubrel_missed: u64,

    // PUBCOMP (QoS 2 step 3)
    pub pubcomp_received: u64,
    pub pubcomp_sent: u64,
    pub pubcomp_missed: u64,

    // SUBSCRIBE / SUBACK
    pub subscribe: u64,
    pub suback: u64,

    // UNSUBSCRIBE / UNSUBACK
    pub unsubscribe: u64,
    pub unsuback: u64,

    // PINGREQ / PINGRESP (keep-alive)
    pub pingreq: u64,
    pub pingresp: u64,

    // DISCONNECT
    pub disconnect_received: u64,
    pub disconnect_sent: u64,

    // AUTH (MQTT 5.0 enhanced authentication)
    pub auth: u64,
}

impl BrokerPacketsMetrics {
    pub(crate) fn collect() -> Self {
        BrokerPacketsMetrics {
            received: record_mqtt_total_packets_received_get(),
            sent: record_mqtt_total_packets_sent_get(),
            connect: record_mqtt_total_packets_connect_get(),
            connack: record_mqtt_total_packets_connack_get(),
            publish_received: record_mqtt_total_packets_publish_received_get(),
            publish_sent: record_mqtt_total_packets_publish_sent_get(),
            puback_received: record_mqtt_total_packets_puback_received_get(),
            puback_sent: record_mqtt_total_packets_puback_sent_get(),
            puback_missed: 0,
            pubrec_received: record_mqtt_total_packets_pubrec_received_get(),
            pubrec_sent: record_mqtt_total_packets_pubrec_sent_get(),
            pubrec_missed: 0,
            pubrel_received: record_mqtt_total_packets_pubrel_received_get(),
            pubrel_sent: record_mqtt_total_packets_pubrel_sent_get(),
            pubrel_missed: 0,
            pubcomp_received: record_mqtt_total_packets_pubcomp_received_get(),
            pubcomp_sent: record_mqtt_total_packets_pubcomp_sent_get(),
            pubcomp_missed: 0,
            subscribe: record_mqtt_total_packets_subscribe_get(),
            suback: record_mqtt_total_packets_suback_get(),
            unsubscribe: record_mqtt_total_packets_unsubscribe_get(),
            unsuback: record_mqtt_total_packets_unsuback_get(),
            pingreq: record_mqtt_total_packets_pingreq_get(),
            pingresp: record_mqtt_total_packets_pingresp_get(),
            disconnect_received: record_mqtt_total_packets_disconnect_received_get(),
            disconnect_sent: record_mqtt_total_packets_disconnect_sent_get(),
            auth: record_mqtt_total_packets_auth_get(),
        }
    }
}

pub(crate) async fn report_broker_metrics_packets(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    let metrics = BrokerPacketsMetrics::collect();
    let payload = serde_json::to_string(&metrics).unwrap_or_default();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS,
        || async move { payload },
    )
    .await;
}
