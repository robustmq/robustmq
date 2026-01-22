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
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// MQTT Packet Received and Sent
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT: &str =
    "$SYS/brokers/${node}/metrics/packets/connect";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK: &str =
    "$SYS/brokers/${node}/metrics/packets/connack";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/publish/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/publish/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/missed";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE: &str =
    "$SYS/brokers/${node}/metrics/packets/subscribe";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK: &str =
    "$SYS/brokers/${node}/metrics/packets/suback";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE: &str =
    "$SYS/brokers/${node}/metrics/packets/unsubscribe";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK: &str =
    "$SYS/brokers/${node}/metrics/packets/unsuback";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ: &str =
    "$SYS/brokers/${node}/metrics/packets/pingreq";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP: &str =
    "$SYS/brokers/${node}/metrics/packets/pingresp";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/disconnect/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/disconnect/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH: &str =
    "$SYS/brokers/${node}/metrics/packets/auth";

pub(crate) async fn report_broker_metrics_packets(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_connect().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK,
        || async {
            "".to_string()
            // metadata_cache.get_packets_connack().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_publish_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_publish_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_puback_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_puback_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_puback_missed().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubrec_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubrec_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubrec_missed().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubrel_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubrel_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubrel_missed().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubcomp_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_disconnect_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_disconnect_sent().to_string()
        },
    )
    .await;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH,
        || async {
            "".to_string()
            // metadata_cache.get_packets_auth().to_string()
        },
    )
    .await;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK,
        || async {
            "".to_string()
            // metadata_cache.get_packets_suback().to_string()
        },
    )
    .await;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE,
        || async {
            "".to_string()
            // metadata_cache.get_packets_unsubscribe().to_string()
        },
    )
    .await;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK,
        || async {
            "".to_string()
            // metadata_cache.get_packets_unsuback().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pingreq().to_string()
        },
    )
    .await;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pingresp().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubcomp_sent().to_string()
        },
    )
    .await;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED,
        || async {
            "".to_string()
            // metadata_cache.get_packets_pubcomp_missed().to_string()
        },
    )
    .await;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE,
        || async {
            "".to_string()
            // metadata_cache.get_packets_subscribe().to_string()
        },
    )
    .await;
}
