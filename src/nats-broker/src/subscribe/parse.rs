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
use crate::subscribe::{directly_group_name, NatsSubscribeManager, NatsSubscriber};
use common_base::tools::now_second;
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::topic::{Topic, TopicSource};
use std::sync::Arc;
use tokio::{
    select,
    sync::{broadcast, mpsc::Receiver},
};
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub enum ParseAction {
    Add,
    Remove,
}

#[derive(Clone, Debug)]
pub struct ParseSubscribeData {
    pub action: ParseAction,
    pub subscribe: Option<NatsSubscribe>,
    pub topic: Option<Topic>,
}

impl ParseSubscribeData {
    pub fn new_subscribe(action: ParseAction, subscribe: NatsSubscribe) -> Self {
        ParseSubscribeData {
            action,
            subscribe: Some(subscribe),
            topic: None,
        }
    }

    pub fn new_topic(action: ParseAction, topic: Topic) -> Self {
        ParseSubscribeData {
            action,
            subscribe: None,
            topic: Some(topic),
        }
    }
}

pub async fn start_parse_thread(
    cache_manager: Arc<NatsCacheManager>,
    subscribe_manager: Arc<NatsSubscribeManager>,
    mut rx: Receiver<ParseSubscribeData>,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();

    loop {
        select! {
            val = stop_rx.recv() => {
                match val {
                    Ok(true) => {
                        info!("NATS subscribe parse thread stopping");
                        break;
                    }
                    Ok(false) => {}
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("NATS subscribe parse thread stop channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!("NATS subscribe parse thread stop channel lagged, skipped {}", n);
                    }
                }
            }

            result = rx.recv() => {
                let Some(data) = result else {
                    info!("NATS subscribe parse thread request channel closed");
                    break;
                };

                match (&data.action, &data.subscribe, &data.topic) {
                    (ParseAction::Add, Some(sub), None) => {
                        parse_by_new_subscribe(&cache_manager, &subscribe_manager, sub);
                    }
                    (ParseAction::Remove, Some(sub), None) => {
                        subscribe_manager.remove_subscribe(sub.connect_id, &sub.sid);
                    }
                    (ParseAction::Add, None, Some(topic)) => {
                        parse_by_new_topic(&subscribe_manager, topic);
                    }
                    (ParseAction::Remove, None, Some(topic)) => {
                        remove_by_topic(&subscribe_manager, &topic.topic_name);
                    }
                    _ => {
                        error!("Unexpected ParseSubscribeData: {:?}", data);
                    }
                }
            }
        }
    }
}

fn parse_by_new_subscribe(
    cache_manager: &Arc<NatsCacheManager>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    sub: &NatsSubscribe,
) {
    debug!(
        "Matching new subscribe: connect_id={}, sid={}, subject={}",
        sub.connect_id, sub.sid, sub.subject
    );

    let topics: Vec<_> = cache_manager
        .node_cache
        .list_topics_by_tenant(&sub.tenant)
        .into_iter()
        .filter(|t| t.source == TopicSource::NATS)
        .collect();

    for topic in topics {
        if nats_subject_match(&sub.subject, &topic.topic_name) {
            register_subscriber(subscribe_manager, sub, &topic.topic_name);
        }
    }
}

fn parse_by_new_topic(subscribe_manager: &Arc<NatsSubscribeManager>, topic: &Topic) {
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
            register_subscriber(subscribe_manager, &sub, &topic.topic_name);
        }
    }
}

fn remove_by_topic(subscribe_manager: &Arc<NatsSubscribeManager>, topic_name: &str) {
    subscribe_manager.directly_push.remove(topic_name);
    subscribe_manager
        .queue_push
        .retain(|key, _| !key.starts_with(&format!("{}#", topic_name)));
}

fn register_subscriber(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    sub: &NatsSubscribe,
    topic_name: &str,
) {
    let subscriber = NatsSubscriber {
        connect_id: sub.connect_id,
        sid: sub.sid.clone(),
        sub_subject: sub.subject.clone(),
        topic_name: topic_name.to_string(),
        group_name: directly_group_name(sub.connect_id, &sub.sid, topic_name),
        create_time: now_second(),
    };

    if sub.queue_group.is_empty() {
        subscribe_manager.add_directly_subscriber(subscriber);
    } else {
        subscribe_manager.add_queue_subscriber(subscriber, &sub.queue_group);
    }
}

// `*` matches exactly one dot-separated token; `>` matches one or more and must be last.
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
