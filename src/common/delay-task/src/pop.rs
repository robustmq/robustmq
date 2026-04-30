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

use crate::delay::delete_delay_task_index;
use crate::handler::lastwill_expire::handle_lastwill_expire;
use crate::handler::session_expire::handle_session_expire;
use crate::manager::{DelayTaskManager, ShardCmd};
use crate::{DelayTask, DelayTaskData};
use broker_core::cache::NodeCacheManager;
use common_base::error::common::CommonError;
use common_base::task::{TaskKind, TaskSupervisor};
use common_base::tools::now_second;
use common_metrics::mqtt::delay_task::{
    record_delay_task_execute_failed, record_delay_task_executed,
    record_delay_task_schedule_latency,
};
use futures::StreamExt;
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::{select, sync::broadcast as bc};
use tokio_util::time::DelayQueue;
use tracing::{debug, error, info, warn};

pub(crate) fn spawn_delay_task_pop_threads(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    delay_task_manager: &Arc<DelayTaskManager>,
    node_call_manager: &Arc<NodeCallManager>,
    broker_cache: &Arc<NodeCacheManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    delay_queue_num: u32,
) {
    info!("Starting delay task pop threads ({})", delay_queue_num);

    for shard_no in 0..delay_queue_num {
        let manager = delay_task_manager.clone();

        // Create command channel for this shard.
        let (tx, rx) = mpsc::unbounded_channel::<ShardCmd>();
        delay_task_manager.register_shard_cmd_tx(shard_no, tx);

        let (stop_send, _) = broadcast::channel(2);
        delay_task_manager.add_delay_queue_pop_thread(shard_no, stop_send.clone());

        let raw_rocksdb_engine_handler = rocksdb_engine_handler.clone();
        let raw_node_call_manager = node_call_manager.clone();
        let raw_broker_cache = broker_cache.clone();

        task_supervisor.spawn(
            format!("{}_{}", TaskKind::DelayTaskPop, shard_no),
            async move {
                run_shard_loop(
                    shard_no,
                    rx,
                    stop_send,
                    manager,
                    raw_rocksdb_engine_handler,
                    raw_node_call_manager,
                    raw_broker_cache,
                )
                .await;
            },
        );
    }
}

/// Per-shard event loop.
/// Owns the DelayQueue exclusively — no Mutex needed.
/// Uses select! to react to either:
///   - a command from the manager (Insert / Delete)
///   - a task expiring in the DelayQueue
async fn run_shard_loop(
    shard_no: u32,
    mut rx: mpsc::UnboundedReceiver<ShardCmd>,
    stop_send: bc::Sender<bool>,
    manager: Arc<DelayTaskManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    node_call_manager: Arc<NodeCallManager>,
    broker_cache: Arc<NodeCacheManager>,
) {
    let mut delay_queue: DelayQueue<DelayTask> = DelayQueue::new();
    let mut stop_recv = stop_send.subscribe();

    loop {
        select! {
            // Stop signal
            val = stop_recv.recv() => {
                match val {
                    Ok(flag) if flag => {
                        info!("Delay task pop thread stopped for shard {}", shard_no);
                        break;
                    }
                    Err(_) => {
                        warn!("Broadcast channel closed, stopping pop thread for shard {}", shard_no);
                        break;
                    }
                    _ => {}
                }
            }

            // Command from manager (Insert or Delete)
            cmd = rx.recv() => {
                match cmd {
                    Some(ShardCmd::Insert(task, target_instant, key_tx)) => {
                        let key = delay_queue.insert_at(task.clone(), target_instant);
                        // Reply with the key so manager can record it in task_key_map.
                        let _ = key_tx.send(key);
                    }
                    Some(ShardCmd::Delete(key, done_tx)) => {
                        delay_queue.remove(&key);
                        // Notify manager that the entry is gone from the queue.
                        let _ = done_tx.send(());
                    }
                    None => {
                        // Channel closed — manager dropped, exit.
                        break;
                    }
                }
            }

            // Expired task
            Some(expired) = delay_queue.next() => {
                let task = expired.into_inner();
                spawn_task_process(
                    rocksdb_engine_handler.clone(),
                    manager.clone(),
                    node_call_manager.clone(),
                    broker_cache.clone(),
                    task,
                )
                .await;
            }
        }
    }
}

pub(crate) async fn spawn_task_process(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    delay_task_manager: Arc<DelayTaskManager>,
    node_call_manager: Arc<NodeCallManager>,
    broker_cache: Arc<NodeCacheManager>,
    task: DelayTask,
) {
    let permit = match delay_task_manager
        .handler_semaphore
        .clone()
        .acquire_owned()
        .await
    {
        Ok(permit) => permit,
        Err(e) => {
            error!(
                "Failed to acquire delay task handler permit: task_id={}, error={}",
                task.task_id, e
            );
            return;
        }
    };

    tokio::spawn(async move {
        let _permit = permit;
        let task_type_str = task.task_type_name();
        if let Err(e) = delay_task_process(
            &delay_task_manager,
            &node_call_manager,
            &rocksdb_engine_handler,
            &broker_cache,
            &task,
        )
        .await
        {
            record_delay_task_execute_failed(task_type_str);
            let err_str = e.to_string();
            if err_str.contains("channel closed") || err_str.contains("channel full") {
                warn!(
                    "Delay task skipped (broker shutting down): task_id={}, task_type={}, error={}",
                    task.task_id, task_type_str, e
                );
            } else {
                error!(
                    "Failed to process delay task: task_id={}, task_type={}, error={}",
                    task.task_id, task_type_str, e
                );
            }
        }
    });
}

pub async fn delay_task_process(
    delay_task_manager: &Arc<DelayTaskManager>,
    node_call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    broker_cache: &Arc<NodeCacheManager>,
    task: &DelayTask,
) -> Result<(), CommonError> {
    let task_type_str = task.task_type_name();
    debug!(
        "Processing delay task: task_id={}, task_type={}",
        task.task_id, task_type_str
    );

    delay_task_manager.remove_task_key(&task.task_id);

    let latency_s = now_second().saturating_sub(task.delay_target_time) as f64;
    record_delay_task_schedule_latency(task_type_str, latency_s);

    match &task.data {
        DelayTaskData::MQTTSessionExpire(tenant, client_id) => {
            handle_session_expire(
                node_call_manager,
                rocksdb_engine_handler,
                broker_cache,
                delay_task_manager,
                tenant,
                client_id,
            )
            .await?;
        }
        DelayTaskData::MQTTLastwillExpire(client_id) => {
            handle_lastwill_expire(node_call_manager, client_id).await?;
        }
    }

    if task.persistent {
        delete_delay_task_index(&delay_task_manager.storage_driver_manager, &task.task_id).await?;
    }

    record_delay_task_executed(task_type_str);
    Ok(())
}
