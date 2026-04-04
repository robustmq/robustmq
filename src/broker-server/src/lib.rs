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

use amqp_broker::broker::AmqpBrokerServerParams;
use broker_core::{
    cache::NodeCacheManager,
    heartbeat::{check_meta_service_status, register_node_and_start_heartbeat},
};
use common_base::{
    role::is_broker_node,
    runtime::{
        create_runtime, resolve_broker_worker_threads, resolve_meta_worker_threads,
        resolve_server_worker_threads,
    },
    task::TaskSupervisor,
};
use common_config::{broker::broker_config, config::BrokerConfig};
use common_healthy::port::wait_for_grpc_ready;
use common_metrics::init_metrics;
use common_security::manager::SecurityManager;
use delay_task::manager::DelayTaskManager;
use grpc_clients::pool::ClientPool;
use kafka_broker::broker::KafkaBrokerServerParams;
use meta_service::MetaServiceServerParams;
use mqtt_broker::broker::MqttBrokerServerParams;
use nats_broker::broker::NatsBrokerServerParams;
use network_server::command::CommandRegistry;
use network_server::common::channel::RequestChannel;
use network_server::common::connection_manager::ConnectionManager as NetworkConnectionManager;
use node_call::NodeCallManager;
use rate_limit::global::GlobalRateLimiterManager;
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::family::{column_family_list, storage_data_fold},
};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use storage_engine::{group::OffsetManager, StorageEngineParams};
use tokio::{runtime::Runtime, sync::broadcast};
use tracing::error;

mod amqp;
mod cluster_service;
pub mod common;
mod connection;
mod daemon;
mod dynamic_cache;
mod engine;
mod grpc;
mod kafka;
mod load_cache;
mod meta;
mod mqtt;
mod nats;
mod server;

/// Shared infrastructure created before any protocol or storage layer.
struct BaseComponents {
    server_runtime: Runtime,
    client_pool: Arc<ClientPool>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    global_rate_limiter: Arc<GlobalRateLimiterManager>,
    broker_cache: Arc<NodeCacheManager>,
    connection_manager: Arc<NetworkConnectionManager>,
    task_supervisor: Arc<TaskSupervisor>,
    offset_manager: Arc<OffsetManager>,
    node_call_manager: Arc<NodeCallManager>,
}

pub struct BrokerServer {
    pub(crate) server_runtime: Runtime,
    pub(crate) meta_runtime: Runtime,
    /// Dedicated runtime for broker tasks; tasks spawned during construction
    /// (e.g. RetainMessageManager's send thread) land here, not on server_runtime.
    pub(crate) broker_runtime: Runtime,
    pub(crate) engine_runtime: Runtime,
    pub(crate) meta_params: MetaServiceServerParams,
    pub(crate) mqtt_params: MqttBrokerServerParams,
    pub(crate) kafka_params: KafkaBrokerServerParams,
    pub(crate) amqp_params: AmqpBrokerServerParams,
    pub(crate) nats_params: NatsBrokerServerParams,
    pub(crate) engine_params: StorageEngineParams,
    pub(crate) client_pool: Arc<ClientPool>,
    pub(crate) rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub(crate) connection_manager: Arc<NetworkConnectionManager>,
    pub(crate) broker_cache: Arc<NodeCacheManager>,
    pub(crate) offset_manager: Arc<OffsetManager>,
    pub(crate) delay_task_manager: Arc<DelayTaskManager>,
    pub(crate) node_call_manager: Arc<NodeCallManager>,
    pub(crate) task_supervisor: Arc<TaskSupervisor>,
    pub(crate) global_rate_limiter: Arc<GlobalRateLimiterManager>,
    pub(crate) config: BrokerConfig,
    pub(crate) shared_request_channel: Arc<RequestChannel>,
}

impl Default for BrokerServer {
    fn default() -> Self {
        Self::new()
    }
}

