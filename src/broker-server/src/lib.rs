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

use crate::{connection::network_connection_gc, grpc::start_grpc_server};
use admin_server::{
    server::AdminServer,
    state::{HttpState, MQTTContext, StorageEngineContext},
};
use broker_core::{
    cache::NodeCacheManager,
    heartbeat::{check_meta_service_status, register_node_and_start_heartbeat},
};
use common_base::{
    role::{is_broker_node, is_engine_node, is_meta_node},
    runtime::{
        create_runtime, resolve_broker_worker_threads, resolve_meta_worker_threads,
        resolve_server_worker_threads,
    },
    task::{TaskKind, TaskSupervisor},
};
use common_config::{broker::broker_config, config::BrokerConfig};
use common_healthy::port::{wait_for_engine_ready, wait_for_grpc_ready};
use common_metrics::{core::server::register_prometheus_export, init_metrics};
use connector::start_connector;
use delay_message::manager::start_delay_message_manager_thread;
use delay_task::{manager::DelayTaskManager, start_delay_task_manager_thread};
use grpc_clients::pool::ClientPool;
use meta_service::{MetaServiceServer, MetaServiceServerParams};
use mqtt_broker::broker::{MqttBrokerServer, MqttBrokerServerParams};
use network_server::common::connection_manager::ConnectionManager as NetworkConnectionManager;
use node_call::NodeCallManager;
use pprof_monitor::pprof_monitor::start_pprof_monitor;
use rate_limit::global::GlobalRateLimiterManager;
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::family::{column_family_list, storage_data_fold},
};
use std::{sync::Arc, thread::sleep, time::Duration};
use storage_adapter::driver::StorageDriverManager;
use storage_engine::{group::OffsetManager, StorageEngineParams, StorageEngineServer};
use system_info::{start_system_info_collection, start_tokio_runtime_info_collection};
use tokio::{runtime::Runtime, signal, sync::broadcast};
use tracing::{error, info};

mod cluster_service;
pub mod common;
mod connection;
mod grpc;
mod load_cache;
mod params;

pub struct BrokerServer {
    server_runtime: Runtime,
    meta_runtime: Runtime,
    /// Dedicated runtime for MQTT broker tasks; build_broker_mqtt_params is called
    /// inside broker_runtime.block_on() so tasks spawned during construction
    /// (e.g. RetainMessageManager's send thread) land here, not on server_runtime.
    broker_runtime: Runtime,
    place_params: MetaServiceServerParams,
    mqtt_params: MqttBrokerServerParams,
    engine_params: StorageEngineParams,
    client_pool: Arc<ClientPool>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    connection_manager: Arc<NetworkConnectionManager>,
    broker_cache: Arc<NodeCacheManager>,
    offset_manager: Arc<OffsetManager>,
    delay_task_manager: Arc<DelayTaskManager>,
    node_call_manager: Arc<NodeCallManager>,
    task_supervisor: Arc<TaskSupervisor>,
    global_rate_limiter: Arc<GlobalRateLimiterManager>,
    config: BrokerConfig,
}

impl Default for BrokerServer {
    fn default() -> Self {
        Self::new()
    }
}

