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

use crate::{counter_metric_get, counter_metric_inc, register_counter_metric};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct NetworkLabel {}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct SessionLabel {
    pub client_id: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct ConnectionLabel {
    pub connection_id: u64,
}

register_counter_metric!(
    MQTT_SESSION_CREATED,
    "mqtt_session_created",
    "Number of MQTT sessions created",
    NetworkLabel
);

register_counter_metric!(
    MQTT_SESSION_DELETED,
    "mqtt_session_deleted",
    "Number of MQTT sessions deleted",
    NetworkLabel
);

register_counter_metric!(
    SESSION_MESSAGES_IN,
    "session_messages_in",
    "Total number of messages received by session",
    SessionLabel
);

register_counter_metric!(
    SESSION_MESSAGES_OUT,
    "session_messages_out",
    "Total number of messages sent by session",
    SessionLabel
);

register_counter_metric!(
    CONNECTION_MESSAGES_IN,
    "connection_messages_in",
    "Total number of messages received by connection",
    ConnectionLabel
);

register_counter_metric!(
    CONNECTION_MESSAGES_OUT,
    "connection_messages_out",
    "Total number of messages sent by connection",
    ConnectionLabel
);

pub fn record_mqtt_session_created() {
    let label = NetworkLabel {};
    counter_metric_inc!(MQTT_SESSION_CREATED, label);
}

pub fn record_mqtt_session_deleted() {
    let label = NetworkLabel {};
    counter_metric_inc!(MQTT_SESSION_DELETED, label);
}

pub fn record_session_messages_in(client_id: &str) {
    let label = SessionLabel {
        client_id: client_id.to_string(),
    };
    counter_metric_inc!(SESSION_MESSAGES_IN, label);
}

pub fn get_session_messages_in(client_id: &str) -> u64 {
    let label = SessionLabel {
        client_id: client_id.to_string(),
    };
    let mut result = 0u64;
    counter_metric_get!(SESSION_MESSAGES_IN, label, result);
    result
}

pub fn record_session_messages_out(client_id: &str) {
    let label = SessionLabel {
        client_id: client_id.to_string(),
    };
    counter_metric_inc!(SESSION_MESSAGES_OUT, label);
}

pub fn get_session_messages_out(client_id: &str) -> u64 {
    let label = SessionLabel {
        client_id: client_id.to_string(),
    };
    let mut result = 0u64;
    counter_metric_get!(SESSION_MESSAGES_OUT, label, result);
    result
}

pub fn record_connection_messages_in(connection_id: u64) {
    let label = ConnectionLabel { connection_id };
    counter_metric_inc!(CONNECTION_MESSAGES_IN, label);
}

pub fn get_connection_messages_in(connection_id: u64) -> u64 {
    let label = ConnectionLabel { connection_id };
    let mut result = 0u64;
    counter_metric_get!(CONNECTION_MESSAGES_IN, label, result);
    result
}

pub fn record_connection_messages_out(connection_id: u64) {
    let label = ConnectionLabel { connection_id };
    counter_metric_inc!(CONNECTION_MESSAGES_OUT, label);
}

pub fn get_connection_messages_out(connection_id: u64) -> u64 {
    let label = ConnectionLabel { connection_id };
    let mut result = 0u64;
    counter_metric_get!(CONNECTION_MESSAGES_OUT, label, result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_message_metrics() {
        record_session_messages_in("client001");
        let _count = get_session_messages_in("client001");

        record_session_messages_out("client001");
        let _count = get_session_messages_out("client001");
    }

    #[test]
    fn test_session_label_equality() {
        let label1 = SessionLabel {
            client_id: "client001".to_string(),
        };
        let label2 = SessionLabel {
            client_id: "client001".to_string(),
        };
        let label3 = SessionLabel {
            client_id: "client002".to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }

    #[test]
    fn test_connection_message_metrics() {
        record_connection_messages_in(12345);
        let _count = get_connection_messages_in(12345);

        record_connection_messages_out(12345);
        let _count = get_connection_messages_out(12345);
    }

    #[test]
    fn test_connection_label_equality() {
        let label1 = ConnectionLabel {
            connection_id: 12345,
        };
        let label2 = ConnectionLabel {
            connection_id: 12345,
        };
        let label3 = ConnectionLabel {
            connection_id: 67890,
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }
}