impl BrokerServer {
    pub fn new() -> Self {
        init_metrics();

        let config: &BrokerConfig = broker_config();

        let (base, meta_runtime, broker_runtime, engine_runtime) = Self::init_base(config);

        let (engine_params, storage_driver_manager, delay_task_manager, meta_params) =
            Self::init_storage(config, &base, &meta_runtime);

        let (mqtt_params, kafka_params, amqp_params, nats_params, shared_request_channel) =
            Self::init_protocol_params(config, &base, &storage_driver_manager, &broker_runtime);

        BrokerServer {
            server_runtime: base.server_runtime,
            meta_runtime,
            broker_runtime,
            engine_runtime,
            broker_cache: base.broker_cache,
            client_pool: base.client_pool,
            rocksdb_engine_handler: base.rocksdb_engine_handler,
            connection_manager: base.connection_manager,
            offset_manager: base.offset_manager,
            node_call_manager: base.node_call_manager,
            task_supervisor: base.task_supervisor,
            global_rate_limiter: base.global_rate_limiter,
            config: config.clone(),
            engine_params,
            delay_task_manager,
            meta_params,
            mqtt_params,
            kafka_params,
            amqp_params,
            nats_params,
            shared_request_channel,
        }
    }

    /// Initialize shared infrastructure: runtimes, RocksDB, connection manager,
    /// rate limiter, offset manager, and node call manager.
    fn init_base(config: &BrokerConfig) -> (BaseComponents, Runtime, Runtime, Runtime) {
        let client_pool = Arc::new(ClientPool::new(config.runtime.channels_per_address));
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let global_rate_limiter = Arc::new(
            GlobalRateLimiterManager::new(config.cluster_limit.max_network_connection_rate)
                .unwrap_or_else(|e| panic!("Failed to create GlobalRateLimiterManager: {e}")),
        );
        let server_runtime = create_runtime(
            "server-runtime",
            resolve_server_worker_threads(config.runtime.server_worker_threads),
        );
        let broker_cache = Arc::new(NodeCacheManager::new(config.clone()));
        let connection_manager = Arc::new(NetworkConnectionManager::new());
        let task_supervisor = Arc::new(TaskSupervisor::new());
        let offset_manager = Arc::new(OffsetManager::new(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            config.storage_runtime.offset_enable_cache,
        ));
        let node_call_manager = Arc::new(NodeCallManager::new(
            client_pool.clone(),
            broker_cache.clone(),
        ));

        // meta_runtime is created here so that Raft::new() tasks (spawned via
        // tokio::spawn inside openraft) land on meta_runtime, not server_runtime.
        let meta_runtime = create_runtime(
            "meta-runtime",
            resolve_meta_worker_threads(config.runtime.meta_worker_threads),
        );
        // broker_runtime is created here so tasks spawned during MQTT param
        // construction (e.g. RetainMessageManager) land on broker_runtime.
        let broker_runtime = create_runtime(
            "broker-runtime",
            resolve_broker_worker_threads(config.runtime.broker_worker_threads),
        );
        let engine_runtime = create_runtime(
            "engine-runtime",
            resolve_server_worker_threads(config.runtime.server_worker_threads),
        );

        let base = BaseComponents {
            server_runtime,
            client_pool,
            rocksdb_engine_handler,
            global_rate_limiter,
            broker_cache,
            connection_manager,
            task_supervisor,
            offset_manager,
            node_call_manager,
        };
        (base, meta_runtime, broker_runtime, engine_runtime)
    }

