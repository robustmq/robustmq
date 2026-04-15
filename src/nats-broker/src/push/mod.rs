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
use crate::push::mq9_fanout::Mq9FanoutPushManager;
use crate::push::nats_fanout::FanoutPushManager;
use crate::push::nats_queue::QueuePushManager;
use crate::push::parse::{ParseAction, ParseSubscribeData};
use common_base::task::{TaskKind, TaskSupervisor};
use common_base::uuid::unique_id;
pub use manager::NatsSubscribeManager;
use network_server::common::connection_manager::ConnectionManager;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::{broadcast, mpsc::Receiver};
use tokio::time::sleep;
use tracing::{debug, error, info};

pub mod buckets;
pub mod common;
pub mod manager;
pub mod mq9_fanout;
pub mod mq9_queue;
pub mod nats_fanout;
pub mod nats_queue;
pub mod parse;

async fn start_parse_thread(
    cache_manager: Arc<NatsCacheManager>,
    subscribe_manager: Arc<NatsSubscribeManager>,
    mut rx: Receiver<ParseSubscribeData>,
    stop_sx: broadcast::Sender<bool>,
) {
    use crate::push::parse::{parse_by_new_subscribe, parse_by_new_topic};
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
                        parse_by_new_subscribe(&cache_manager, &subscribe_manager, sub, source);
                    }
                    (ParseAction::Remove, _, Some(sub), None) => {
                        subscribe_manager.remove_push_by_sid(sub.connect_id, &sub.sid);
                    }
                    (ParseAction::Add, _, None, Some(topic)) => {
                        parse_by_new_topic(&subscribe_manager, topic);
                    }
                    (ParseAction::Remove, _, None, Some(topic)) => {
                        subscribe_manager.remove_by_topic(&topic.topic_name);
                    }
                    _ => {
                        error!("Unexpected ParseSubscribeData: {:?}", data);
                    }
                }
            }
        }
    }
}

pub async fn start_push(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    cache_manager: Arc<NatsCacheManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    task_supervisor: Arc<TaskSupervisor>,
    push_thread_num: usize,
    stop_sx: broadcast::Sender<bool>,
) {
    // parse thread
    let (parse_tx, parse_rx) = tokio::sync::mpsc::channel(1024);
    subscribe_manager.set_parse_sender(parse_tx).await;

    let sm = subscribe_manager.clone();
    let cm = cache_manager;
    let sx = stop_sx.clone();
    task_supervisor.spawn(TaskKind::NATSSubscribeParse.to_string(), async move {
        start_parse_thread(cm, sm, parse_rx, sx).await;
    });

    // nats core fanout push
    let bucket_ids: Vec<String> = (0..push_thread_num).map(|_| unique_id()).collect();
    for bucket_id in &bucket_ids {
        subscribe_manager
            .nats_core_fanout_push
            .register_bucket(bucket_id.clone());
    }
    for bucket_id in bucket_ids {
        let mgr = FanoutPushManager::new(
            subscribe_manager.clone(),
            connection_manager.clone(),
            storage_driver_manager.clone(),
            bucket_id.clone(),
        );
        let sx = stop_sx.clone();
        task_supervisor.spawn(
            format!("{}_{}", TaskKind::NATSSubscribePush, bucket_id),
            async move { mgr.start(&sx).await },
        );
    }

    // mq9 fanout push
    let mq9_bucket_ids: Vec<String> = (0..push_thread_num).map(|_| unique_id()).collect();
    for bucket_id in &mq9_bucket_ids {
        subscribe_manager
            .mq9_fanout_push
            .register_bucket(bucket_id.clone());
    }
    for bucket_id in mq9_bucket_ids {
        let mgr = Mq9FanoutPushManager::new(
            subscribe_manager.clone(),
            connection_manager.clone(),
            storage_driver_manager.clone(),
            bucket_id.clone(),
        );
        let sx = stop_sx.clone();
        task_supervisor.spawn(
            format!("{}_{}", TaskKind::MQ9SubscribePush, bucket_id),
            async move { mgr.start(&sx).await },
        );
    }

    // nats queue push
    let sm = subscribe_manager.clone();
    let conn = connection_manager.clone();
    let stor = storage_driver_manager.clone();
    let sup = task_supervisor.clone();
    let sx = stop_sx.clone();
    task_supervisor.spawn(TaskKind::NATSQueuePush.to_string(), async move {
        queue_group_watcher(sm, conn, stor, sup, sx).await;
    });
}

async fn queue_group_watcher(
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();
    let mut running_groups: HashSet<String> = HashSet::new();

    loop {
        tokio::select! {
            val = stop_rx.recv() => {
                match val {
                    Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                        info!("NATS queue group watcher stopped");
                        break;
                    }
                    _ => {}
                }
            }
            _ = sleep(Duration::from_millis(100)) => {
                let current_keys: HashSet<String> = subscribe_manager
                    .nats_core_queue_push
                    .iter()
                    .map(|e| e.key().clone())
                    .collect();

                for queue_key in &current_keys {
                    if running_groups.contains(queue_key) {
                        continue;
                    }
                    running_groups.insert(queue_key.clone());

                    let mut mgr = QueuePushManager::new(
                        subscribe_manager.clone(),
                        connection_manager.clone(),
                        storage_driver_manager.clone(),
                        queue_key.clone(),
                    );
                    let sx = stop_sx.clone();
                    task_supervisor.spawn(
                        format!("{}_{}", TaskKind::NATSQueuePush, queue_key),
                        async move { mgr.start(&sx).await },
                    );
                }

                running_groups.retain(|k| current_keys.contains(k));
            }
        }
    }
}
