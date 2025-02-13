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
use crate::subscribe::sub_common::validate_wildcard_topic_subscription;

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
            if validate_wildcard_topic_subscription(&filter.path, &topic_rewrite_rule.source_topic)
            {
                if let Some(val) = gen_rewrite_topic(
                    &filter.path,
                    &topic_rewrite_rule.regex,
                    &topic_rewrite_rule.dest_topic,
                ) {
                    filter.path = val;
                }
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
            if validate_wildcard_topic_subscription(filter, &topic_rewrite_rule.source_topic) {
                if let Some(val) = gen_rewrite_topic(
                    filter,
                    &topic_rewrite_rule.regex,
                    &topic_rewrite_rule.dest_topic,
                ) {
                    *filter = val;
                }
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
        if validate_wildcard_topic_subscription(&topic_name, &topic_rewrite_rule.source_topic) {
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
