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

use crate::{counter_metric_inc, counter_metric_inc_by, register_counter_metric};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct TopicLabel {
    pub topic: String,
}

register_counter_metric!(
    TOPIC_MESSAGES_WRITTEN,
    "topic_messages_written",
    "Total number of messages written to topic",
    TopicLabel
);

register_counter_metric!(
    TOPIC_BYTES_WRITTEN,
    "topic_bytes_written",
    "Total bytes of messages written to topic",
    TopicLabel
);

register_counter_metric!(
    TOPIC_MESSAGES_SENT,
    "topic_messages_sent",
    "Total number of messages sent from topic",
    TopicLabel
);

register_counter_metric!(
    TOPIC_BYTES_SENT,
    "topic_bytes_sent",
    "Total bytes of messages sent from topic",
    TopicLabel
);

pub fn record_topic_messages_written(topic: &str) {
    let label = TopicLabel {
        topic: topic.to_string(),
    };
    counter_metric_inc!(TOPIC_MESSAGES_WRITTEN, label);
}

pub fn record_topic_bytes_written(topic: &str, bytes: u64) {
    let label = TopicLabel {
        topic: topic.to_string(),
    };
    counter_metric_inc_by!(TOPIC_BYTES_WRITTEN, label, bytes);
}

pub fn record_topic_messages_sent(topic: &str) {
    let label = TopicLabel {
        topic: topic.to_string(),
    };
    counter_metric_inc!(TOPIC_MESSAGES_SENT, label);
}

pub fn record_topic_bytes_sent(topic: &str, bytes: u64) {
    let label = TopicLabel {
        topic: topic.to_string(),
    };
    counter_metric_inc_by!(TOPIC_BYTES_SENT, label, bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_metrics() {
        // Test write metrics
        record_topic_messages_written("test/topic1");
        record_topic_bytes_written("test/topic1", 1024);

        // Test send metrics
        record_topic_messages_sent("test/topic2");
        record_topic_bytes_sent("test/topic2", 2048);

        // Verify metrics are recorded (actual values would need registry access)
        // This is a basic smoke test to ensure functions execute without panic
    }

    #[test]
    fn test_topic_label_equality() {
        let label1 = TopicLabel {
            topic: "test/topic".to_string(),
        };
        let label2 = TopicLabel {
            topic: "test/topic".to_string(),
        };
        let label3 = TopicLabel {
            topic: "different/topic".to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }
}
