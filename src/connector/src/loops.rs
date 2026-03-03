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

use crate::core::BridgePluginReadConfig;
use crate::failure::{failure_message_process, FailureContext};
use crate::manager::ConnectorManager;
use crate::storage::connector::ConnectorStorage;
use crate::storage::message::MessageStorage;
use crate::traits::ConnectorSink;
use common_base::error::common::CommonError;
use common_base::tools::{now_millis, now_second};
use common_metrics::mqtt::connector::{
    record_connector_messages_sent_failure, record_connector_messages_sent_success,
    record_connector_send_duration,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::connector::status::MQTTStatus;
use metadata_struct::storage::{
    adapter_record::AdapterWriteRecord, convert::convert_engine_record_to_adapter,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::{select, sync::mpsc, time::sleep};
use tracing::{error, info};

pub async fn run_connector_loop<S: ConnectorSink>(
    sink: &S,
    client_pool: &Arc<ClientPool>,
    connector_manager: &Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector_name: String,
    config: BridgePluginReadConfig,
    mut stop_recv: mpsc::Receiver<bool>,
) -> Result<(), CommonError> {
    sink.validate().await?;

    let mut resource = sink.init_sink().await?;
    let message_storage = MessageStorage::new(storage_driver_manager.clone());
    let group_name = connector_name.clone();

    loop {
        let mut offsets = message_storage.get_group_offset(&group_name).await?;
        select! {
            val = stop_recv.recv() => {
                if let Some(flag) = val {
                    if flag {
                        sink.cleanup_sink(resource).await?;
                        break;
                    }
                }
            },

            val = message_storage.read_topic_message(&config.topic_name, &offsets, config.record_num) => {
                match val {
                    Ok(data) => {
                        connector_manager.report_heartbeat(&connector_name);

                        if data.is_empty() {
                            sleep(Duration::from_millis(100)).await;
                            continue;
                        }

                        let start_time = now_millis();
                        let message_count = data.len() as u64;
                        let mut retry_times = 0;

                        let (data_list, max_offsets) = extract_max_offsets_and_convert(&data);

                        loop{
                            match sink.send_batch(&data_list, &mut resource).await {
                                Ok(_) => {
                                    for (k,v) in max_offsets.iter(){
                                        offsets.insert(k.to_string(), *v);
                                    }
                                    message_storage.commit_group_offset(
                                        &group_name,
                                        &offsets,
                                    ).await?;

                                    update_last_active(
                                        connector_manager,
                                        &connector_name,
                                        start_time,
                                        message_count,
                                        true
                                    );
                                    break;
                                },
                                Err(e) => {
                                    update_last_active(
                                        connector_manager,
                                        &connector_name,
                                        start_time,
                                        message_count,
                                        false
                                    );
                                    let err_msg = e.to_string();
                                    error!("Connector {} failed to send batch: {}", connector_name, err_msg);
                                    let context = FailureContext {
                                        storage_driver_manager: &storage_driver_manager,
                                        connector_name: &connector_name,
                                        source_topic: &config.topic_name,
                                        error_message: &err_msg,
                                        records: &data_list,
                                    };
                                    if failure_message_process(config.strategy.clone(), retry_times, &context).await {
                                        for (k, v) in max_offsets.iter() {
                                            offsets.insert(k.to_string(), *v);
                                        }
                                        message_storage
                                            .commit_group_offset(&group_name, &offsets)
                                            .await?;
                                        sleep(Duration::from_millis(100)).await;
                                        break
                                    }
                                    retry_times +=1;
                                }
                            }
                        }

                    },
                    Err(e) => {
                        update_last_active(connector_manager, &connector_name, now_millis(), 0, false);
                        if stop_connector(client_pool,connector_manager, &connector_name, &e.to_string()).await?{
                            info!("Connector '{}' has been sealed up and stopped, reason: {}", connector_name, e);
                            break;
                        }
                        error!("Connector {} failed to read Topic {} data: {}", connector_name, config.topic_name, e);
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn stop_connector(
    client_pool: &Arc<ClientPool>,
    connector_manager: &Arc<ConnectorManager>,
    connector_name: &str,
    e: &str,
) -> Result<bool, CommonError> {
    if e.contains("not found in broker cache") && e.contains("Topic") {
        if let Some(mut connector) = connector_manager.get_connector(connector_name) {
            let storage = ConnectorStorage::new(client_pool.clone());
            connector.status = MQTTStatus::Stop;
            connector.update_time = now_second();
            storage.update_connector(connector).await?;
        }
        return Ok(true);
    }

    Ok(false)
}

fn extract_max_offsets_and_convert(
    data: &[metadata_struct::storage::storage_record::StorageRecord],
) -> (Vec<AdapterWriteRecord>, HashMap<String, u64>) {
    let mut data_list = Vec::with_capacity(data.len());
    let mut max_offsets = HashMap::new();

    for record in data {
        data_list.push(convert_engine_record_to_adapter(record.clone()));
        let current_offset = max_offsets
            .get(&record.metadata.shard)
            .copied()
            .unwrap_or(0);
        max_offsets.insert(
            record.metadata.shard.clone(),
            current_offset.max(record.metadata.offset + 1),
        );
    }

    (data_list, max_offsets)
}

pub fn update_last_active(
    connector_manager: &Arc<ConnectorManager>,
    connector_name: &str,
    start_time: u128,
    message_count: u64,
    success: bool,
) {
    if let Some(mut thread) = connector_manager.connector_thread.get_mut(connector_name) {
        thread.last_send_time = now_second();

        if success {
            thread.send_success_total += message_count;
            let duration_ms = (now_millis() - start_time) as f64;
            record_connector_messages_sent_success(connector_name.to_owned(), message_count);
            record_connector_send_duration(connector_name.to_owned(), duration_ms);
        } else {
            thread.send_fail_total += message_count;
            record_connector_messages_sent_failure(connector_name.to_owned(), message_count);
        }
    }
}
