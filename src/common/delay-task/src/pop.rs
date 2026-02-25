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
use crate::handler::{handle_lastwill_expire, handle_session_expire};
use crate::manager::{DelayTaskManager, SharedDelayQueue};
use crate::{DelayTask, DelayTaskType};
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_metrics::mqtt::delay_task::{
    record_delay_task_execute_failed, record_delay_task_executed,
    record_delay_task_schedule_latency,
};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio::{select, sync::broadcast};
use tracing::{debug, error, info, warn};

const POP_LOCK_TIMEOUT_MS: u64 = 100;

pub(crate) fn spawn_delay_task_pop_threads(
    delay_task_manager: &Arc<DelayTaskManager>,
    delay_queue_num: u32,
) {
    info!("Starting delay task pop threads ({})", delay_queue_num);

    for shard_no in 0..delay_queue_num {
        let manager = delay_task_manager.clone();

        let (stop_send, _) = broadcast::channel(2);
        delay_task_manager.add_delay_queue_pop_thread(shard_no, stop_send.clone());

        tokio::spawn(async move {
            info!("Delay task pop thread started for shard {}", shard_no);
            let mut recv = stop_send.subscribe();
            loop {
                select! {
                    val = recv.recv() => {
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
                    res = pop_delay_queue(&manager, shard_no) => {
                        if let Err(e) = res {
                            error!("Delay task pop error on shard {}: {}", shard_no, e);
                        }
                    }
                }
            }
        });
    }
}

async fn pop_delay_queue(
    delay_task_manager: &Arc<DelayTaskManager>,
    shard_no: u32,
) -> Result<(), CommonError> {
    let queue_arc: SharedDelayQueue =
        if let Some(q) = delay_task_manager.delay_queue_list.get(&shard_no) {
            q.clone()
        } else {
            return Err(CommonError::CommonError(format!(
                "Delay task queue shard {} not found",
                shard_no
            )));
        };

    let mut delay_queue = queue_arc.lock().await;

    match tokio::time::timeout(
        Duration::from_millis(POP_LOCK_TIMEOUT_MS),
        delay_queue.next(),
    )
    .await
    {
        Ok(Some(expired)) => {
            let task = expired.into_inner();
            drop(delay_queue);
            spawn_task_process(delay_task_manager.clone(), task);
        }
        Ok(None) => {
            debug!(
                "Delay task queue shard {} returned None - queue may be empty or closed",
                shard_no
            );
            drop(delay_queue);
            sleep(Duration::from_millis(10)).await;
        }
        Err(_) => {
            drop(delay_queue);
        }
    }

    Ok(())
}

pub(crate) fn spawn_task_process(delay_task_manager: Arc<DelayTaskManager>, task: DelayTask) {
    tokio::spawn(async move {
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

        let _permit = permit;
        let task_type_str = format!("{:?}", task.task_type);
        if let Err(e) = delay_task_process(&delay_task_manager, &task).await {
            record_delay_task_execute_failed(&task_type_str);
            error!(
                "Failed to process delay task: task_id={}, task_type={:?}, error={}",
                task.task_id, task.task_type, e
            );
        }
    });
}

pub async fn delay_task_process(
    delay_task_manager: &Arc<DelayTaskManager>,
    task: &DelayTask,
) -> Result<(), CommonError> {
    debug!(
        "Processing delay task: task_id={}, task_type={:?}",
        task.task_id, task.task_type
    );

    delay_task_manager.remove_task_key(&task.task_id);

    let task_type_str = format!("{:?}", task.task_type);
    let latency_s = now_second().saturating_sub(task.delay_target_time) as f64;
    record_delay_task_schedule_latency(&task_type_str, latency_s);

    match task.task_type {
        DelayTaskType::MQTTSessionExpire => {
            handle_session_expire(task).await?;
        }
        DelayTaskType::MQTTLastwillExpire => {
            handle_lastwill_expire(task).await?;
        }
    }

    if task.persistent {
        delete_delay_task_index(&delay_task_manager.storage_driver_manager, &task.task_id).await?;
    }

    record_delay_task_executed(&task_type_str);
    Ok(())
}