impl BrokerServer {
    pub fn new() -> Self {
        init_metrics();
        let config = broker_config();
        let client_pool = Arc::new(ClientPool::new(config.runtime.channels_per_address));
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let server_worker_threads =
            resolve_server_worker_threads(config.runtime.server_worker_threads);
        let meta_worker_threads = resolve_meta_worker_threads(config.runtime.meta_worker_threads);
        let broker_worker_threads =
            resolve_broker_worker_threads(config.runtime.broker_worker_threads);
        let global_rate_limiter = Arc::new(
            GlobalRateLimiterManager::new(config.cluster_limit.max_network_connection_rate)
                .unwrap_or_else(|e| panic!("Failed to create GlobalRateLimiterManager: {e}")),
        );
        let server_runtime = create_runtime("server-runtime", server_worker_threads);
        let broker_cache = Arc::new(NodeCacheManager::new(config.clone()));
        let connection_manager = Arc::new(NetworkConnectionManager::new());
        let task_supervisor = Arc::new(TaskSupervisor::new());

        let (main_stop_send, _) = broadcast::channel(2);

        let offset_manager = Arc::new(OffsetManager::new(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            config.storage_runtime.offset_enable_cache,
        ));

        let node_call_manager = Arc::new(NodeCallManager::new(
            client_pool.clone(),
            broker_cache.clone(),
        ));

        // storage adapter driver (sync, no async needed)
        let engine_params = params::build_storage_engine_params(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            broker_cache.clone(),
            connection_manager.clone(),
            offset_manager.clone(),
            global_rate_limiter.clone(),
        );

        // Create meta_runtime here so that Raft::new() (inside build_meta_service) is
        // called within meta_runtime.block_on().  openraft spawns ~9 internal tasks via
        // tokio::spawn() during Raft::new(); those calls resolve against the *current*
        // runtime context, so they all land on meta_runtime instead of server_runtime.
        let meta_runtime = create_runtime("meta-runtime", meta_worker_threads);
        let mqtt_om = offset_manager.clone();
        let mqtt_seh = engine_params.storage_engine_handler.clone();
        let storage_driver_manager = meta_runtime.block_on(async move {
            match StorageDriverManager::new(mqtt_om.clone(), mqtt_seh).await {
                Ok(storage) => Arc::new(storage),
                Err(e) => {
                    error!("Failed to build message storage driver: {}", e);
                    std::process::exit(1);
                }
            }
        });

        let delay_task_manager: Arc<DelayTaskManager> = Arc::new(DelayTaskManager::new(
            client_pool.clone(),
            storage_driver_manager.clone(),
            1,
            10,
        ));

        // Run build_meta_service on meta_runtime so all openraft internal tasks
        // (core loop, log IO, state machine worker, etc.) are isolated from the
        // gRPC server_runtime, eliminating task-scheduler contention.
        let meta_params = meta_runtime.block_on(params::build_meta_service(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            delay_task_manager.clone(),
            node_call_manager.clone(),
            broker_cache.clone(),
            task_supervisor.clone(),
        ));

        // Create broker_runtime here so that tasks spawned during MQTT param
        // construction (e.g. RetainMessageManager's send thread) land on
        // broker_runtime rather than server_runtime.
        let broker_runtime = create_runtime("broker-runtime", broker_worker_threads);

        let mqtt_om = offset_manager.clone();
        let mqtt_cp = client_pool.clone();
        let mqtt_bc = broker_cache.clone();
        let mqtt_re = rocksdb_engine_handler.clone();
        let mqtt_cm = connection_manager.clone();
        let mqtt_stop = main_stop_send.clone();
        let mqttt_sdm = storage_driver_manager.clone();
        let mqtt_task_supervisor = task_supervisor.clone();
        let mqtt_global_rate_limiter = global_rate_limiter.clone();
        let mqtt_node_call = node_call_manager.clone();

        let mqtt_params = broker_runtime.block_on(async move {
            match params::build_broker_mqtt_params(
                mqtt_cp,
                mqtt_bc,
                mqtt_re,
                mqtt_cm,
                mqttt_sdm,
                mqtt_om,
                mqtt_task_supervisor,
                mqtt_global_rate_limiter,
                mqtt_node_call.clone(),
                mqtt_stop,
            )
            .await
            {
                Ok(params) => params,
                Err(e) => {
                    error!("Failed to build MQTT broker params: {}", e);
                    std::process::exit(1);
                }
            }
        });

        BrokerServer {
            broker_cache,
            server_runtime,
            meta_runtime,
            broker_runtime,
            place_params: meta_params,
            engine_params,
            config: config.clone(),
            mqtt_params,
            client_pool,
            delay_task_manager,
            rocksdb_engine_handler,
            connection_manager,
            node_call_manager,
            offset_manager,
            task_supervisor,
            global_rate_limiter,
        }
    }