    /// Build storage engine params, message storage driver, delay managers,
    /// and meta (Raft) service params. All blocking init runs on `meta_runtime`.
    fn init_storage(
        config: &BrokerConfig,
        base: &BaseComponents,
        meta_runtime: &Runtime,
    ) -> (
        StorageEngineParams,
        Arc<StorageDriverManager>,
        Arc<DelayTaskManager>,
        MetaServiceServerParams,
    ) {
        let engine_params = engine::build_storage_engine_params(
            base.client_pool.clone(),
            base.rocksdb_engine_handler.clone(),
            base.broker_cache.clone(),
            base.connection_manager.clone(),
            base.offset_manager.clone(),
            base.global_rate_limiter.clone(),
            base.task_supervisor.clone(),
        );

        let storage_driver_manager = {
            let om = base.offset_manager.clone();
            let seh = engine_params.storage_engine_handler.clone();
            meta_runtime.block_on(async move {
                match StorageDriverManager::new(om, seh).await {
                    Ok(s) => Arc::new(s),
                    Err(e) => {
                        error!("Failed to build message storage driver: {}", e);
                        std::process::exit(1);
                    }
                }
            })
        };

        let delay_task_manager = Arc::new(DelayTaskManager::new(
            base.client_pool.clone(),
            storage_driver_manager.clone(),
            config.delay_task.delay_task_queue_num as u32,
            config.delay_task.delay_task_handler_concurrency,
        ));

        let meta_params = meta_runtime.block_on(meta::build_meta_service(
            base.rocksdb_engine_handler.clone(),
            delay_task_manager.clone(),
            base.node_call_manager.clone(),
            base.broker_cache.clone(),
            base.task_supervisor.clone(),
        ));

        (
            engine_params,
            storage_driver_manager,
            delay_task_manager,
            meta_params,
        )
    }

    /// Build per-protocol params and the shared request channel used by all
    /// broker protocol acceptors.
    fn init_protocol_params(
        config: &BrokerConfig,
        base: &BaseComponents,
        storage_driver_manager: &Arc<StorageDriverManager>,
        broker_runtime: &Runtime,
    ) -> (
        MqttBrokerServerParams,
        KafkaBrokerServerParams,
        AmqpBrokerServerParams,
        NatsBrokerServerParams,
        Arc<RequestChannel>,
    ) {
        // Dummy stop sender used during construction; protocols receive the real
        // app-level stop signal via start_xxx_broker() calls in start().
        let (main_stop_send, _) = broadcast::channel(2);

        // All broker protocols share one RequestChannel; the unified handler pool
        // is started once in start() via start_broker_handler_pool().
        let shared_request_channel =
            Arc::new(RequestChannel::new(config.broker_network.queue_size));

        let security_manager = Arc::new(SecurityManager::new());

        let mqtt_params = mqtt::build_mqtt_params(
            mqtt::MqttBuildParams {
                client_pool: base.client_pool.clone(),
                broker_cache: base.broker_cache.clone(),
                rocksdb_engine_handler: base.rocksdb_engine_handler.clone(),
                connection_manager: base.connection_manager.clone(),
                storage_driver_manager: storage_driver_manager.clone(),
                offset_manager: base.offset_manager.clone(),
                task_supervisor: base.task_supervisor.clone(),
                global_limit_manager: base.global_rate_limiter.clone(),
                node_call: base.node_call_manager.clone(),
                stop_sx: main_stop_send.clone(),
                request_channel: shared_request_channel.clone(),
                security_manager: security_manager.clone(),
            },
            broker_runtime,
        );

        let kafka_params = kafka::build_kafka_params(kafka::KafkaBuildParams {
            connection_manager: base.connection_manager.clone(),
            client_pool: base.client_pool.clone(),
            broker_cache: base.broker_cache.clone(),
            global_limit_manager: base.global_rate_limiter.clone(),
            task_supervisor: base.task_supervisor.clone(),
            stop_sx: main_stop_send.clone(),
            shared_request_channel: shared_request_channel.clone(),
            storage_driver_manager: storage_driver_manager.clone(),
        });
        let amqp_params = amqp::build_amqp_params(amqp::AmqpBuildParams {
            connection_manager: base.connection_manager.clone(),
            client_pool: base.client_pool.clone(),
            broker_cache: base.broker_cache.clone(),
            global_limit_manager: base.global_rate_limiter.clone(),
            task_supervisor: base.task_supervisor.clone(),
            stop_sx: main_stop_send.clone(),
            shared_request_channel: shared_request_channel.clone(),
            storage_driver_manager: storage_driver_manager.clone(),
        });
        let nats_params = nats::build_nats_params(nats::NatsBuildParams {
            connection_manager: base.connection_manager.clone(),
            client_pool: base.client_pool.clone(),
            broker_cache: base.broker_cache.clone(),
            global_limit_manager: base.global_rate_limiter.clone(),
            task_supervisor: base.task_supervisor.clone(),
            stop_sx: main_stop_send,
            shared_request_channel: shared_request_channel.clone(),
            storage_driver_manager: storage_driver_manager.clone(),
            security_manager,
        });

        (
            mqtt_params,
            kafka_params,
            amqp_params,
            nats_params,
            shared_request_channel,
        )
    }

