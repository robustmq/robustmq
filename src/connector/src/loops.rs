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
use crate::failure::{failure_message_process, FailureRecordInfo};
use crate::manager::ConnectorManager;
use crate::storage::connector::ConnectorStorage;
use crate::traits::ConnectorSink;
use common_base::error::common::CommonError;
use common_base::tools::{now_millis, now_second};
use common_metrics::mqtt::connector::{
    record_connector_messages_sent_failure, record_connector_messages_sent_success,
    record_connector_offset_commit_failure, record_connector_send_duration,
    record_connector_source_read_failure,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::connector::status::MQTTStatus;
use metadata_struct::connector::FailureHandlingStrategy;
use metadata_struct::storage::{adapter_read_config::AdapterReadConfig, record::StorageRecord};
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::consumer::GroupConsumer;
use storage_adapter::driver::StorageDriverManager;
use tokio::{select, sync::mpsc, time::sleep};
use tracing::{error, info};

enum SendResultAction {
    Retry,
    BatchDone,
}

enum ReadErrorAction {
    Stop,
    Continue,
}

struct SendFailureParams<'a> {
    data_list: &'a [StorageRecord],
    start_time: u128,
    message_count: u64,
    retry_times: u32,
    error: CommonError,
}

struct SendSuccessParams<'a> {
    strategy: &'a FailureHandlingStrategy,
    fail_messages: &'a [FailureRecordInfo],
    start_time: u128,
    message_count: u64,
}

struct BatchCtx<'a> {
    connector_name: &'a str,
    connector_type: &'a str,
    tenant: &'a str,
    storage_driver_manager: &'a Arc<StorageDriverManager>,
    connector_manager: &'a Arc<ConnectorManager>,
}