    pub fn start(&self) {
        let config = broker_config();
        let monitor_interval_ms = config.prometheus.monitor_interval_ms;

        // ── Phase 1: Start network-facing servers ─────────────────────
        self.start_grpc_server();
        self.start_admin_server();
        self.start_pprof_server();
        self.start_prometheus_server();

        if !wait_for_grpc_ready(self.config.grpc_port) {
            std::process::exit(1);
        }

        // ── Phase 2: Meta (Raft) service ──────────────────────────────
        let meta_stop_send = self.start_meta_service();
        self.server_runtime.block_on(async {
            check_meta_service_status(self.client_pool.clone()).await;
        });

        // ── Phase 3: Load Cache ────────────────────────────────────
        self.start_load_cache();

        // ── Phase 4: NodeCallManager ───────────────────────────────────
        let (app_stop, _) = broadcast::channel::<bool>(2);
        let raw_app_stop = app_stop.clone();
        self.server_runtime.block_on(async {
            self.start_node_call_manager(raw_app_stop.clone());
            self.wait_for_node_call_manager_ready().await;
        });

        // ── Phase 5: Register node ─────────────────────────────────────
        let client_pool = self.client_pool.clone();
        let broker_cache = self.broker_cache.clone();
        let task_supervisor = self.task_supervisor.clone();
        self.server_runtime.block_on(async {
            register_node_and_start_heartbeat(
                &client_pool,
                &broker_cache,
                &task_supervisor,
                app_stop.clone(),
            )
            .await;
        });

        // ── Phase 6: Engine service ────────────────────────────────────
        let engine_stop_send = self.start_engine_service();

        // ── Phase 7: MQTT broker  ─────────────
        let mqtt_stop_send = self
            .server_runtime
            .block_on(async { self.start_mqtt_broker(app_stop.clone()).await });

        // ── Phase 8: Background services ──────────────────────────────
        self.server_runtime.block_on(async {
            self.start_background_services(app_stop.clone(), monitor_interval_ms)
                .await;
        });

        self.awaiting_stop(meta_stop_send, mqtt_stop_send, engine_stop_send);
    }

    fn start_grpc_server(&self) {
        let place_params = self.place_params.clone();
        let mqtt_params = self.mqtt_params.clone();
        let engine_params = self.engine_params.clone();
        let grpc_port = self.config.grpc_port;
        self.server_runtime.spawn(Box::pin(async move {
            if let Err(e) =
                start_grpc_server(place_params, mqtt_params, engine_params, grpc_port).await
            {
                error!("Failed to start GRPC server: {}", e);
                std::process::exit(1);
            }
        }));
    }

    fn start_admin_server(&self) {
        let broker_cache = self.broker_cache.clone();
        let state = Arc::new(HttpState {
            client_pool: self.client_pool.clone(),
            connection_manager: self.mqtt_params.connection_manager.clone(),
            mqtt_context: MQTTContext {
                cache_manager: self.mqtt_params.cache_manager.clone(),
                subscribe_manager: self.mqtt_params.subscribe_manager.clone(),
                metrics_manager: self.mqtt_params.metrics_cache_manager.clone(),
                connector_manager: self.mqtt_params.connector_manager.clone(),
                schema_manager: self.mqtt_params.schema_manager.clone(),
                retain_message_manager: self.mqtt_params.retain_message_manager.clone(),
                push_manager: self.mqtt_params.push_manager.clone(),
                storage_driver_manager: self.mqtt_params.storage_driver_manager.clone(),
            },
            engine_context: StorageEngineContext {
                engine_adapter_handler: self.engine_params.storage_engine_handler.clone(),
                cache_manager: self.engine_params.cache_manager.clone(),
            },
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            broker_cache,
            storage_driver_manager: self.mqtt_params.storage_driver_manager.clone(),
            rate_limiter: self.global_rate_limiter.clone(),
        });
        let http_port = self.config.http_port;
        self.server_runtime.spawn(async move {
            AdminServer::new().start(http_port, state).await;
        });
    }

