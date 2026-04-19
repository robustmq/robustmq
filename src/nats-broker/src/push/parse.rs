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
use crate::core::error::NatsBrokerError;
use crate::push::manager::NatsSubscribeManager;
use crate::push::queue::add_member_by_group;
use common_base::tools::now_second;
use common_base::uuid::unique_id;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::nats::subscriber::NatsSubscriber;
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

pub(crate) async fn parse_by_new_subscribe(
    cache_manager: &Arc<NatsCacheManager>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    client_pool: Arc<ClientPool>,
    sub: &NatsSubscribe,
    source: &SubscribeSource,
) -> Result<(), NatsBrokerError> {
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
                    register_subscriber(
                        subscribe_manager,
                        client_pool.clone(),
                        sub,
                        &topic.topic_name,
                        source,
                    )
                    .await?;
                }
            }
        }
        SubscribeSource::Mq9 => {
            register_subscriber(subscribe_manager, client_pool, sub, &sub.subject, source).await?;
        }
    }
    Ok(())
}

pub(crate) async fn parse_by_new_topic(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    client_pool: Arc<ClientPool>,
    topic: &Topic,
) -> Result<(), NatsBrokerError> {
    if topic.source != TopicSource::NATS {
        return Ok(());
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
                client_pool.clone(),
                &sub,
                &topic.topic_name,
                &SubscribeSource::NatsCore,
            )
            .await?;
        }
    }
    Ok(())
}

async fn register_subscriber(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    client_pool: Arc<ClientPool>,
    sub: &NatsSubscribe,
    topic_name: &str,
    source: &SubscribeSource,
) -> Result<(), NatsBrokerError> {
    let conf = broker_config();
    let subscriber = NatsSubscriber {
        uniq_id: unique_id(),
        tenant: sub.tenant.clone(),
        connect_id: sub.connect_id,
        sid: sub.sid.clone(),
        sub_subject: sub.subject.clone(),
        subject: topic_name.to_string(),
        broker_id: conf.broker_id,
        queue_group: sub.queue_group.clone(),
        create_time: now_second(),
    };

    match source {
        SubscribeSource::NatsCore => {
            if sub.queue_group.is_empty() {
                subscribe_manager.add_nats_core_fanout_subscriber(subscriber);
            } else {
                add_member_by_group(client_pool, &subscriber).await?;
            }
        }
        SubscribeSource::Mq9 => {
            if sub.queue_group.is_empty() {
                subscribe_manager.add_mq9_fanout_subscriber(subscriber);
            } else {
                add_member_by_group(client_pool, &subscriber).await?;
            }
        }
    }
    Ok(())
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
    fn test_nats_subject_match() {
        // exact
        assert!(nats_subject_match("foo.bar", "foo.bar"));
        assert!(!nats_subject_match("foo.bar", "foo.baz"));
        assert!(!nats_subject_match("foo.bar", "foo"));
        assert!(!nats_subject_match("foo", "foo.bar"));

        // * wildcard (one token)
        assert!(nats_subject_match("foo.*", "foo.bar"));
        assert!(!nats_subject_match("foo.*", "foo.bar.baz"));
        assert!(nats_subject_match("*.bar", "foo.bar"));
        assert!(nats_subject_match("foo.*.baz", "foo.bar.baz"));
        assert!(!nats_subject_match("foo.*.baz", "foo.bar.qux"));

        // > wildcard (one or more trailing tokens)
        assert!(nats_subject_match("foo.>", "foo.bar"));
        assert!(nats_subject_match("foo.>", "foo.bar.baz"));
        assert!(!nats_subject_match("foo.>", "foo"));
        assert!(nats_subject_match(">", "foo.bar.baz"));

        // combined
        assert!(nats_subject_match("foo.*.>", "foo.bar.baz"));
        assert!(nats_subject_match("foo.*.>", "foo.bar.baz.qux"));
        assert!(!nats_subject_match("foo.*.>", "foo.bar"));

        // edge cases
        assert!(!nats_subject_match("foo.bar", "bar.foo"));
        assert!(!nats_subject_match("", "foo"));
        assert!(!nats_subject_match("foo", ""));
        assert!(nats_subject_match("", ""));
    }
}