pub async fn run_connector_loop<S: ConnectorSink>(
    sink: &S,
    client_pool: &Arc<ClientPool>,
    connector_manager: &Arc<ConnectorManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    connector_name: String,
    config: BridgePluginReadConfig,
    mut stop_recv: mpsc::Receiver<bool>,
) -> Result<(), CommonError> {
    sink.validate().await?;

    let mut resource = Some(sink.init_sink().await?);
    let connector_tenant = config.tenant.clone();
    let connector_type = connector_manager
        .get_connector(&connector_name)
        .map(|c| c.connector_type.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let ctx = BatchCtx {
        connector_name: &connector_name,
        connector_type: &connector_type,
        tenant: &connector_tenant,
        storage_driver_manager,
        connector_manager,
    };

    let consumer = GroupConsumer::new_manual(storage_driver_manager.clone(), &connector_name);
    let read_config = AdapterReadConfig {
        max_record_num: config.record_num,
        max_size: 1024 * 1024 * 30,
    };

    let mut run_result: Result<(), CommonError> = Ok(());

    'run: loop {
        select! {
            val = stop_recv.recv() => {
                match val {
                    Some(true) | None => break,
                    Some(false) => {}
                }
            },

            val = consumer.next_messages(&config.tenant, &config.topic_name, &read_config) => {
                match val {
                    Ok(data) => {
                        connector_manager.report_heartbeat(&connector_tenant, &connector_name);

                        if data.is_empty() {
                            sleep(Duration::from_millis(100)).await;
                            continue;
                        }

                        let start_time = now_millis();
                        let message_count = data.len() as u64;
                        let mut retry_times: u32 = 0;

                        loop {
                            match sink.send_batch(
                                &data,
                                resource
                                    .as_mut()
                                    .expect("sink resource must exist during connector loop"),
                            )
                            .await
                            {
                                Ok(fail_messages) => {
                                    if let Err(e) = handle_send_success(
                                        &ctx,
                                        &consumer,
                                        SendSuccessParams {
                                            strategy: &config.strategy,
                                            fail_messages: &fail_messages,
                                            start_time,
                                            message_count,
                                        },
                                    )
                                    .await
                                    {
                                        run_result = Err(e);
                                        break 'run;
                                    }
                                    break;
                                }
                                Err(e) => {
                                    match handle_send_failure(
                                        &ctx,
                                        &consumer,
                                        &config,
                                        SendFailureParams {
                                            data_list: &data,
                                            start_time,
                                            message_count,
                                            retry_times,
                                            error: e,
                                        },
                                    )
                                    .await
                                    {
                                        Ok(SendResultAction::BatchDone) => break,
                                        Ok(SendResultAction::Retry) => {
                                            retry_times += 1;
                                        }
                                        Err(err) => {
                                            run_result = Err(err);
                                            break 'run;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        match handle_read_error(client_pool, &ctx, &config.topic_name, e).await {
                            Ok(ReadErrorAction::Stop) => break,
                            Ok(ReadErrorAction::Continue) => {}
                            Err(err) => {
                                run_result = Err(err);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(raw_resource) = resource.take() {
        if let Err(cleanup_err) = sink.cleanup_sink(raw_resource).await {
            if run_result.is_ok() {
                run_result = Err(cleanup_err);
            } else {
                error!(
                    "Connector '{}' cleanup failed after run error, cleanup_error={}",
                    connector_name, cleanup_err
                );
            }
        }
    }

    run_result
}

async fn handle_send_success(
    ctx: &BatchCtx<'_>,
    consumer: &GroupConsumer,
    params: SendSuccessParams<'_>,
) -> Result<(), CommonError> {
    commit_consumer_offsets(ctx, consumer).await?;
    process_fail_messages(
        ctx.storage_driver_manager,
        params.strategy,
        params.fail_messages,
    )
    .await;
    update_last_active(
        ctx.connector_manager,
        ctx.tenant,
        ctx.connector_name,
        ctx.connector_type,
        params.start_time,
        params.message_count,
        true,
    );
    Ok(())
}

async fn handle_send_failure(
    ctx: &BatchCtx<'_>,
    consumer: &GroupConsumer,
    config: &BridgePluginReadConfig,
    params: SendFailureParams<'_>,
) -> Result<SendResultAction, CommonError> {
    if params.retry_times == 0 {
        update_last_active(
            ctx.connector_manager,
            ctx.tenant,
            ctx.connector_name,
            ctx.connector_type,
            params.start_time,
            params.message_count,
            false,
        );
    }

    let err_msg = params.error.to_string();
    error!(
        connector_name = ctx.connector_name,
        retry_times = params.retry_times,
        "failed to send batch: {}",
        err_msg
    );

    let context = FailureRecordInfo {
        tenant: ctx.tenant.to_string(),
        connector_name: ctx.connector_name.to_string(),
        connector_type: ctx.connector_type.to_string(),
        source_topic: config.topic_name.clone(),
        error_message: err_msg,
        records: params.data_list.to_vec(),
    };

    if failure_message_process(
        ctx.storage_driver_manager,
        &config.strategy,
        params.retry_times,
        &context,
    )
    .await
    {
        commit_consumer_offsets(ctx, consumer).await?;
        sleep(Duration::from_millis(100)).await;
        return Ok(SendResultAction::BatchDone);
    }

    Ok(SendResultAction::Retry)
}

async fn handle_read_error(
    client_pool: &Arc<ClientPool>,
    ctx: &BatchCtx<'_>,
    topic_name: &str,
    error: CommonError,
) -> Result<ReadErrorAction, CommonError> {
    record_connector_source_read_failure(
        ctx.tenant,
        ctx.connector_type.to_string(),
        ctx.connector_name.to_string(),
    );
    update_last_active(
        ctx.connector_manager,
        ctx.tenant,
        ctx.connector_name,
        ctx.connector_type,
        now_millis(),
        0,
        false,
    );
    match stop_connector(
        client_pool,
        ctx.connector_manager,
        ctx.connector_name,
        &error,
    )
    .await
    {
        Ok(true) => {
            info!(
                connector_name = ctx.connector_name,
                "connector sealed and stopped, reason: {}", error
            );
            Ok(ReadErrorAction::Stop)
        }
        Ok(false) => {
            error!(
                connector_name = ctx.connector_name,
                topic_name, "failed to read topic data: {}", error
            );
            sleep(Duration::from_millis(100)).await;
            Ok(ReadErrorAction::Continue)
        }
        Err(err) => Err(err),
    }
}

async fn process_fail_messages(
    storage_driver_manager: &Arc<StorageDriverManager>,
    strategy: &FailureHandlingStrategy,
    fail_messages: &[FailureRecordInfo],
) {
    for context in fail_messages {
        failure_message_process(storage_driver_manager, strategy, 99999, context).await;
    }
}

async fn commit_consumer_offsets(
    ctx: &BatchCtx<'_>,
    consumer: &GroupConsumer,
) -> Result<(), CommonError> {
    consumer.commit().await.inspect_err(|_| {
        record_connector_offset_commit_failure(
            ctx.tenant,
            ctx.connector_type.to_string(),
            ctx.connector_name.to_string(),
        );
    })
}

async fn stop_connector(
    client_pool: &Arc<ClientPool>,
    connector_manager: &Arc<ConnectorManager>,
    connector_name: &str,
    error: &CommonError,
) -> Result<bool, CommonError> {
    if should_stop_by_read_error(error) {
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

fn should_stop_by_read_error(error: &CommonError) -> bool {
    match error {
        CommonError::TopicNotFoundInBrokerCache(_, _) => true,
        CommonError::CommonError(message) => {
            message.contains("not found in broker cache") && message.contains("Topic")
        }
        _ => false,
    }
}

pub fn update_last_active(
    connector_manager: &Arc<ConnectorManager>,
    tenant: &str,
    connector_name: &str,
    connector_type: &str,
    start_time: u128,
    message_count: u64,
    success: bool,
) {
    let tenant = tenant.to_owned();
    connector_manager.update_connector_thread_last_active(connector_name, |thread| {
        thread.last_send_time = now_second();
        if success {
            thread.send_success_total += message_count;
            let duration_ms = (now_millis() - start_time) as f64;
            record_connector_messages_sent_success(
                &tenant,
                connector_type.to_owned(),
                connector_name.to_owned(),
                message_count,
            );
            record_connector_send_duration(
                &tenant,
                connector_type.to_owned(),
                connector_name.to_owned(),
                duration_ms,
            );
        } else {
            thread.send_fail_total += message_count;
            record_connector_messages_sent_failure(
                &tenant,
                connector_type.to_owned(),
                connector_name.to_owned(),
                message_count,
            );
        }
    });
}