    fn start_pprof_server(&self) {
        let pprof_port = self.config.pprof.port;
        let pprof_frequency = self.config.pprof.frequency;
        if self.config.pprof.enable {
            self.server_runtime.spawn(async move {
                start_pprof_monitor(pprof_port, pprof_frequency).await;
            });
        }
    }

    fn start_prometheus_server(&self) {
        let prometheus_port = self.config.prometheus.port;
        if self.config.prometheus.enable {
            self.server_runtime.spawn(async move {
                register_prometheus_export(prometheus_port).await;
            });
        }
    }

    fn start_load_cache(&self) {
        let cache_manager = self.mqtt_params.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        let connector_manager = self.mqtt_params.connector_manager.clone();
        let schema_manager = self.mqtt_params.schema_manager.clone();
        self.server_runtime.block_on(async {
            if let Err(e) = load_cache::load_metadata_cache(
                &cache_manager,
                &client_pool,
                &connector_manager,
                &schema_manager,
            )
            .await
            {
                error!("Failed to load metadata cache: {}", e);
                std::process::exit(1);
            }
        });

        if is_engine_node(&self.config.roles) {
            let engine_cache = self.engine_params.cache_manager.clone();
            let client_pool = self.client_pool.clone();
            self.server_runtime.block_on(async {
                if let Err(e) = load_cache::load_engine_cache(&engine_cache, &client_pool).await {
                    error!("Failed to load engine cache: {}", e);
                    std::process::exit(1);
                }
            });
        }
    }

    fn start_meta_service(&self) -> Option<broadcast::Sender<bool>> {
        if !is_meta_node(&self.config.roles) {
            return None;
        }
        let (stop_send, _) = broadcast::channel(2);
        let place_params = self.place_params.clone();
        let tx = stop_send.clone();
        self.meta_runtime.spawn(Box::pin(async move {
            MetaServiceServer::new(place_params, tx).start().await;
        }));
        Some(stop_send)
    }

    // broker_runtime was created in new() so all engine tasks (including those
    // spawned during construction) are already on the same runtime.
    fn start_engine_service(&self) -> Option<broadcast::Sender<bool>> {
        if !is_engine_node(&self.config.roles) {
            return None;
        }
        let (stop_send, _) = broadcast::channel(2);
        let stop_handle = stop_send.clone();
        let server = StorageEngineServer::new(
            self.engine_params.clone(),
            stop_send,
            self.task_supervisor.clone(),
        );
        self.broker_runtime.spawn(Box::pin(async move {
            server.start().await;
        }));
        if !wait_for_engine_ready(self.config.storage_runtime.tcp_port) {
            std::process::exit(1);
        }
        Some(stop_handle)
    }

    async fn start_mqtt_broker(
        &self,
        stop: broadcast::Sender<bool>,
    ) -> Option<broadcast::Sender<bool>> {
        if !is_broker_node(&self.config.roles) {
            return None;
        }
        let stop_handle = stop.clone();
        let server = MqttBrokerServer::new(self.mqtt_params.clone(), stop).await;
        self.broker_runtime.spawn(Box::pin(async move {
            server.start().await;
        }));
        Some(stop_handle)
    }

    fn start_node_call_manager(&self, stop: broadcast::Sender<bool>) {
        let node_call_manager = self.node_call_manager.clone();
        self.task_supervisor
            .spawn(TaskKind::BrokerNodeCall.to_string(), async move {
                node_call_manager.start(stop).await;
            });
    }

    async fn wait_for_node_call_manager_ready(&self) {
        use tokio::time::{sleep, Duration};
        loop {
            if self.node_call_manager.is_ready().await {
                return;
            }
            sleep(Duration::from_millis(5)).await;
        }
    }

