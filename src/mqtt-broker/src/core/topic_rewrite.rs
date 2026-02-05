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

use super::cache::MQTTCacheManager;
use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;
use crate::subscribe::common::{decode_sub_path, is_match_sub_and_topic};
use common_base::enum_type::topic_rewrite_action_enum::TopicRewriteActionEnum;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use regex::Regex;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::warn;

pub async fn start_topic_rewrite_convert_thread(
    cache_manager: Arc<MQTTCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if let Err(e) = convert_rewrite_topic(cache_manager.clone()).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, 3000, &stop_send).await;
}

async fn convert_rewrite_topic(cache_manager: Arc<MQTTCacheManager>) -> ResultMqttBrokerError {
    if !cache_manager.is_re_calc_topic_rewrite().await {
        return Ok(());
    }
    let mut rules: Vec<MqttTopicRewriteRule> = cache_manager.get_all_topic_rewrite_rule();
    rules.sort_by_key(|rule| rule.timestamp);
    
    for topic in cache_manager.broker_cache.topic_list.iter() {
        let topic_name = topic.topic_name.clone();
        
        for rule in rules.iter() {
            let allow = rule.action == TopicRewriteActionEnum::All.to_string()
                || rule.action == TopicRewriteActionEnum::Publish.to_string();

            if !allow {
                continue;
            }

            if is_match_sub_and_topic(&rule.source_topic, &topic_name).is_ok() {
                match gen_rewrite_topic(topic_name.clone(), &rule.regex, &rule.dest_topic) {
                    Ok(new_topic_name) => {
                        if new_topic_name != topic_name {
                            cache_manager.add_new_rewrite_name(&topic_name, &new_topic_name);
                        }
                        break;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to compile regex for rule, topic:{}, pattern:{}, error:{}",
                            topic_name, rule.regex, e
                        );
                        continue;
                    }
                }
            }
        }
    }
    
    cache_manager.set_re_calc_topic_rewrite(false).await;
    Ok(())
}

fn gen_rewrite_topic(
    input: String,
    pattern: &str,
    template: &str,
) -> Result<String, MqttBrokerError> {
    let topic = decode_sub_path(&input);
    let re = Regex::new(pattern)?;
    
    if let Some(captures) = re.captures(topic.as_str()) {
        let mut rewrite_topic = template.to_string();
        
        for (i, capture) in captures.iter().skip(1).enumerate() {
            if let Some(matched_str) = capture {
                let placeholder = format!("${}", i + 1);
                rewrite_topic = rewrite_topic.replace(&placeholder, matched_str.as_str());
            }
        }
        
        return Ok(rewrite_topic);
    }
    
    Ok(input)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::core::tool::test_build_mqtt_cache_manager;

    use super::*;
    use common_base::tools::now_millis;
    use tokio::time::sleep;

    const SRC_TOPICS: [&str; 5] = ["y/a/z/b", "y/def", "x/1/2", "x/y/2", "x/y/z"];
    const DST_TOPICS: [&str; 5] = ["y/z/b", "y/def", "x/1/2", "z/y/2", "x/y/z"];

    #[tokio::test]
    async fn gen_rewrite_topic_test() {
        let cache_manager = build_rules().await;

        for (index, input) in SRC_TOPICS.iter().enumerate() {
            let d1 = DST_TOPICS[index].to_string();
            let mut t1: String = input.to_string();
            let mut rules = cache_manager.get_all_topic_rewrite_rule();
            rules.sort_by_key(|rule| rule.timestamp);
            for rule in rules.iter() {
                if is_match_sub_and_topic(&rule.source_topic, input).is_ok() {
                    let rewrite_topic =
                        gen_rewrite_topic(input.to_string(), &rule.regex, &rule.dest_topic);
                    assert!(rewrite_topic.is_ok());
                    t1 = rewrite_topic.unwrap();
                }
            }
            assert_eq!(t1, d1);
        }
    }

    async fn build_rules() -> Arc<MQTTCacheManager> {
        let rules = [
            SimpleRule::new(r"y/+/z/#", r"y/z/$2", r"^y/(.+)/z/(.+)$"),
            SimpleRule::new(r"x/#", r"z/y/x/$1", r"^x/y/(.+)$"),
            SimpleRule::new(r"x/y/+", r"z/y/$1", r"^x/y/(\d+)$"),
        ];

        let cache_manager = test_build_mqtt_cache_manager().await;
        for rule in rules.iter() {
            let rule = MqttTopicRewriteRule {
                action: TopicRewriteActionEnum::All.to_string(),
                source_topic: rule.source.to_string(),
                dest_topic: rule.destination.to_string(),
                regex: rule.regex.to_string(),
                timestamp: now_millis(),
            };
            cache_manager.add_topic_rewrite_rule(rule);
            sleep(Duration::from_millis(200)).await;
        }
        cache_manager
    }

    struct SimpleRule {
        source: String,
        destination: String,
        regex: String,
    }

    impl SimpleRule {
        fn new(source: &str, destination: &str, regex: &str) -> Self {
            SimpleRule {
                source: source.to_string(),
                destination: destination.to_string(),
                regex: regex.to_string(),
            }
        }
    }
}
