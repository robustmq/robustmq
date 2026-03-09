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

use std::sync::Arc;

use common_base::task::{TaskKind, TaskSupervisor};
use grpc_clients::pool::ClientPool;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;

use crate::{
    core::start_connector_thread, heartbeat::start_connector_report_heartbeat_thread,
    manager::ConnectorManager,
};

pub mod cassandra;
pub mod clickhouse_connector;
pub mod core;
pub mod elasticsearch;
pub mod failure;
pub mod file;
pub mod greptimedb;
pub mod heartbeat;
pub mod influxdb_connector;
pub mod kafka;
pub mod loops;
pub mod manager;
pub mod mongodb;
pub mod mqtt_bridge;
pub mod mysql;
pub mod opentsdb;
pub mod postgres;
pub mod pulsar;
pub mod rabbitmq;
pub mod redis;
pub mod s3;
pub mod storage;
pub mod traits;
pub mod webhook;

pub async fn start_connector(
    client_pool: &Arc<ClientPool>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    connector_manager: &Arc<ConnectorManager>,
    task_supervisor: &Arc<TaskSupervisor>,
    stop_send: &broadcast::Sender<bool>,
) {
    // connector check
    let raw_message_storage = storage_driver_manager.clone();
    let raw_connector_manager = connector_manager.clone();
    let raw_stop_send = stop_send.clone();
    let raw_client_poll = client_pool.clone();
    task_supervisor.spawn(
        TaskKind::ConnectorManager.to_string(),
        Box::pin(async move {
            start_connector_thread(
                raw_client_poll,
                raw_message_storage,
                raw_connector_manager,
                raw_stop_send,
            )
            .await;
        }),
    );

    // connector heartbeat
    let raw_connector_manager = connector_manager.clone();
    let raw_stop_send = stop_send.clone();
    let raw_client_poll = client_pool.clone();
    task_supervisor.spawn(
        TaskKind::ConnectorHeartbeat.to_string(),
        Box::pin(async move {
            start_connector_report_heartbeat_thread(
                raw_client_poll,
                raw_connector_manager,
                raw_stop_send,
            )
            .await;
        }),
    );
}