    async fn start_background_services(
        &self,
        stop: broadcast::Sender<bool>,
        monitor_interval_ms: u64,
    ) {
        // delay task
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let broker_cache = self.broker_cache.clone();
        let delay_task_manager = self.delay_task_manager.clone();
        let node_call_manager = self.node_call_manager.clone();
        let task_supervisor = self.task_supervisor.clone();
        self.server_runtime.spawn(async move {
            if let Err(e) = start_delay_task_manager_thread(
                &rocksdb_engine_handler,
                &delay_task_manager,
                &broker_cache,
                &node_call_manager,
                &task_supervisor,
            )
            .await
            {
                error!("Failed to start DelayTask pop threads: {}", e);
                std::process::exit(1);
            }
        });

        // delay message
        let delay_message_manager = self.mqtt_params.delay_message_manager.clone();
        let broker_cache = self.broker_cache.clone();
        let task_supervisor = self.task_supervisor.clone();
        if let Err(e) = start_delay_message_manager_thread(
            &delay_message_manager,
            &task_supervisor,
            &broker_cache,
        )
        .await
        {
            error!("Failed to start delay message manager, error:{}", e);
            std::process::exit(1);
        }

        // connection gc
        let connection_manager = self.connection_manager.clone();
        let tx = stop.clone();
        self.task_supervisor
            .spawn(TaskKind::NetworkConnectionGC.to_string(), async move {
                network_connection_gc(connection_manager, tx).await
            });

        // offset async commit
        let offset_cache = self.offset_manager.clone();
        let tx = stop.clone();
        self.task_supervisor.spawn(
            TaskKind::OffsetAsyncCommit.to_string(),
            Box::pin(async move {
                offset_cache.offset_async_save_thread(tx).await;
            }),
        );

        // system info collection
        let tx = stop.clone();
        self.task_supervisor
            .spawn(TaskKind::SystemInfoCollection.to_string(), async move {
                start_system_info_collection(tx, monitor_interval_ms).await;
            });

        // tokio runtime info collection
        let runtime_handles = vec![
            ("server".to_string(), self.server_runtime.handle().clone()),
            ("meta".to_string(), self.meta_runtime.handle().clone()),
            ("broker".to_string(), self.broker_runtime.handle().clone()),
        ];
        let tx = stop.clone();
        self.task_supervisor.spawn(
            TaskKind::TokioRuntimeInfoCollection.to_string(),
            async move {
                start_tokio_runtime_info_collection(runtime_handles, tx, monitor_interval_ms).await;
            },
        );

        //
        let message_storage = self.mqtt_params.storage_driver_manager.clone();
        let connector_manager = self.mqtt_params.connector_manager.clone();
        let client_pool = self.client_pool.clone();
        let task_supervisor = self.task_supervisor.clone();
        self.server_runtime.spawn(Box::pin(async move {
            start_connector(
                &client_pool,
                &message_storage,
                &connector_manager,
                &task_supervisor,
                &stop,
            )
            .await;
        }));
    }

    pub fn awaiting_stop(
        &self,
        meta_stop: Option<broadcast::Sender<bool>>,
        mqtt_stop: Option<broadcast::Sender<bool>>,
        engine_stop: Option<broadcast::Sender<bool>>,
    ) {
        self.server_runtime.block_on(Box::pin(async {
            self.broker_cache
                .set_status(common_base::node_status::NodeStatus::Running)
                .await;
            signal::ctrl_c().await.expect("failed to listen for event");
            info!(
                "{}",
                "When ctrl + c is received, the service starts to stop"
            );

            self.broker_cache
                .set_status(common_base::node_status::NodeStatus::Stopping)
                .await;

            if let Some(sx) = mqtt_stop {
                if let Err(e) = sx.send(true) {
                    error!("mqtt stop signal, error message:{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Err(e) = self.offset_manager.flush().await {
                error!(
                    "Offset manager flush operation failed. Error message: {}",
                    e
                );
            }

            if let Err(e) = self.delay_task_manager.stop().await {
                error!("delay task stop signal, error message{}", e);
            }

            if let Some(sx) = engine_stop {
                if let Err(e) = sx.send(true) {
                    error!("storage engine stop signal, error message{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Some(sx) = meta_stop {
                if let Err(e) = sx.send(true) {
                    error!("meta stop signal, error message{}", e);
                }
            }

            sleep(Duration::from_secs(3));
        }));
    }
}
