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

use common_base::enum_type::topic_rewrite_action_enum::TopicRewriteActionEnum;
use dashmap::DashMap;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::mqtt::common::{Subscribe, Unsubscribe};

use crate::handler::error::MqttBrokerError;
use crate::handler::topic::gen_rewrite_topic;
use crate::subscribe::sub_common::path_regex_match;

pub fn process_sub_topic_rewrite(
    subscribe: &mut Subscribe,
    rules_map: &DashMap<String, MqttTopicRewriteRule>,
) {
    let mut rules: Vec<MqttTopicRewriteRule> = rules_map
        .iter()
        .map(|entry| entry.value().clone())
        .collect();
    rules.sort_by_key(|rule| rule.timestamp);
    for filter in subscribe.filters.iter_mut() {
        for topic_rewrite_rule in rules.iter().rev() {
            if topic_rewrite_rule.action != TopicRewriteActionEnum::All.to_string()
                && topic_rewrite_rule.action != TopicRewriteActionEnum::Subscribe.to_string()
            {
                continue;
            }
            // rewrite performed only for the first match
            if path_regex_match(&filter.path, &topic_rewrite_rule.source_topic) {
                if let Some(val) = gen_rewrite_topic(
                    &filter.path,
                    &topic_rewrite_rule.regex,
                    &topic_rewrite_rule.dest_topic,
                ) {
                    filter.path = val;
                }
                break;
            }
        }
    }
}

pub fn process_unsub_topic_rewrite(
    un_subscribe: &mut Unsubscribe,
    rules_map: &DashMap<String, MqttTopicRewriteRule>,
) {
    let mut rules: Vec<MqttTopicRewriteRule> = rules_map
        .iter()
        .map(|entry| entry.value().clone())
        .collect();
    rules.sort_by_key(|rule| rule.timestamp);
    for filter in un_subscribe.filters.iter_mut() {
        for topic_rewrite_rule in rules.iter().rev() {
            if topic_rewrite_rule.action != TopicRewriteActionEnum::All.to_string()
                && topic_rewrite_rule.action != TopicRewriteActionEnum::Subscribe.to_string()
            {
                continue;
            }
            // rewrite performed only for the first match
            if path_regex_match(filter, &topic_rewrite_rule.source_topic) {
                if let Some(val) = gen_rewrite_topic(
                    filter,
                    &topic_rewrite_rule.regex,
                    &topic_rewrite_rule.dest_topic,
                ) {
                    *filter = val;
                }
                break;
            }
        }
    }
}

pub fn process_publish_topic_rewrite(
    topic_name: String,
    rules_map: &DashMap<String, MqttTopicRewriteRule>,
) -> Result<Option<String>, MqttBrokerError> {
    let mut rules: Vec<MqttTopicRewriteRule> = rules_map
        .iter()
        .map(|entry| entry.value().clone())
        .collect();
    rules.sort_by_key(|rule| rule.timestamp);
    for topic_rewrite_rule in rules.iter().rev() {
        if topic_rewrite_rule.action != TopicRewriteActionEnum::All.to_string()
            && topic_rewrite_rule.action != TopicRewriteActionEnum::Publish.to_string()
        {
            continue;
        }
        if path_regex_match(&topic_name, &topic_rewrite_rule.source_topic) {
            let rewrite_topic = gen_rewrite_topic(
                &topic_name,
                &topic_rewrite_rule.regex,
                &topic_rewrite_rule.dest_topic,
            );

            return match rewrite_topic {
                Some(rewrite_topic) => Ok(Some(rewrite_topic)),
                None => Ok(Some(topic_name)),
            };
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools;
    use protocol::mqtt::common::{Filter, QoS, RetainForwardRule, Subscribe};

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
    const REGEX_MATCHES: [bool; 5] = [true, false, true, true, true];

    #[test]
    fn sub_topic_rewrite_test() {
        let filters = SRC_TOPICS.map(|src_topic| Filter {
            path: src_topic.to_string(),
            qos: QoS::AtMostOnce,
            nolocal: false,
            preserve_retain: false,
            retain_forward_rule: RetainForwardRule::Never,
        });
        let mut subscribe = Subscribe {
            packet_identifier: 0,
            filters: filters.to_vec(),
        };
        let rules = build_rules();
        process_sub_topic_rewrite(&mut subscribe, &rules);

        // verify rewrote topics
        for (index, filter) in subscribe.filters.iter().enumerate() {
            assert_eq!(filter.path, DST_TOPICS[index]);
        }
    }

    #[test]
    fn unsub_topic_rewrite_test() {
        let filters = SRC_TOPICS.map(|src_topic| src_topic.to_string()).to_vec();
        let mut unsub = Unsubscribe { pkid: 0, filters };

        let rules = build_rules();
        process_unsub_topic_rewrite(&mut unsub, &rules);

        // verify rewrote topics
        for (index, filter) in unsub.filters.iter().enumerate() {
            assert_eq!(filter, DST_TOPICS[index]);
        }
    }

    #[test]
    fn publish_topic_rewrite_test() {
        let rules = build_rules();
        for (index, src_topic) in SRC_TOPICS.iter().enumerate() {
            let result = process_publish_topic_rewrite(src_topic.to_string(), &rules);
            assert!(result.is_ok());
            let option = result.unwrap();

            let matches = REGEX_MATCHES[index];
            assert_eq!(option.is_some(), matches);
            if matches {
                let dst_topic = option.unwrap();
                assert_eq!(dst_topic, DST_TOPICS[index]);
            } else {
                assert!(option.is_none());
            }
        }
    }

    fn build_rules() -> DashMap<String, MqttTopicRewriteRule> {
        let rules = vec![
            SimpleRule::new(r"y/+/z/#", r"y/z/$2", r"^y/(.+)/z/(.+)$"),
            SimpleRule::new(r"x/#", r"z/y/x/$1", r"^x/y/(.+)$"),
            SimpleRule::new(r"x/y/+", r"z/y/$1", r"^x/y/(\d+)$"),
        ];

        let map = DashMap::with_capacity(rules.len());
        for (index, rule) in rules.iter().enumerate() {
            let rule = MqttTopicRewriteRule {
                cluster: "default".to_string(),
                action: TopicRewriteActionEnum::All.to_string(),
                source_topic: rule.source.to_string(),
                dest_topic: rule.destination.to_string(),
                regex: rule.regex.to_string(),
                timestamp: tools::now_nanos(),
            };
            map.insert(format!("rule{}", index), rule);
        }
        map
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
