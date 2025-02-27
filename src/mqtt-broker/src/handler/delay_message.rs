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

pub struct DelayPublishTopic {
    pub topic: String,
    pub delay_timestamp: u64,
}

pub fn is_delay_message(topic: &str) -> bool {
    topic.starts_with(DELAY_PUBLISH_MESSAGE_PREFIXED)
}

pub fn decode_delay_topic(topic: &str) -> Result<Option<DelayPublishTopic>, MqttBrokerError> {
    if !is_delay_message(topic) {
        return Ok(None);
    }
    let mut str_slice: Vec<&str> = topic.split("/").collect();
    if str_slice.len() < 3 {
        return Ok(None);
    }
    let delay_timestamp = str_slice[1].parse::<u64>()?;
    str_slice.remove(0);
    str_slice.remove(0);
    let msg = DelayPublishTopic {
        topic: format!("/{}", str_slice.join("/")),
        delay_timestamp,
    };
    Ok(Some(msg))
}

#[cfg(test)]
mod test {
    #[test]
    pub fn is_delay_message_test() {
        let topic_name = "$delayed/60/a/b";
        assert!(super::is_delay_message(topic_name));

        let topic_name = "/60/a/b";
        assert!(!super::is_delay_message(topic_name));

        let topic_name = "/a/b";
        assert!(!super::is_delay_message(topic_name));
    }

    #[test]
    pub fn decode_delay_message_test() {
        let topic_name = "$delayed/60/a/b";
        let msg = super::decode_delay_topic(topic_name).unwrap();
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.topic, "/a/b");
        assert_eq!(msg.delay_timestamp, 60);
    }
}
