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

use crate::delay::{delete_delay_index_info, delete_delay_message, DELAY_QUEUE_MESSAGE_TOPIC};
use crate::manager::{DelayMessageManager, ShardCmd, DELAY_MESSAGE_SAVE_MS};
use common_base::error::common::CommonError;
use common_base::task::{TaskKind, TaskSupervisor};
use common_base::tools::now_second;
use common_metrics::mqtt::delay::{
    record_delay_msg_deliver, record_delay_msg_deliver_duration, record_delay_msg_deliver_fail,
};
use futures::StreamExt;
use metadata_struct::adapter::adapter_record::{AdapterWriteRecord, RecordHeader};
use metadata_struct::delay_info::DelayMessageIndexInfo;
use metadata_struct::storage::record::StorageRecord;
use metadata_struct::tenant::DEFAULT_TENANT;
use std::sync::Arc;
use std::time::Instant;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::{broadcast, mpsc};
use tokio::{select, sync::broadcast as bc};
use tokio_util::time::DelayQueue;
use tracing::{debug, error, info, warn};

pub(crate) fn spawn_delay_message_pop_threads(
    delay_message_manager: &Arc<DelayMessageManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    delay_queue_num: u32,
) {
    info!("Starting delay message pop threads ({})", delay_queue_num);

    for shard_no in 0..delay_queue_num {
        let manager = delay_message_manager.clone();

        // Create command channel for this shard.
        let (tx, rx) = mpsc::unbounded_channel::<ShardCmd>();
        delay_message_manager.register_shard_cmd_tx(shard_no, tx);

        let (stop_send, _) = broadcast::channel(2);
        delay_message_manager.add_delay_queue_pop_thread(shard_no, stop_send.clone());

        task_supervisor.spawn(
            format!("{}_{}", TaskKind::DelayMessagePop, shard_no),
            async move {
                run_shard_loop(shard_no, rx, stop_send, manager).await;
            },
        );
    }
}

/// Per-shard event loop.
/// Owns the DelayQueue exclusively — no Mutex needed.
/// Uses select! to react to either:
///   - a command from the manager (Insert)
///   - a message expiring in the DelayQueue
async fn run_shard_loop(
    shard_no: u32,
    mut rx: mpsc::UnboundedReceiver<ShardCmd>,
    stop_send: bc::Sender<bool>,
    manager: Arc<DelayMessageManager>,
) {
    let mut delay_queue: DelayQueue<DelayMessageIndexInfo> = DelayQueue::new();
    let mut stop_recv = stop_send.subscribe();

    loop {
        select! {
            // Stop signal
            val = stop_recv.recv() => {
                match val {
                    Ok(flag) if flag => {
                        info!("Delay message pop thread stopped for shard {}", shard_no);
                        break;
                    }
                    Err(_) => {
                        warn!("Broadcast channel closed, stopping pop thread for shard {}", shard_no);
                        break;
                    }
                    _ => {}
                }
            }

            // Command from manager (Insert / Delete)
            cmd = rx.recv() => {
                match cmd {
                    Some(ShardCmd::Insert(delay_info, target_instant, key_tx)) => {
                        let key = delay_queue.insert_at(delay_info, target_instant);
                        let _ = key_tx.send(key);
                    }
                    Some(ShardCmd::Delete(key, done_tx)) => {
                        delay_queue.remove(&key);
                        let _ = done_tx.send(());
                    }
                    None => {
                        // Channel closed — manager dropped, exit.
                        break;
                    }
                }
            }

            // Expired message
            Some(expired) = delay_queue.next() => {
                let delay_message = expired.into_inner();
                manager.remove_message_key(&delay_message.unique_id);
                let storage = manager.storage_driver_manager.clone();
                tokio::spawn(async move {
                    if let Err(e) = delay_message_process(
                        &storage,
                        &delay_message,
                        now_second(),
                    )
                    .await
                    {
                        error!(
                            "Failed to process delay message: offset={}, target={}, error={}",
                            delay_message.offset, delay_message.target_topic_name, e
                        );
                    }
                });
            }
        }
    }
}