    pub fn start(&self) {
        let config = broker_config();
        let monitor_interval_ms = config.prometheus.monitor_interval_ms;

        // Phase 1: Network-facing servers
        self.start_grpc_server();
        self.start_admin_server();
        self.start_pprof_server();
        self.start_prometheus_server();

        if !wait_for_grpc_ready(self.config.grpc_port) {
            std::process::exit(1);
        }

        // Phase 2: Meta (Raft) service
        let meta_stop_send = self.start_meta_service();
        self.server_runtime.block_on(async {
            check_meta_service_status(self.client_pool.clone()).await;
        });

        // Phase 3: Load Cache
        self.start_load_cache();

        // Phase 4: NodeCallManager
        let (app_stop, _) = broadcast::channel::<bool>(2);
        let raw_app_stop = app_stop.clone();
        self.server_runtime.block_on(async {
            self.start_node_call_manager(raw_app_stop.clone());
            self.wait_for_node_call_manager_ready().await;
        });

        // Phase 5: Register node
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

        // Phase 6: Engine service
        let engine_stop_send = self.start_engine_service();

        // Phase 7: Shared handler pool — must start before protocol acceptors begin
        // pushing packets into the channel.
        let mqtt_result = self
            .server_runtime
            .block_on(async { self.start_mqtt_broker(app_stop.clone()).await });
        let mqtt_stop_send = mqtt_result.as_ref().map(|(sx, _)| sx.clone());
        let mqtt_cmd = mqtt_result.map(|(_, cmd)| cmd);
        let (kafka_cmd, amqp_cmd, nats_cmd) = if is_broker_node(&self.config.roles) {
            (
                Some(kafka_broker::handler::command::create_command_with_storage(
                    self.kafka_params.storage_driver_manager.clone(),
                )),
                Some(amqp_broker::handler::command::create_command_with_state(
                    self.connection_manager.clone(),
                    self.amqp_params.storage_driver_manager.clone(),
                )),
                Some(nats_broker::handler::command::create_command(
                    self.connection_manager.clone(),
                    self.nats_params.cache_manager.clone(),
                    self.nats_params.storage_driver_manager.clone(),
                    self.nats_params.client_pool.clone(),
                )),
            )
        } else {
            (None, None, None)
        };
        let commands = CommandRegistry {
            mqtt: mqtt_cmd,
            kafka: kafka_cmd,
            amqp: amqp_cmd,
            nats: nats_cmd,
            storage_engine: None,
        };
        self.server_runtime.block_on(async {
            self.start_broker_handler_pool(commands, app_stop.clone());
        });

        // Phase 8: Broker protocol acceptors
        self.start_kafka_broker(app_stop.clone());
        self.start_amqp_broker(app_stop.clone());
        self.start_nats_broker(app_stop.clone());

        // Phase 9: Background services
        self.server_runtime.block_on(async {
            self.start_background_services(app_stop.clone(), monitor_interval_ms)
                .await;
        });

        self.awaiting_stop(meta_stop_send, mqtt_stop_send, engine_stop_send);
    }
}
