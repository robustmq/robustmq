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

// metrics
pub const SYSTEM_TOPIC_BROKERS_METRICS_BYTES_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/bytes/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_BYTES_SENT: &str = "$SYS/brokers/${node}/metrics/bytes/sent";

// MQTT Packet Received and Sent
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNECT: &str =
    "$SYS/brokers/${node}/metrics/packets/connect";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_CONNACK: &str =
    "$SYS/brokers/${node}/metrics/packets/connack";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/publish/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBLISH_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/publish/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBACK_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/puback/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREC_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrec/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBREL_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubrel/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PUBCOMP_MISSED: &str =
    "$SYS/brokers/${node}/metrics/packets/pubcomp/missed";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBSCRIBE: &str =
    "$SYS/brokers/${node}/metrics/packets/subscribe";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_SUBACK: &str =
    "$SYS/brokers/${node}/metrics/packets/suback";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBSCRIBE: &str =
    "$SYS/brokers/${node}/metrics/packets/unsubscribe";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_UNSUBACK: &str =
    "$SYS/brokers/${node}/metrics/packets/unsuback";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGREQ: &str =
    "$SYS/brokers/${node}/metrics/packets/pingreq";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_PINGRESP: &str =
    "$SYS/brokers/${node}/metrics/packets/pingresp";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/packets/disconnect/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_DISCONNECT_SENT: &str =
    "$SYS/brokers/${node}/metrics/packets/disconnect/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_PACKETS_AUTH: &str =
    "$SYS/brokers/${node}/metrics/packets/auth";
// MQTT Message Received and Sent
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED: &str =
    "$SYS/brokers/${node}/metrics/messages/expired";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED: &str =
    "$SYS/brokers/${node}/metrics/messages/retained";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED: &str =
    "$SYS/brokers/${node}/metrics/messages/dropped";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD: &str =
    "$SYS/brokers/${node}/metrics/messages/forward";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos0/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos0/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos1/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos1/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/received";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/sent";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/expired";
pub const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/dropped";
