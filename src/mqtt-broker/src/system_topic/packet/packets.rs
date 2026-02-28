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
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// MQTT Packet Received and Sent
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED: &str =
    "$SYS/brokers/metrics/packets/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT: &str =
    "$SYS/brokers/metrics/packets/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT: &str =
    "$SYS/brokers/metrics/packets/connect";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK: &str =
    "$SYS/brokers/metrics/packets/connack";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED: &str =
    "$SYS/brokers/metrics/packets/publish/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT: &str =
    "$SYS/brokers/metrics/packets/publish/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED: &str =
    "$SYS/brokers/metrics/packets/puback/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT: &str =
    "$SYS/brokers/metrics/packets/puback/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED: &str =
    "$SYS/brokers/metrics/packets/puback/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED: &str =
    "$SYS/brokers/metrics/packets/pubrec/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT: &str =
    "$SYS/brokers/metrics/packets/pubrec/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED: &str =
    "$SYS/brokers/metrics/packets/pubrec/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED: &str =
    "$SYS/brokers/metrics/packets/pubrel/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT: &str =
    "$SYS/brokers/metrics/packets/pubrel/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED: &str =
    "$SYS/brokers/metrics/packets/pubrel/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED: &str =
    "$SYS/brokers/metrics/packets/pubcomp/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT: &str =
    "$SYS/brokers/metrics/packets/pubcomp/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED: &str =
    "$SYS/brokers/metrics/packets/pubcomp/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE: &str =
    "$SYS/brokers/metrics/packets/subscribe";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK: &str =
    "$SYS/brokers/metrics/packets/suback";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE: &str =
    "$SYS/brokers/metrics/packets/unsubscribe";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK: &str =
    "$SYS/brokers/metrics/packets/unsuback";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ: &str =
    "$SYS/brokers/metrics/packets/pingreq";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP: &str =
    "$SYS/brokers/metrics/packets/pingresp";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED: &str =
    "$SYS/brokers/metrics/packets/disconnect/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT: &str =
    "$SYS/brokers/metrics/packets/disconnect/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH: &str =
    "$SYS/brokers/metrics/packets/auth";

pub(crate) async fn report_broker_metrics_packets(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    macro_rules! report {
        ($topic:expr, $value:expr) => {
            let v = $value;
            report_system_data(
                client_pool,
                metadata_cache,
                storage_driver_manager,
                $topic,
                || async move { v.to_string() },
            )
            .await;
        };
    }

    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED,
        record_mqtt_total_packets_received_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT,
        record_mqtt_total_packets_sent_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT,
        record_mqtt_total_packets_connect_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK,
        record_mqtt_total_packets_connack_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED,
        record_mqtt_total_packets_publish_received_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT,
        record_mqtt_total_packets_publish_sent_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED,
        record_mqtt_total_packets_puback_received_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT,
        record_mqtt_total_packets_puback_sent_get()
    );
    // missed: not tracked, report 0
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED,
        || async { "0".to_string() },
    )
    .await;
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED,
        record_mqtt_total_packets_pubrec_received_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT,
        record_mqtt_total_packets_pubrec_sent_get()
    );
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED,
        || async { "0".to_string() },
    )
    .await;
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED,
        record_mqtt_total_packets_pubrel_received_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT,
        record_mqtt_total_packets_pubrel_sent_get()
    );
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED,
        || async { "0".to_string() },
    )
    .await;
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED,
        record_mqtt_total_packets_pubcomp_received_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT,
        record_mqtt_total_packets_pubcomp_sent_get()
    );
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED,
        || async { "0".to_string() },
    )
    .await;
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE,
        record_mqtt_total_packets_subscribe_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK,
        record_mqtt_total_packets_suback_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE,
        record_mqtt_total_packets_unsubscribe_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK,
        record_mqtt_total_packets_unsuback_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ,
        record_mqtt_total_packets_pingreq_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP,
        record_mqtt_total_packets_pingresp_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED,
        record_mqtt_total_packets_disconnect_received_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT,
        record_mqtt_total_packets_disconnect_sent_get()
    );
    report!(
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH,
        record_mqtt_total_packets_auth_get()
    );
}
