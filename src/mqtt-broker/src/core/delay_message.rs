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

use super::error::MqttBrokerError;
use common_base::tools::now_second;
use delay_message::manager::{
    DelayMessageManager, DELAY_MESSAGE_FLAG, DELAY_MESSAGE_RECV_MS, DELAY_MESSAGE_TARGET_MS,
};
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{Publish, PublishProperties};

const DELAY_PUBLISH_MESSAGE_PREFIXED: &str = "$delayed/";
const MAX_DELAY_SECONDS: u64 = 42949669;
const TIMESTAMP_THRESHOLD: u64 = 1_000_000_000;

#[derive(Debug, Clone)]
pub struct DelayPublishTopic {
    pub target_topic_name: String,
    pub target_shard_name: Option<String>,
    pub delay_timestamp: u64,
}

/// Checks if a topic is a delayed publish topic.
/// Returns true only if the topic starts with "$delayed/" (with trailing slash).
pub fn is_delay_topic(topic: &str) -> bool {
    topic.starts_with(DELAY_PUBLISH_MESSAGE_PREFIXED)
}

/// Decodes a delayed publish topic into its components.
///
/// Format: `$delayed/{DelayInterval}/{TopicName}`
/// - DelayInterval: Delay time in seconds or Unix timestamp
/// - TopicName: Target topic name
///
/// Returns `DelayPublishTopic` with parsed delay timestamp and target topic.
pub fn decode_delay_topic(topic: &str) -> Result<DelayPublishTopic, MqttBrokerError> {
    let parts: Vec<&str> = topic.split('/').collect();

    if parts.len() < 3 {
        return Err(MqttBrokerError::NotConformDeferredTopic(topic.to_string()));
    }

    if parts[0] != "$delayed" {
        return Err(MqttBrokerError::NotConformDeferredTopic(topic.to_string()));
    }

    let delay_timestamp = parts[1]
        .parse::<u64>()
        .map_err(|_| MqttBrokerError::NotConformDeferredTopic(topic.to_string()))?;

    let is_absolute = delay_timestamp > TIMESTAMP_THRESHOLD;

    if is_absolute {
        if delay_timestamp < now_second() {
            return Err(MqttBrokerError::NotConformDeferredTopic(format!(
                "Timestamp {} is in the past",
                delay_timestamp
            )));
        }
    } else if delay_timestamp > MAX_DELAY_SECONDS {
        return Err(MqttBrokerError::DelayIntervalTooLarge(delay_timestamp));
    }

    let target_topic_name = if parts.len() > 2 {
        let topic = format!("/{}", parts[2..].join("/"));
        if topic == "/" {
            return Err(MqttBrokerError::EmptyDelayTopicName);
        }
        topic
    } else {
        return Err(MqttBrokerError::EmptyDelayTopicName);
    };

    Ok(DelayPublishTopic {
        target_topic_name,
        target_shard_name: None,
        delay_timestamp,
    })
}

/// Saves a delay message with metadata in UserProperties.
///
/// Adds the following UserProperties:
/// - delay_message_flag: "true"
/// - delay_message_recv_ms: Current timestamp
/// - delay_message_target_ms: Target delivery timestamp
pub async fn save_delay_message(
    delay_message_manager: &Arc<DelayMessageManager>,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
    client_id: &str,
    message_expire: u64,
    delay_info: &DelayPublishTopic,
) -> Result<Option<String>, MqttBrokerError> {
    let recv_time = now_second();
    let trigger_time = now_second() + delay_info.delay_timestamp;
    let user_properties = vec![
        (DELAY_MESSAGE_FLAG.to_string(), "true".to_string()),
        (DELAY_MESSAGE_RECV_MS.to_string(), recv_time.to_string()),
        (
            DELAY_MESSAGE_TARGET_MS.to_string(),
            trigger_time.to_string(),
        ),
    ];

    let mut new_publish_properties = publish_properties.clone().unwrap_or_default();
    new_publish_properties.user_properties = user_properties;

    let record = MqttMessage::build_record(
        client_id,
        publish,
        &Some(new_publish_properties),
        message_expire,
    )
    .ok_or(MqttBrokerError::FailedToBuildMessage)?;

    let target_shard_name = delay_info
        .target_shard_name
        .as_ref()
        .ok_or(MqttBrokerError::MissingTargetShardName)?;

    delay_message_manager
        .send(target_shard_name, trigger_time, record)
        .await?;

    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_delay_message_test() {
        assert!(is_delay_topic("$delayed/60/a/b"));
        assert!(!is_delay_topic("$delayedtopic"));
        assert!(!is_delay_topic("/60/a/b"));
        assert!(!is_delay_topic("$delayed"));
    }

    #[test]
    fn decode_delay_message_test() {
        let msg = decode_delay_topic("$delayed/60/a/b").unwrap();
        assert_eq!(msg.target_topic_name, "/a/b");
        assert_eq!(msg.delay_timestamp, now_second() + 60);

        let msg = decode_delay_topic("$delayed/0/topic").unwrap();
        assert_eq!(msg.delay_timestamp, now_second());
    }

    #[test]
    fn decode_delay_message_error_test() {
        assert!(decode_delay_topic("$delayed").is_err());
        assert!(decode_delay_topic("$delayed/60").is_err());
        assert!(decode_delay_topic("$delayed/abc/topic").is_err());
        assert!(decode_delay_topic("$delayed/60/").is_err());
        assert!(decode_delay_topic(&format!("$delayed/{}/test", MAX_DELAY_SECONDS + 1)).is_err());
        assert!(decode_delay_topic(&format!("$delayed/{}/test", now_second() - 3600)).is_err());
    }
}
