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
use common_base::task::{TaskKind, TaskSupervisor};
use common_base::tools::now_second;
use common_base::uuid::unique_id;
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::nats::subscriber::NatsSubscriber;
use metadata_struct::topic::{Topic, TopicSource};
use std::collections::HashMap;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub enum ParseAction {
    Add,
    Remove,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubscribeSource {
    NatsCore,
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
                    register_subscriber(subscribe_manager, sub, &topic.topic_name, source);
                }
            }
        }
    }
    Ok(())
}

pub(crate) async fn parse_by_new_topic(
    subscribe_manager: &Arc<NatsSubscribeManager>,
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
                &sub,
                &topic.topic_name,
                &SubscribeSource::NatsCore,
            );
        }
    }
    Ok(())
}

/// Snapshots each shard's current end_offset for `topic_name` right now.
/// Returns an empty map if the topic doesn't exist yet — correct, since
/// nothing has been published yet, so "latest" is the start of an empty log.
async fn snapshot_topic_offsets(
    storage_driver_manager: &Arc<StorageDriverManager>,
    tenant: &str,
    topic_name: &str,
) -> HashMap<String, u64> {
    storage_driver_manager
        .list_storage_resource(tenant, topic_name)
        .await
        .map(|resources| {
            resources
                .into_values()
                .map(|detail| (detail.shard_name, detail.offset.end_offset))
                .collect()
        })
        .unwrap_or_default()
}

/// Snapshots offsets for every topic that *already exists* and matches
/// `subject_pattern`, at the moment a SUB is being processed (see
/// `process_sub` in nats/subscribe.rs) — this is the only point where "now"
/// reliably means "subscribe time", since everything downstream (raft
/// broadcast of the subscribe intent, matching it against topics, and the
/// fanout push loop noticing the resulting subscriber) is asynchronous and
/// can be arbitrarily delayed. A topic that doesn't exist yet simply won't
/// be in the returned map; `register_subscriber` treats that as "start from
/// the beginning once it's created", which is correct since nothing could
/// have been published to a topic that didn't exist yet.
pub(crate) async fn snapshot_known_topic_offsets(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    tenant: &str,
    subject_pattern: &str,
) -> HashMap<String, HashMap<String, u64>> {
    let topics: Vec<_> = cache_manager
        .node_cache
        .list_topics_by_tenant(tenant)
        .into_iter()
        .filter(|t| t.source == TopicSource::NATS)
        .filter(|t| nats_subject_match(subject_pattern, &t.topic_name))
        .collect();

    let mut result = HashMap::with_capacity(topics.len());
    for topic in topics {
        let offsets =
            snapshot_topic_offsets(storage_driver_manager, tenant, &topic.topic_name).await;
        result.insert(topic.topic_name, offsets);
    }
    result
}

fn register_subscriber(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    sub: &NatsSubscribe,
    topic_name: &str,
    source: &SubscribeSource,
) {
    let is_fanout = sub.queue_group.as_deref().unwrap_or("").is_empty();
    let initial_offsets = if !is_fanout {
        HashMap::new()
    } else if let Some(offsets) = sub.known_topic_offsets.get(topic_name) {
        // This topic already existed at subscribe time — use the snapshot
        // taken then, not whatever the tail is now that registration is
        // actually happening.
        offsets.clone()
    } else {
        // Wasn't known at subscribe time. Either it's a topic created after
        // subscribing (correct to start from empty/beginning), or this
        // subscribe intent arrived from a remote broker via replication
        // (queue_group aside, cross-node NatsCore fanout isn't currently
        // replicated this way, so this path is the "new topic" case in
        // practice). Falling back to a live snapshot would reintroduce the
        // original race, so we deliberately don't.
        HashMap::new()
    };

    let subscriber = NatsSubscriber {
        uniq_id: unique_id(),
        tenant: sub.tenant.clone(),
        connect_id: sub.connect_id,
        sid: sub.sid.clone(),
        sub_subject: sub.subject.clone(),
        subject: topic_name.to_string(),
        broker_id: sub.broker_id,
        queue_group: sub.queue_group.clone(),
        create_time: now_second(),
        initial_offsets,
    };

    match source {
        SubscribeSource::NatsCore => {
            if is_fanout {
                subscribe_manager.add_nats_core_fanout_subscriber(subscriber);
            } else {
                subscribe_manager.add_nats_core_queue_subscriber(&subscriber);
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

pub(crate) async fn start_subscribe_parse_thread(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    cache_manager: Arc<NatsCacheManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let (parse_tx, parse_rx) = tokio::sync::mpsc::channel(1024);
    subscribe_manager.set_parse_sender(parse_tx).await;

    let sm = subscribe_manager.clone();
    let sx = stop_sx.clone();
    task_supervisor.spawn(TaskKind::NATSSubscribeParse.to_string(), async move {
        start_parse_thread(cache_manager, sm, parse_rx, sx).await;
    });
}

async fn start_parse_thread(
    cache_manager: Arc<NatsCacheManager>,
    subscribe_manager: Arc<NatsSubscribeManager>,
    mut rx: Receiver<ParseSubscribeData>,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();

    loop {
        tokio::select! {
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

                match (&data.action, &data.source, &data.subscribe, &data.topic) {
                    (ParseAction::Add, source, Some(sub), None) => {
                        if let Err(e) = parse_by_new_subscribe(&cache_manager, &subscribe_manager, sub, source).await {
                            error!("{}", e.to_string());
                        }
                    }
                    (ParseAction::Remove, _, Some(sub), None) => {
                        subscribe_manager.remove_push_by_sub(sub.broker_id, sub.connect_id, &sub.sid);
                    }
                    (ParseAction::Add, _, None, Some(topic)) => {
                        if let Err(e) = parse_by_new_topic(&subscribe_manager, topic).await {
                            error!("{}", e.to_string());
                        }
                    }
                    (ParseAction::Remove, _, None, Some(topic)) => {
                        subscribe_manager.remove_fanout_by_subject(&topic.topic_name);
                    }
                    _ => {
                        error!("Unexpected ParseSubscribeData: {:?}", data);
                    }
                }
            }
        }
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
