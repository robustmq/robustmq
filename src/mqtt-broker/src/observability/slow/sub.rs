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

use std::collections::VecDeque;
use std::env;
use std::fs::File;

use common_base::tools::{get_local_ip, now_second};
use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::Searcher;
use log::info;
use serde::{Deserialize, Serialize};

use crate::handler::error::MqttBrokerError;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SlowSubData {
    sub_name: String,
    client_id: String,
    topic: String,
    time_ms: u128,
    node_info: String,
    create_time: u64,
}

impl SlowSubData {
    pub fn build(sub_name: String, client_id: String, topic_name: String, time_ms: u128) -> Self {
        let ip = get_local_ip();
        let node_info = format!("RobustMQ-MQTT@{}", ip);
        SlowSubData {
            sub_name,
            client_id,
            topic: topic_name,
            time_ms,
            node_info,
            create_time: now_second(),
        }
    }
}

pub fn record_slow_sub_data(slow_data: SlowSubData, whole_ms: u128) -> Result<(), MqttBrokerError> {
    let data = serde_json::to_string(&slow_data)?;

    if slow_data.time_ms > whole_ms {
        info!(target: "mqtt-broker::observability", "{}", data);
    }

    Ok(())
}

pub fn connect_regex_pattern(sub_name: String, client_id: String, topic: String) -> String {
    let mut pattern: String = String::new();
    pattern += "\\{";
    if !sub_name.is_empty() {
        pattern += ".*";
        pattern += sub_name.as_str();
    }
    if !client_id.is_empty() {
        pattern += ".*";
        pattern += client_id.as_str();
    }
    if !topic.is_empty() {
        pattern += ".*";
        pattern += topic.as_str();
    }
    pattern += ".*";
    pattern += "\\}";
    pattern
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SlowSubRecord {
    pub sub_name: String,
    pub client_id: String,
    pub topic: String,
    pub sort: String,
    pub line: u8,
}

pub fn read_slow_sub_record(
    slow_sub_record: SlowSubRecord,
    path: String,
) -> Result<VecDeque<String>, MqttBrokerError> {
    let regex_pattern = connect_regex_pattern(
        slow_sub_record.sub_name,
        slow_sub_record.client_id,
        slow_sub_record.topic,
    );
    if File::open(path).is_ok() {
        let matcher = RegexMatcher::new(regex_pattern.as_str())?;
        let mut matches_queue: VecDeque<String> = VecDeque::new();
        let current_dir = env::current_dir()?;
        println!("{:?}", current_dir.display());
        let file = File::open(path)?;
        Searcher::new().search_file(
            &matcher,
            &file,
            UTF8(|_lnum, line| {
                let match_byte = matcher.find(line.as_bytes())?.unwrap();
                if matches_queue.len() == slow_sub_record.line as usize {
                    matches_queue.pop_front();
                    matches_queue.push_back(line[match_byte].to_string())
                } else {
                    matches_queue.push_back(line[match_byte].to_string());
                }
                Ok(true)
            }),
        )?;
    }

    Ok(matches_queue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_pattern_param_is_empty() {
        let sub_name = "".to_string();
        let client_id = "".to_string();
        let topic_id = "".to_string();

        let regex_pattern = connect_regex_pattern(sub_name, client_id, topic_id);

        assert_eq!(regex_pattern, "\\{.*\\}");
    }

    #[test]
    fn test_regex_pattern_param_is_one() {
        let sub_name = "sub_name_test".to_string();
        let client_id = "".to_string();
        let topic_id = "".to_string();

        let regex_pattern = connect_regex_pattern(sub_name, client_id, topic_id);

        assert_eq!(regex_pattern, "\\{.*sub_name_test.*\\}");
    }

    #[test]
    fn test_regex_pattern_param_is_two() {
        let sub_name = "sub_name_test".to_string();
        let client_id = "client_id_test".to_string();
        let topic_id = "".to_string();

        let regex_pattern = connect_regex_pattern(sub_name, client_id, topic_id);

        assert_eq!(regex_pattern, "\\{.*sub_name_test.*client_id_test.*\\}");
    }

    #[test]
    fn test_regex_pattern_param_is_three() {
        let sub_name = "sub_name_test".to_string();
        let client_id = "client_id_test".to_string();
        let topic_id = "topic_id_test".to_string();

        let regex_pattern = connect_regex_pattern(sub_name, client_id, topic_id);

        assert_eq!(
            regex_pattern,
            "\\{.*sub_name_test.*client_id_test.*topic_id_test.*\\}"
        );
    }
}
