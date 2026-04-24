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
use std::sync::Arc;
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
                    register_subscriber(subscribe_manager, sub, &topic.topic_name, source).await?;
                }
            }
        }
        SubscribeSource::Mq9 => {
            register_subscriber(subscribe_manager, sub, &sub.subject, source).await?;
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
            )
            .await?;
        }
    }
    Ok(())
}

async fn register_subscriber(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    sub: &NatsSubscribe,
    topic_name: &str,
    source: &SubscribeSource,
) -> Result<(), NatsBrokerError> {
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
    };

    match source {
        SubscribeSource::NatsCore => {
            if sub.queue_group.is_empty() {
                subscribe_manager.add_nats_core_fanout_subscriber(subscriber);
            } else {
                subscribe_manager.add_nats_core_queue_subscriber(&subscriber);
            }
        }
        SubscribeSource::Mq9 => {
            if sub.queue_group.is_empty() {
                subscribe_manager.add_mq9_fanout_subscriber(subscriber);
            } else {
                subscribe_manager.add_mq9_queue_subscriber(&subscriber);
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
                        subscribe_manager.remove_push_by_sub(sub.connect_id, &sub.sid);
                    }
                    (ParseAction::Add, _, None, Some(topic)) => {
                        if let Err(e) = parse_by_new_topic(&subscribe_manager, topic).await {
                            error!("{}", e.to_string());
                        }
                    }
                    (ParseAction::Remove, _, None, Some(topic)) => {
                        subscribe_manager.remove_by_subject(&topic.topic_name);
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
