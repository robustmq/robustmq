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

use super::error::MqttBrokerError;

const DELAY_PUBLISH_MESSAGE_PREFIXED: &str = "$delayed";
pub const DELAY_MESSAGE_FLAG: &str = "delay_message_flag";
pub const DELAY_MESSAGE_RECV_MS: &str = "delay_message_recv_ms";
pub const DELAY_MESSAGE_TARGET_MS: &str = "delay_message_save_ms";

#[derive(Debug, Clone)]
pub struct DelayPublishTopic {
    pub target_topic_name: String,
    pub target_shard_name: Option<String>,
    pub delay_timestamp: u64,
}

pub fn is_delay_topic(topic: &str) -> bool {
    topic.starts_with(DELAY_PUBLISH_MESSAGE_PREFIXED)
}

pub fn decode_delay_topic(topic: &str) -> Result<DelayPublishTopic, MqttBrokerError> {
    let mut str_slice: Vec<&str> = topic.split("/").collect();
    if str_slice.len() < 3 {
        return Err(MqttBrokerError::NotConformDeferredTopic(topic.to_string()));
    }

    let delay_timestamp = str_slice[1].parse::<u64>()?;
    str_slice.remove(0);
    str_slice.remove(0);
    let target_topic_name = format!("/{}", str_slice.join("/"));

    let msg = DelayPublishTopic {
        target_topic_name,
        target_shard_name: None,
        delay_timestamp,
    };

    Ok(msg)
}

#[cfg(test)]
mod test {
    #[test]
    pub fn is_delay_message_test() {
        let topic_name = "$delayed/60/a/b";
        assert!(super::is_delay_topic(topic_name));

        let topic_name = "/60/a/b";
        assert!(!super::is_delay_topic(topic_name));

        let topic_name = "/a/b";
        assert!(!super::is_delay_topic(topic_name));
    }

    #[test]
    pub fn decode_delay_message_test() {
        let topic_name = "$delayed/60/a/b";
        let msg = super::decode_delay_topic(topic_name).unwrap();
        assert_eq!(msg.target_topic_name, "/a/b");
        assert_eq!(msg.delay_timestamp, 60);
    }
}
