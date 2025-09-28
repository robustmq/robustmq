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

use common_base::enum_type::topic_rewrite_action_enum::TopicRewriteActionEnum;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use regex::Regex;
use tracing::info;

use crate::handler::error::MqttBrokerError;
use crate::subscribe::common::{decode_sub_path, is_match_sub_and_topic};

use super::cache::MQTTCacheManager;

pub fn convert_publish_topic_by_rewrite_rule(
    cache_manager: &Arc<MQTTCacheManager>,
    topic_name: String,
) -> Result<Option<String>, MqttBrokerError> {
    gen_convert_rewrite_name(cache_manager, &topic_name)
}

pub fn convert_sub_path_by_rewrite_rule(
    cache_manager: &Arc<MQTTCacheManager>,
    path: &str,
) -> Result<Option<String>, MqttBrokerError> {
    gen_convert_rewrite_name(cache_manager, path)
}

fn gen_convert_rewrite_name(
    cache_manager: &Arc<MQTTCacheManager>,
    name: &str,
) -> Result<Option<String>, MqttBrokerError> {
    let mut rules: Vec<MqttTopicRewriteRule> = cache_manager.get_all_topic_rewrite_rule();
    rules.sort_by_key(|rule| rule.timestamp);
    let mut new_topic_name = "".to_string();
    for rule in rules.iter() {
        let allow = rule.action != TopicRewriteActionEnum::All.to_string()
            || rule.action != TopicRewriteActionEnum::Publish.to_string();

        if !allow {
            continue;
        }

        if is_match_sub_and_topic(&rule.source_topic, name).is_ok() {
            new_topic_name = gen_rewrite_topic(name.to_string(), &rule.regex, &rule.dest_topic)?;
        }
    }

    if new_topic_name.is_empty() {
        return Ok(None);
    }
    info!(
        "topic rewriteconvert topic name: {} to {}",
        name, new_topic_name
    );
    Ok(Some(new_topic_name))
}

fn gen_rewrite_topic(
    input: String,
    pattern: &str,
    template: &str,
) -> Result<String, MqttBrokerError> {
    let prefix = String::new();
    let topic = decode_sub_path(&input);
    let re = Regex::new(pattern)?;
    let mut rewrite_topic = template.to_string();
    if let Some(captures) = re.captures(topic.as_str()) {
        for (i, capture) in captures.iter().skip(1).enumerate() {
            let prefix = format!("${}", (i + 1)).to_string();
            rewrite_topic = rewrite_topic
                .replace(&prefix, capture.unwrap().as_str())
                .clone();
        }
        return Ok(format!("{prefix}{rewrite_topic}"));
    }
    Ok(input.to_owned())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::common::tool::test_build_mqtt_cache_manager;

    use super::*;
    use common_base::tools::now_mills;
    use tokio::time::sleep;

    /// * Assume that the following topic rewrite rules have been added to the conf file:
    ///  ```bash
    ///  rewrite = [
    ///    {
    ///      action:       "all"
    ///      source_topic: "y/+/z/#"
    ///      dest_topic:   "y/z/$2"
    ///      re:           "^y/(.+)/z/(.+)$"
    ///    }
    ///    {
    ///      action:       "all"
    ///      source_topic: "x/#"
    ///      dest_topic:   "z/y/x/$1"
    ///      re:           "^x/y/(.+)$"
    ///    }
    ///    {
    ///      action:       "all"
    ///      source_topic: "x/y/+"
    ///      dest_topic:   "z/y/$1"
    ///      re:           "^x/y/(\d+)$"
    ///    }
    ///  ]
    ///  ```
    ///  * At this time we subscribe to five topics: `y/a/z/b`, `y/def`, `x/1/2`, `x/y/2`, and `x/y/z`:
    ///
    ///  * `y/def` does not match any of the topic filters, so it does not perform topic rewriting,
    ///    and just subscribes to `y/def` topics.
    ///
    ///  * `y/a/z/b` matches t   1he `y/+/z/#` topic filter, executes the first rule, and matches
    ///    the element \[a、b\] through a regular expression, brings the matched second element
    ///    into `y/z/$2`, and actually subscribes to the topic `y/z/b`.
    ///
    ///  * `x/1/2` matches `x/#` topic filter, executes the second rule. It does not match
    ///    elements through regular expressions, does not perform topic rewrite, and actually
    ///    subscribes to the topic of `x/1/2`.
    ///
    ///  * `x/y/2` matches two topic filters of `x/#` and `x/y/`+ at the same time, reads the
    ///    configuration in reverse order, so it matches the third preferentially. Through
    ///    regular replacement, it actually subscribed to the `z/y/2` topic.
    ///
    ///  * `x/y/z` matches two topic filters of `x/#` and `x/y/+` at the same time, reads the
    ///    configuration in reverse order, so it matches the third preferentially. The element is
    ///    not matched through the regular expression, the topic rewrite is not performed, and it
    ///    actually subscribes to the `x/y/z` topic. It should be noted that even if the regular
    ///    expression matching of the third fails, it will not match the rules of the second again.
    ///
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

    #[tokio::test]
    async fn gen_convert_rewrite_name_test() {
        let cache_manager = build_rules().await;
        for (index, src_topic) in SRC_TOPICS.iter().enumerate() {
            let result =
                convert_publish_topic_by_rewrite_rule(&cache_manager, src_topic.to_string());
            assert!(result.is_ok());
            if let Some(dst_topic) = result.unwrap() {
                println!("{src_topic:?}");
                assert_eq!(dst_topic, DST_TOPICS[index]);
            } else {
                assert_eq!(src_topic.to_owned(), DST_TOPICS[index]);
            }
        }
    }

    async fn build_rules() -> Arc<MQTTCacheManager> {
        let rules = [
            SimpleRule::new(r"y/+/z/#", r"y/z/$2", r"^y/(.+)/z/(.+)$"),
            SimpleRule::new(r"x/#", r"z/y/x/$1", r"^x/y/(.+)$"),
            SimpleRule::new(r"x/y/+", r"z/y/$1", r"^x/y/(\d+)$"),
        ];

        let cache_manager = test_build_mqtt_cache_manager();
        for rule in rules.iter() {
            let rule = MqttTopicRewriteRule {
                cluster: "default".to_string(),
                action: TopicRewriteActionEnum::All.to_string(),
                source_topic: rule.source.to_string(),
                dest_topic: rule.destination.to_string(),
                regex: rule.regex.to_string(),
                timestamp: now_mills(),
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
