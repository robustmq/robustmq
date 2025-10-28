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

use crate::{
    counter_metric_get, counter_metric_inc, counter_metric_inc_by, register_counter_metric,
};
use prometheus_client::encoding::EncodeLabelSet;

pub const STATUS_SUCCESS: &str = "success";
pub const STATUS_FAILED: &str = "failed";

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct SubscribeLabel {
    pub client_id: String,
    pub path: String,
    pub status: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct SubscribeTopicLabel {
    pub client_id: String,
    pub path: String,
    pub topic_name: String,
    pub status: String,
}

register_counter_metric!(
    SUBSCRIBE_MESSAGES_SENT,
    "subscribe_messages_sent",
    "Total number of messages sent to subscription by client_id and path",
    SubscribeLabel
);

register_counter_metric!(
    SUBSCRIBE_TOPIC_MESSAGES_SENT,
    "subscribe_topic_messages_sent",
    "Total number of messages sent to subscription by client_id, path and topic_name",
    SubscribeTopicLabel
);

register_counter_metric!(
    SUBSCRIBE_BYTES_SENT,
    "subscribe_bytes_sent",
    "Total bytes of messages sent to subscription by client_id and path",
    SubscribeLabel
);

register_counter_metric!(
    SUBSCRIBE_TOPIC_BYTES_SENT,
    "subscribe_topic_bytes_sent",
    "Total bytes of messages sent to subscription by client_id, path and topic_name",
    SubscribeTopicLabel
);

pub fn record_subscribe_messages_sent(client_id: &str, path: &str, success: bool) {
    let label = SubscribeLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    counter_metric_inc!(SUBSCRIBE_MESSAGES_SENT, label);
}

pub fn get_subscribe_messages_sent(client_id: &str, path: &str, success: bool) -> u64 {
    let label = SubscribeLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    let mut result = 0u64;
    counter_metric_get!(SUBSCRIBE_MESSAGES_SENT, label, result);
    result
}

pub fn record_subscribe_topic_messages_sent(
    client_id: &str,
    path: &str,
    topic_name: &str,
    success: bool,
) {
    let label = SubscribeTopicLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        topic_name: topic_name.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    counter_metric_inc!(SUBSCRIBE_TOPIC_MESSAGES_SENT, label);
}

pub fn get_subscribe_topic_messages_sent(
    client_id: &str,
    path: &str,
    topic_name: &str,
    success: bool,
) -> u64 {
    let label = SubscribeTopicLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        topic_name: topic_name.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    let mut result = 0u64;
    counter_metric_get!(SUBSCRIBE_TOPIC_MESSAGES_SENT, label, result);
    result
}

pub fn record_subscribe_bytes_sent(client_id: &str, path: &str, bytes: u64, success: bool) {
    let label = SubscribeLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    counter_metric_inc_by!(SUBSCRIBE_BYTES_SENT, label, bytes);
}

pub fn get_subscribe_bytes_sent(client_id: &str, path: &str, success: bool) -> u64 {
    let label = SubscribeLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    let mut result = 0u64;
    counter_metric_get!(SUBSCRIBE_BYTES_SENT, label, result);
    result
}

pub fn record_subscribe_topic_bytes_sent(
    client_id: &str,
    path: &str,
    topic_name: &str,
    bytes: u64,
    success: bool,
) {
    let label = SubscribeTopicLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        topic_name: topic_name.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    counter_metric_inc_by!(SUBSCRIBE_TOPIC_BYTES_SENT, label, bytes);
}

pub fn get_subscribe_topic_bytes_sent(
    client_id: &str,
    path: &str,
    topic_name: &str,
    success: bool,
) -> u64 {
    let label = SubscribeTopicLabel {
        client_id: client_id.to_string(),
        path: path.to_string(),
        topic_name: topic_name.to_string(),
        status: if success {
            STATUS_SUCCESS
        } else {
            STATUS_FAILED
        }
        .to_string(),
    };
    let mut result = 0u64;
    counter_metric_get!(SUBSCRIBE_TOPIC_BYTES_SENT, label, result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_messages_metrics() {
        record_subscribe_messages_sent("client001", "sensor/+", true);
        let _count = get_subscribe_messages_sent("client001", "sensor/+", true);

        record_subscribe_topic_messages_sent("client001", "sensor/+", "sensor/temperature", true);
        let _count =
            get_subscribe_topic_messages_sent("client001", "sensor/+", "sensor/temperature", true);
    }

    #[test]
    fn test_subscribe_bytes_metrics() {
        record_subscribe_bytes_sent("client001", "sensor/+", 1024, true);
        let _bytes = get_subscribe_bytes_sent("client001", "sensor/+", true);

        record_subscribe_topic_bytes_sent(
            "client001",
            "sensor/+",
            "sensor/temperature",
            2048,
            true,
        );
        let _bytes =
            get_subscribe_topic_bytes_sent("client001", "sensor/+", "sensor/temperature", true);
    }

    #[test]
    fn test_subscribe_label_equality() {
        let label1 = SubscribeLabel {
            client_id: "client001".to_string(),
            path: "sensor/+".to_string(),
            status: STATUS_SUCCESS.to_string(),
        };
        let label2 = SubscribeLabel {
            client_id: "client001".to_string(),
            path: "sensor/+".to_string(),
            status: STATUS_SUCCESS.to_string(),
        };
        let label3 = SubscribeLabel {
            client_id: "client002".to_string(),
            path: "sensor/+".to_string(),
            status: STATUS_SUCCESS.to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }

    #[test]
    fn test_subscribe_topic_label_equality() {
        let label1 = SubscribeTopicLabel {
            client_id: "client001".to_string(),
            path: "sensor/+".to_string(),
            topic_name: "sensor/temperature".to_string(),
            status: STATUS_SUCCESS.to_string(),
        };
        let label2 = SubscribeTopicLabel {
            client_id: "client001".to_string(),
            path: "sensor/+".to_string(),
            topic_name: "sensor/temperature".to_string(),
            status: STATUS_SUCCESS.to_string(),
        };
        let label3 = SubscribeTopicLabel {
            client_id: "client001".to_string(),
            path: "sensor/+".to_string(),
            topic_name: "sensor/humidity".to_string(),
            status: STATUS_SUCCESS.to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }
}
