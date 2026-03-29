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

use broker_core::cache::NodeCacheManager;
use common_base::{error::common::CommonError, role::is_broker_node, task::TaskSupervisor};
use connector::manager::ConnectorManager;
use delay_message::manager::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use mqtt_broker::{
    broker::{MqttBrokerServer, MqttBrokerServerParams},
    core::{
        cache::MQTTCacheManager as MqttCacheManager, event::EventReportManager,
        retain::RetainMessageManager,
    },
    security::AuthManager,
    storage::session::SessionBatcher,
    subscribe::{manager::SubscribeManager, PushManager},
};
use network_server::command::ArcCommandAdapter;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager;
use node_call::NodeCallManager;
use rate_limit::global::GlobalRateLimiterManager;
use rocksdb_engine::{metrics::mqtt::MQTTMetricsCache, rocksdb::RocksDBEngine};
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use storage_engine::group::OffsetManager;
use tokio::sync::broadcast;
use tracing::error;

use crate::BrokerServer;

pub struct MqttBuildParams {
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub offset_manager: Arc<OffsetManager>,
    pub task_supervisor: Arc<TaskSupervisor>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub node_call: Arc<NodeCallManager>,
    pub stop_sx: broadcast::Sender<bool>,
    pub request_channel: Arc<RequestChannel>,
}

pub fn build_mqtt_params(
    p: MqttBuildParams,
    broker_runtime: &tokio::runtime::Runtime,
) -> MqttBrokerServerParams {
    let cp = p.client_pool;
    let bc = p.broker_cache;
    let re = p.rocksdb_engine_handler;
    let cm = p.connection_manager;
    let sdm = p.storage_driver_manager;
    let om = p.offset_manager;
    let ts = p.task_supervisor;
    let glm = p.global_limit_manager;
    let nc = p.node_call;
    let stop = p.stop_sx;
    let request_channel = p.request_channel;

    broker_runtime.block_on(async move {
        match build_broker_mqtt_params(cp, bc, re, cm, sdm, om, ts, glm, nc, stop, request_channel)
            .await
        {
            Ok(params) => params,
            Err(e) => {
                error!("Failed to build MQTT broker params: {}", e);
                std::process::exit(1);
            }
        }
    })
}

/// Build [`MqttBrokerServerParams`] on the caller's runtime so that tasks
/// spawned during construction (e.g. `RetainMessageManager`) land there.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn build_broker_mqtt_params(
    client_pool: Arc<ClientPool>,
    broker_cache: Arc<NodeCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    offset_manager: Arc<OffsetManager>,
    task_supervisor: Arc<TaskSupervisor>,
    global_limit_manager: Arc<GlobalRateLimiterManager>,
    node_call: Arc<NodeCallManager>,
    stop_sx: broadcast::Sender<bool>,
    request_channel: Arc<RequestChannel>,
) -> Result<MqttBrokerServerParams, CommonError> {
    let cache_manager = Arc::new(MqttCacheManager::new(
        client_pool.clone(),
        broker_cache.clone(),
    ));
    let subscribe_manager = Arc::new(SubscribeManager::new());
    let connector_manager = Arc::new(ConnectorManager::new());
    let auth_driver = Arc::new(AuthManager::new(cache_manager.clone()));
    let delay_message_manager = Arc::new(
        DelayMessageManager::new(client_pool.clone(), storage_driver_manager.clone(), 5).await?,
    );
    let metrics_cache_manager = Arc::new(MQTTMetricsCache::new(rocksdb_engine_handler.clone()));
    let schema_manager = Arc::new(SchemaRegisterManager::new());
    let retain_message_manager = RetainMessageManager::new(
        cache_manager.clone(),
        client_pool.clone(),
        connection_manager.clone(),
        stop_sx,
    );
    let push_manager = Arc::new(PushManager::new(
        cache_manager.clone(),
        storage_driver_manager.clone(),
        connection_manager.clone(),
        rocksdb_engine_handler.clone(),
        subscribe_manager.clone(),
        client_pool.clone(),
    ));

    let session_batcher = SessionBatcher::new();
    let event_manager = EventReportManager::new();

    Ok(MqttBrokerServerParams {
        cache_manager,
        client_pool,
        session_batcher,
        event_manager,
        storage_driver_manager,
        subscribe_manager,
        connection_manager,
        connector_manager,
        auth_driver,
        delay_message_manager,
        schema_manager,
        metrics_cache_manager,
        rocksdb_engine_handler,
        node_cache: broker_cache,
        offset_manager,
        retain_message_manager,
        push_manager,
        task_supervisor,
        global_limit_manager,
        node_call,
        request_channel,
    })
}

impl BrokerServer {
    pub async fn start_mqtt_broker(
        &self,
        stop: broadcast::Sender<bool>,
    ) -> Option<(broadcast::Sender<bool>, ArcCommandAdapter)> {
        if !is_broker_node(&self.config.roles) {
            return None;
        }
        let stop_handle = stop.clone();
        let server = MqttBrokerServer::new(self.mqtt_params.clone(), stop).await;
        let command = server.command.clone();
        self.broker_runtime.spawn(Box::pin(async move {
            server.start().await;
        }));
        Some((stop_handle, command))
    }
}
