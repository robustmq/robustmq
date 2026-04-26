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

use crate::push::manager::QueuePushThreadInfo;
use crate::push::mq9_fanout::Mq9FanoutPushManager;
use crate::push::nats_fanout::FanoutPushManager;
use crate::push::nats_queue::{QueuePushManager, QueuePushManagerParams};
use crate::push::NatsSubscribeManager;
use broker_core::cache::NodeCacheManager;
use common_base::error::ResultCommonError;
use common_base::task::{TaskKind, TaskSupervisor};
use common_base::tools::loop_select_ticket;
use common_base::uuid::unique_id;
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::{info, warn};

pub(crate) struct SubPushThreadParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub node_cache: Arc<NodeCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub push_thread_num: usize,
    pub stop_sx: broadcast::Sender<bool>,
}

pub(crate) async fn start_sub_push_thread(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    p: SubPushThreadParams,
) {
    start_nats_core_fanout_push_threads(
        subscribe_manager,
        &p.connection_manager,
        &p.storage_driver_manager,
        &p.task_supervisor,
        p.push_thread_num,
        &p.stop_sx,
    );

    start_mq9_fanout_push_threads(
        subscribe_manager,
        &p.connection_manager,
        &p.storage_driver_manager,
        &p.task_supervisor,
        p.push_thread_num,
        &p.stop_sx,
    );

    start_nats_queue_push_watcher(
        subscribe_manager,
        p.connection_manager,
        p.storage_driver_manager,
        p.node_cache,
        p.client_pool,
        &p.task_supervisor,
        p.stop_sx,
    );
}

fn start_nats_core_fanout_push_threads(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    connection_manager: &Arc<ConnectionManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    push_thread_num: usize,
    stop_sx: &broadcast::Sender<bool>,
) {
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
}

fn start_mq9_fanout_push_threads(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    connection_manager: &Arc<ConnectionManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    push_thread_num: usize,
    stop_sx: &broadcast::Sender<bool>,
) {
    let bucket_ids: Vec<String> = (0..push_thread_num).map(|_| unique_id()).collect();
    for bucket_id in &bucket_ids {
        subscribe_manager
            .mq9_fanout_push
            .register_bucket(bucket_id.clone());
    }
    for bucket_id in bucket_ids {
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
}

fn start_nats_queue_push_watcher(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    node_cache: Arc<NodeCacheManager>,
    client_pool: Arc<ClientPool>,
    task_supervisor: &Arc<TaskSupervisor>,
    stop_sx: broadcast::Sender<bool>,
) {
    let sm = subscribe_manager.clone();
    let sup = task_supervisor.clone();
    task_supervisor.spawn(TaskKind::NATSQueuePush.to_string(), async move {
        nats_core_queue_push_thread(
            sm,
            connection_manager,
            storage_driver_manager,
            node_cache,
            client_pool,
            sup,
            stop_sx,
        )
        .await;
    });
}

async fn nats_core_queue_push_thread(
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    node_cache: Arc<NodeCacheManager>,
    client_pool: Arc<ClientPool>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_sx: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        stop_empty_queue_group_tasks(&subscribe_manager);
        start_new_queue_group_tasks(
            &subscribe_manager,
            &connection_manager,
            &storage_driver_manager,
            &node_cache,
            &client_pool,
            &task_supervisor,
        );
        Ok(())
    };
    loop_select_ticket(ac_fn, 100, &stop_sx).await;
}

fn stop_empty_queue_group_tasks(subscribe_manager: &Arc<NatsSubscribeManager>) {
    let empty_keys: Vec<String> = subscribe_manager
        .nats_core_queue_push_thread
        .iter()
        .filter(|e| !subscribe_manager.nats_core_queue_push.contains_key(e.key()))
        .map(|e| e.key().clone())
        .collect();

    for queue_key in empty_keys {
        if let Some((_, info)) = subscribe_manager
            .nats_core_queue_push_thread
            .remove(&queue_key)
        {
            if let Err(e) = info.stop_tx.send(true) {
                warn!(
                    "Failed to send stop signal to queue group task [{}]: {}",
                    queue_key, e
                );
            }
            info!("NATS queue group task stopped: {}", queue_key);
        }
    }
}

fn start_new_queue_group_tasks(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    connection_manager: &Arc<ConnectionManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    node_cache: &Arc<NodeCacheManager>,
    client_pool: &Arc<ClientPool>,
    task_supervisor: &Arc<TaskSupervisor>,
) {
    for entry in subscribe_manager.nats_core_queue_push.iter() {
        let queue_key = entry.key().clone();
        if entry.value().buckets_data_list.is_empty() {
            continue;
        }
        if subscribe_manager
            .nats_core_queue_push_thread
            .contains_key(&queue_key)
        {
            continue;
        }

        let mut parts = queue_key.splitn(3, '#');
        let tenant = parts.next().unwrap_or("").to_string();
        let group_name = parts.next().unwrap_or("").to_string();
        let subject = parts.next().unwrap_or("").to_string();

        let (task_stop_sx, _) = broadcast::channel(1);
        let thread_info = Arc::new(QueuePushThreadInfo::new(task_stop_sx.clone()));
        subscribe_manager
            .nats_core_queue_push_thread
            .insert(queue_key.clone(), thread_info.clone());

        let mut mgr = QueuePushManager::new(QueuePushManagerParams {
            subscribe_manager: subscribe_manager.clone(),
            connection_manager: connection_manager.clone(),
            storage_driver_manager: storage_driver_manager.clone(),
            node_cache: node_cache.clone(),
            client_pool: client_pool.clone(),
            tenant,
            group_name,
            subject,
        });
        task_supervisor.spawn(
            format!("{}_{}", TaskKind::NATSQueuePush, queue_key),
            async move { mgr.start(&task_stop_sx).await },
        );
        info!("NATS queue group task started: {}", queue_key);
    }
}
