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

use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use common_base::tools::{get_local_ip, now_second};
use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::Searcher;
use log::info;
use protocol::broker_mqtt::broker_mqtt_admin::ListSlowSubscribeRequest;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SlowSubData {
    pub(crate) sub_name: String,
    pub(crate) client_id: String,
    pub(crate) topic: String,
    pub(crate) time_ms: u64,
    pub(crate) node_info: String,
    pub(crate) create_time: u64,
}

impl SlowSubData {
    pub fn build(sub_name: String, client_id: String, topic_name: String, time_ms: u64) -> Self {
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

pub fn record_slow_sub_data(slow_data: SlowSubData, whole_ms: u64) -> Result<(), MqttBrokerError> {
    let data = serde_json::to_string(&slow_data)?;

    if slow_data.time_ms > whole_ms {
        info!("{}", data);
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

pub fn read_slow_sub_record(
    search_options: ListSlowSubscribeRequest,
    path: PathBuf,
) -> Result<VecDeque<String>, MqttBrokerError> {
    let regex_pattern = connect_regex_pattern(
        search_options.sub_name,
        search_options.client_id,
        search_options.topic,
    );
    let file = File::open(path);
    let mut matches_queue: VecDeque<String> = VecDeque::new();
    if file.is_ok() {
        let matcher = RegexMatcher::new(regex_pattern.as_str())?;
        Searcher::new().search_file(
            &matcher,
            &file.unwrap(),
            UTF8(|_lnum, line| {
                let match_byte = matcher.find(line.as_bytes())?.unwrap();
                if matches_queue.len() == search_options.list as usize {
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

pub async fn enable_slow_sub(
    cache_manager: &Arc<CacheManager>,
    is_enable: bool,
) -> Result<(), MqttBrokerError> {
    let mut sub = cache_manager.get_slow_sub_config().clone();

    sub.enable = is_enable;

    cache_manager.update_slow_sub_config(sub).await?;

    Ok(())
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
