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

use crate::core::cache::NatsCacheManager;
use crate::push::manager::NatsSubscribeManager;
use crate::push::manager::NatsSubscriber;
use common_base::tools::now_second;
use common_base::uuid::unique_id;
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::topic::{Topic, TopicSource};
use std::sync::Arc;
use tracing::debug;

#[derive(Clone, Debug)]
pub enum ParseAction {
    Add,
    Remove,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubscribeSource {
    NatsCore,
    Mq9,
}

#[derive(Clone, Debug)]
pub struct ParseSubscribeData {
    pub action: ParseAction,
    pub source: SubscribeSource,
    pub subscribe: Option<NatsSubscribe>,
    pub topic: Option<Topic>,
}

impl ParseSubscribeData {
    pub fn new_subscribe(
        action: ParseAction,
        source: SubscribeSource,
        subscribe: NatsSubscribe,
    ) -> Self {
        ParseSubscribeData {
            action,
            source,
            subscribe: Some(subscribe),
            topic: None,
        }
    }

    pub fn new_topic(action: ParseAction, topic: Topic) -> Self {
        ParseSubscribeData {
            action,
            source: SubscribeSource::NatsCore,
            subscribe: None,
            topic: Some(topic),
        }
    }
}

pub(crate) fn parse_by_new_subscribe(
    cache_manager: &Arc<NatsCacheManager>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    sub: &NatsSubscribe,
    source: &SubscribeSource,
) {
    debug!(
        "Matching new subscribe: connect_id={}, sid={}, subject={}, source={:?}",
        sub.connect_id, sub.sid, sub.subject, source
    );

    match source {
        SubscribeSource::NatsCore => {
            let topics: Vec<_> = cache_manager
                .node_cache
                .list_topics_by_tenant(&sub.tenant)
                .into_iter()
                .filter(|t| t.source == TopicSource::NATS)
                .collect();

            for topic in topics {
                if nats_subject_match(&sub.subject, &topic.topic_name) {
                    register_subscriber(subscribe_manager, sub, &topic.topic_name, source);
                }
            }
        }
        SubscribeSource::Mq9 => {
            register_subscriber(subscribe_manager, sub, &sub.subject, source);
        }
    }
}

pub(crate) fn parse_by_new_topic(subscribe_manager: &Arc<NatsSubscribeManager>, topic: &Topic) {
    if topic.source != TopicSource::NATS {
        return;
    }
    debug!("Matching new topic: {}", topic.topic_name);

    let subscribes: Vec<_> = subscribe_manager
        .subscribe_list
        .iter()
        .map(|e| e.value().clone())
        .collect();

    for sub in subscribes {
        if nats_subject_match(&sub.subject, &topic.topic_name) {
            register_subscriber(
                subscribe_manager,
                &sub,
                &topic.topic_name,
                &SubscribeSource::NatsCore,
            );
        }
    }
}

fn register_subscriber(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    sub: &NatsSubscribe,
    topic_name: &str,
    source: &SubscribeSource,
) {
    let subscriber = NatsSubscriber {
        uniq_id: unique_id(),
        tenant: sub.tenant.clone(),
        connect_id: sub.connect_id,
        sid: sub.sid.clone(),
        sub_subject: sub.subject.clone(),
        subject: topic_name.to_string(),
        queue_group: sub.queue_group.clone(),
        priority: sub.priority.clone(),
        create_time: now_second(),
    };

    match source {
        SubscribeSource::NatsCore => {
            if sub.queue_group.is_empty() {
                subscribe_manager.add_nats_core_fanout_subscriber(subscriber);
            } else {
                subscribe_manager.add_nats_core_queue_subscriber(subscriber, &sub.queue_group);
            }
        }
        SubscribeSource::Mq9 => {
            if sub.queue_group.is_empty() {
                subscribe_manager.add_mq9_fanout_subscriber(subscriber);
            } else {
                subscribe_manager.add_mq9_queue_subscriber(subscriber, &sub.queue_group);
            }
        }
    }
}

pub fn nats_subject_match(pattern: &str, topic: &str) -> bool {
    let pat: Vec<&str> = pattern.split('.').collect();
    let top: Vec<&str> = topic.split('.').collect();
    match_tokens(&pat, &top)
}

fn match_tokens(pat: &[&str], top: &[&str]) -> bool {
    match (pat.first(), top.first()) {
        (None, None) => true,
        (Some(&">"), _) => !top.is_empty(),
        (None, _) | (_, None) => false,
        (Some(&"*"), _) => match_tokens(&pat[1..], &top[1..]),
        (Some(p), Some(t)) => p == t && match_tokens(&pat[1..], &top[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(nats_subject_match("foo.bar", "foo.bar"));
        assert!(!nats_subject_match("foo.bar", "foo.baz"));
        assert!(!nats_subject_match("foo.bar", "foo"));
        assert!(!nats_subject_match("foo", "foo.bar"));
    }

    #[test]
    fn test_star_wildcard() {
        assert!(nats_subject_match("foo.*", "foo.bar"));
        assert!(!nats_subject_match("foo.*", "foo.bar.baz"));
        assert!(!nats_subject_match("foo.*", "foo"));
        assert!(nats_subject_match("*.bar", "foo.bar"));
        assert!(nats_subject_match("foo.*.baz", "foo.bar.baz"));
        assert!(!nats_subject_match("foo.*.baz", "foo.bar.qux"));
    }

    #[test]
    fn test_gt_wildcard() {
        assert!(nats_subject_match("foo.>", "foo.bar"));
        assert!(nats_subject_match("foo.>", "foo.bar.baz"));
        assert!(nats_subject_match("foo.>", "foo.a.b.c.d"));
        assert!(!nats_subject_match("foo.>", "foo"));
        assert!(nats_subject_match(">", "foo"));
        assert!(nats_subject_match(">", "foo.bar.baz"));
    }

    #[test]
    fn test_combined_wildcards() {
        assert!(nats_subject_match("foo.*.>", "foo.bar.baz"));
        assert!(nats_subject_match("foo.*.>", "foo.bar.baz.qux"));
        assert!(!nats_subject_match("foo.*.>", "foo.bar"));
    }

    #[test]
    fn test_no_match() {
        assert!(!nats_subject_match("foo.bar", "bar.foo"));
        assert!(!nats_subject_match("", "foo"));
        assert!(!nats_subject_match("foo", ""));
        assert!(nats_subject_match("", ""));
    }
}