pub async fn delay_message_process(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_info: &DelayMessageIndexInfo,
    trigger_time: u64,
) -> Result<(), CommonError> {
    let start = Instant::now();

    match send_delay_message_to_shard(storage_driver_manager, delay_info, trigger_time).await {
        Ok(offset) => {
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            record_delay_msg_deliver();
            record_delay_msg_deliver_duration(duration_ms);

            info!(
                "Delay message processed successfully. unique_id={}, target_topic={}, offset={}, duration_ms={:.2}",
                delay_info.unique_id,
                delay_info.target_topic_name,
                offset,
                duration_ms
            );
        }
        Err(e) => {
            record_delay_msg_deliver_fail();

            error!(
                "Failed to send delay message to target shard. unique_id={}, target_topic={}, offset={}, error={}",
                delay_info.unique_id, delay_info.target_topic_name, delay_info.offset, e
            );
        }
    };
    delete_delay_index_info(storage_driver_manager, delay_info).await?;
    delete_delay_message(storage_driver_manager, &delay_info.unique_id).await?;

    Ok(())
}

async fn send_delay_message_to_shard(
    storage_driver_manager: &Arc<StorageDriverManager>,
    delay_message: &DelayMessageIndexInfo,
    trigger_time: u64,
) -> Result<u64, CommonError> {
    // read data
    let results = storage_driver_manager
        .read_by_key(
            DEFAULT_TENANT,
            DELAY_QUEUE_MESSAGE_TOPIC,
            &delay_message.unique_id,
        )
        .await?;

    if results.is_empty() {
        return Err(CommonError::CommonError(format!(
            "Delay message not found: unique_id={}, offset={}",
            delay_message.unique_id, delay_message.offset
        )));
    }

    if results.len() > 1 {
        return Err(CommonError::CommonError(format!(
            "Multiple delay messages found for unique_id={}, expected 1 but found {}",
            delay_message.unique_id,
            results.len()
        )));
    }

    let record = if let Some(record) = results.first() {
        record.clone()
    } else {
        return Err(CommonError::CommonError(format!(
            "Failed to retrieve delay message record for unique_id={}",
            delay_message.unique_id
        )));
    };

    let send_record = build_new_record(delay_message, &record, trigger_time);

    // send to target topic under the original tenant
    let resp = storage_driver_manager
        .write(
            &delay_message.tenant,
            &delay_message.target_topic_name,
            &[send_record],
        )
        .await?;

    let write_resp = if let Some(data) = resp.first() {
        data.clone()
    } else {
        return Err(CommonError::CommonError(format!(
            "Write response is empty when sending delay message to topic '{}'",
            delay_message.target_topic_name
        )));
    };

    if write_resp.is_error() {
        return Err(CommonError::CommonError(write_resp.error_info()));
    }
    debug!(
        "Expired delay message sent successfully: delay queue -> {} (offset: {})",
        delay_message.target_topic_name, delay_message.offset
    );
    Ok(write_resp.offset)
}

fn build_new_record(
    delay_message: &DelayMessageIndexInfo,
    record: &StorageRecord,
    trigger_time: u64,
) -> AdapterWriteRecord {
    let trigger_header = RecordHeader {
        name: DELAY_MESSAGE_SAVE_MS.to_string(),
        value: trigger_time.to_string(),
    };

    let mut send_record =
        AdapterWriteRecord::new(delay_message.target_topic_name.clone(), record.data.clone())
            .with_protocol_data(record.protocol_data.clone());

    // header
    let header = if let Some(header) = record.metadata.header.clone() {
        let mut new_header = Vec::new();
        for raw in header {
            new_header.push(RecordHeader {
                name: raw.name,
                value: raw.value,
            });
        }
        new_header.push(trigger_header);
        new_header
    } else {
        vec![trigger_header]
    };
    send_record = send_record.with_header(header);

    // key
    if let Some(key) = record.metadata.key.clone() {
        send_record = send_record.with_key(key);
    }

    // tags
    if let Some(tags) = record.metadata.tags.clone() {
        send_record = send_record.with_tags(tags);
    }

    send_record
}
#[cfg(test)]
mod test {}
