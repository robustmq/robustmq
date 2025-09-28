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

use crate::grpc::start_grpc_server;
use admin_server::{
    server::AdminServer,
    state::{HttpState, MQTTContext},
};
use broker_core::{
    cache::BrokerCacheManager,
    heartbeat::{check_meta_service_status, register_node, report_heartbeat},
    rocksdb::{column_family_list, storage_data_fold, RocksDBEngine},
};
use common_base::runtime::create_runtime;
use common_config::{broker::broker_config, config::BrokerConfig};
use common_metrics::core::server::register_prometheus_export;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use journal_server::{
    core::cache::CacheManager as JournalCacheManager, segment::manager::SegmentFileManager,
    server::connection_manager::ConnectionManager as JournalConnectionManager, JournalServer,
    JournalServerParams,
};
use meta_service::{
    controller::{
        journal::call_node::JournalInnerCallManager, mqtt::call_broker::MQTTInnerCallManager,
    },
    core::cache::CacheManager as PlacementCacheManager,
    raft::{
        raft_node::create_raft_node,
        route::{apply::StorageDriver, DataRoute},
        type_config::TypeConfig,
    },
    MetaServiceServer, MetaServiceServerParams,
};
use mqtt_broker::{
    bridge::manager::ConnectorManager,
    broker::{MqttBrokerServer, MqttBrokerServerParams},
    common::metrics_cache::MetricsCacheManager,
    handler::cache::MQTTCacheManager as MqttCacheManager,
    security::AuthDriver,
    storage::message::build_message_storage_driver,
    subscribe::manager::SubscribeManager,
};
use network_server::common::connection_manager::ConnectionManager as MqttConnectionManager;
use openraft::Raft;
use pprof_monitor::pprof_monitor::start_pprof_monitor;
use rate_limit::RateLimiterManager;
use schema_register::schema::SchemaRegisterManager;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};
use tokio::{runtime::Runtime, signal, sync::broadcast};
use tracing::{error, info};

mod cluster_service;
pub mod common;
mod grpc;

pub struct BrokerServer {
    main_runtime: Runtime,
    place_params: MetaServiceServerParams,
    mqtt_params: MqttBrokerServerParams,
    journal_params: JournalServerParams,
    client_pool: Arc<ClientPool>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rate_limiter_manager: Arc<RateLimiterManager>,
    broker_cache: Arc<BrokerCacheManager>,
    config: BrokerConfig,
}

impl Default for BrokerServer {
    fn default() -> Self {
        Self::new()
    }
}

impl BrokerServer {
    pub fn new() -> Self {
        let config = broker_config();
        let client_pool = Arc::new(ClientPool::new(100));
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let rate_limiter_manager = Arc::new(RateLimiterManager::new());
        let main_runtime = create_runtime("init_runtime", config.runtime.runtime_worker_threads);
        let broker_cache = Arc::new(BrokerCacheManager::new(config.cluster_name.clone()));
        let place_params = main_runtime.block_on(async {
            BrokerServer::build_meta_service(
                client_pool.clone(),
                rocksdb_engine_handler.clone(),
                broker_cache.clone(),
            )
            .await
        });
        let mqtt_params = BrokerServer::build_mqtt_server(
            client_pool.clone(),
            broker_cache.clone(),
            rocksdb_engine_handler.clone(),
        );
        let journal_params = BrokerServer::build_journal_server(client_pool.clone());

        BrokerServer {
            broker_cache,
            main_runtime,
            place_params,
            journal_params,
            config: config.clone(),
            mqtt_params,
            client_pool,
            rocksdb_engine_handler,
            rate_limiter_manager,
        }
    }
    pub fn start(&self) {
        // start grpc server
        let place_params = self.place_params.clone();
        let mqtt_params = self.mqtt_params.clone();
        let journal_params = self.journal_params.clone();
        let broker_cache = self.broker_cache.clone();
        let server_runtime =
            create_runtime("server-runtime", self.config.runtime.runtime_worker_threads);
        let grpc_port = self.config.grpc_port;

        let grpc_ready = Arc::new(AtomicBool::new(false));
        let grpc_ready_check = grpc_ready.clone();
        server_runtime.spawn(async move {
            if let Err(e) =
                start_grpc_server(place_params, mqtt_params, journal_params, grpc_port).await
            {
                panic!("{e}")
            }
        });

        // Start Admin Server
        let state = Arc::new(HttpState {
            client_pool: self.client_pool.clone(),
            connection_manager: self.mqtt_params.connection_manager.clone(),
            mqtt_context: MQTTContext {
                cache_manager: self.mqtt_params.cache_manager.clone(),
                subscribe_manager: self.mqtt_params.subscribe_manager.clone(),
                metrics_manager: self.mqtt_params.metrics_cache_manager.clone(),
                connector_manager: self.mqtt_params.connector_manager.clone(),
                schema_manager: self.mqtt_params.schema_manager.clone(),
            },
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            broker_cache: broker_cache.clone(),
            rate_limiter_manager: self.rate_limiter_manager.clone(),
        });
        server_runtime.spawn(async move {
            let admin_server = AdminServer::new();
            admin_server.start(8080, state).await;
        });

        // check grpc server ready
        self.check_grpc_server_ready(grpc_port, grpc_ready_check);

        // start pprof server
        server_runtime.spawn(async move {
            let conf = broker_config();
            if conf.p_prof.enable {
                start_pprof_monitor(conf.p_prof.port, conf.p_prof.frequency).await;
            }
        });

        // start prometheus
        let prometheus_port = self.config.prometheus.port;
        if self.config.prometheus.enable {
            server_runtime.spawn(async move {
                register_prometheus_export(prometheus_port).await;
            });
        }

        self.wait_for_grpc_ready(&grpc_ready);

        let mut place_stop_send = None;
        let mut mqtt_stop_send = None;
        let mut journal_stop_send = None;

        let config = broker_config();
        // start meta service
        let (stop_send, _) = broadcast::channel(2);
        let place_runtime =
            create_runtime("place-runtime", self.config.runtime.runtime_worker_threads);
        let place_params = self.place_params.clone();
        if config.is_start_meta() {
            place_stop_send = Some(stop_send.clone());
            place_runtime.spawn(async move {
                let mut pc = MetaServiceServer::new(place_params, stop_send.clone());
                pc.start().await;
            });

            // check placement ready
            self.main_runtime.block_on(async {
                check_meta_service_status(self.client_pool.clone()).await;
            });
        }

        // start journal server
        let (stop_send, _) = broadcast::channel(2);
        let journal_runtime = create_runtime(
            "journal-runtime",
            self.config.runtime.runtime_worker_threads,
        );

        if config.is_start_journal() {
            journal_stop_send = Some(stop_send.clone());
            let server = JournalServer::new(self.journal_params.clone(), stop_send.clone());
            journal_runtime.spawn(async move {
                server.start().await;
            });
            self.wait_for_journal_ready();
        }

        // start mqtt server
        let (stop_send, _) = broadcast::channel(2);
        let mqtt_runtime =
            create_runtime("mqtt-runtime", self.config.runtime.runtime_worker_threads);
        if config.is_start_broker() {
            mqtt_stop_send = Some(stop_send.clone());
            let server = MqttBrokerServer::new(self.mqtt_params.clone(), stop_send.clone());
            mqtt_runtime.spawn(async move {
                server.start().await;
            });
        }

        // register node
        let raw_stop_send = stop_send.clone();
        server_runtime.block_on(async move {
            self.register_node(raw_stop_send.clone()).await;
        });

        // awaiting stop
        self.awaiting_stop(place_stop_send, mqtt_stop_send, journal_stop_send);
    }

    async fn build_meta_service(
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        broker_cache: Arc<BrokerCacheManager>,
    ) -> MetaServiceServerParams {
        let cache_manager = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let journal_call_manager = Arc::new(JournalInnerCallManager::new(cache_manager.clone()));
        let mqtt_call_manager = Arc::new(MQTTInnerCallManager::new(
            cache_manager.clone(),
            broker_cache,
        ));

        let data_route = Arc::new(DataRoute::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
        ));
        let raf_node: Raft<TypeConfig> = create_raft_node(client_pool.clone(), data_route).await;
        let storage_driver: Arc<StorageDriver> = Arc::new(StorageDriver::new(raf_node.clone()));
        MetaServiceServerParams {
            cache_manager,
            rocksdb_engine_handler,
            client_pool,
            journal_call_manager,
            mqtt_call_manager,
            raf_node,
            storage_driver,
        }
    }

    fn build_mqtt_server(
        client_pool: Arc<ClientPool>,
        broker_cache: Arc<BrokerCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> MqttBrokerServerParams {
        let config = broker_config();
        let cache_manager = Arc::new(MqttCacheManager::new(
            client_pool.clone(),
            broker_cache.clone(),
        ));

        let storage_driver = match build_message_storage_driver() {
            Ok(storage) => storage,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let arc_storage_driver = Arc::new(storage_driver);
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let connector_manager = Arc::new(ConnectorManager::new());
        let connection_manager = Arc::new(MqttConnectionManager::new(
            config.network.lock_max_try_mut_times as i32,
            config.network.lock_try_mut_sleep_time_ms,
        ));
        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_pool.clone()));
        let delay_message_manager = Arc::new(DelayMessageManager::new(
            config.cluster_name.clone(),
            1,
            arc_storage_driver.clone(),
        ));
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let schema_manager = Arc::new(SchemaRegisterManager::new());

        MqttBrokerServerParams {
            cache_manager,
            client_pool,
            message_storage_adapter: arc_storage_driver,
            subscribe_manager,
            connection_manager,
            connector_manager,
            auth_driver,
            delay_message_manager,
            schema_manager,
            metrics_cache_manager,
            rocksdb_engine_handler,
            broker_cache,
        }
    }

    fn build_journal_server(client_pool: Arc<ClientPool>) -> JournalServerParams {
        let config = broker_config();
        let connection_manager = Arc::new(JournalConnectionManager::new());
        let cache_manager = Arc::new(JournalCacheManager::new());
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &journal_server::index::engine::storage_data_fold(&config.journal_storage.data_path),
            10000,
            column_family_list(),
        ));

        let segment_file_manager =
            Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

        JournalServerParams {
            cache_manager,
            client_pool,
            connection_manager,
            segment_file_manager,
            rocksdb_engine_handler,
        }
    }

    pub fn awaiting_stop(
        &self,
        place_stop: Option<broadcast::Sender<bool>>,
        mqtt_stop: Option<broadcast::Sender<bool>>,
        journal_stop: Option<broadcast::Sender<bool>>,
    ) {
        self.broker_cache
            .set_status(common_base::node_status::NodeStatus::Running);
        self.main_runtime.block_on(async {
            // Wait for all the request packets in the TCP Channel to be processed completely before starting to stop other processing threads.
            signal::ctrl_c().await.expect("failed to listen for event");
            info!(
                "{}",
                "When ctrl + c is received, the service starts to stop"
            );

            self.broker_cache
                .set_status(common_base::node_status::NodeStatus::Stopping);

            if let Some(sx) = mqtt_stop {
                if let Err(e) = sx.send(true) {
                    error!("mqtt stop signal, error message:{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Some(sx) = journal_stop {
                if let Err(e) = sx.send(true) {
                    error!("journal stop signal, error message{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Some(sx) = place_stop {
                if let Err(e) = sx.send(true) {
                    error!("place stop signal, error message{}", e);
                }
            }
            sleep(Duration::from_secs(3));
        });
    }

    fn check_grpc_server_ready(&self, grpc_port: u32, grpc_ready: Arc<AtomicBool>) {
        let max_retries = 30;
        let retry_interval = Duration::from_millis(100);

        std::thread::spawn(move || {
            let addr = format!("127.0.0.1:{grpc_port}");

            for attempt in 1..=max_retries {
                match std::net::TcpStream::connect(&addr) {
                    Ok(_) => {
                        info!("GRPC server is ready on port {grpc_port}");
                        grpc_ready.store(true, Ordering::Relaxed);
                        return;
                    }
                    Err(e) => {
                        if attempt % 10 == 0 {
                            info!(
                                "GRPC server not ready yet (attempt {}/{}): {}",
                                attempt, max_retries, e
                            );
                        }
                    }
                }

                std::thread::sleep(retry_interval);
            }
            error!(
                "GRPC server failed to start within {} attempts",
                max_retries
            );
            std::process::exit(1);
        });
    }

    fn wait_for_grpc_ready(&self, grpc_ready: &Arc<AtomicBool>) {
        let max_wait_time = Duration::from_secs(10);
        let check_interval = Duration::from_millis(100);
        let start_time = std::time::Instant::now();

        while !grpc_ready.load(Ordering::Relaxed) {
            if start_time.elapsed() > max_wait_time {
                error!("GRPC server failed to start within {:?}", max_wait_time);
                std::process::exit(1);
            }
            std::thread::sleep(check_interval);
        }

        info!("GRPC server startup check completed");
    }

    fn wait_for_journal_ready(&self) {
        let journal_port = self.config.journal_server.tcp_port;
        let max_wait_time = Duration::from_secs(10);
        let check_interval = Duration::from_millis(100);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < max_wait_time {
            match std::net::TcpStream::connect(format!("127.0.0.1:{journal_port}")) {
                Ok(_) => {
                    info!("Journal server startup check completed");
                    return;
                }
                Err(_) => {
                    std::thread::sleep(check_interval);
                }
            }
        }

        error!("Journal server failed to start within {:?}", max_wait_time);
        std::process::exit(1);
    }

    async fn register_node(&self, main_stop: broadcast::Sender<bool>) {
        // register node
        let client_pool = self.client_pool.clone();
        let broker_cache = self.broker_cache.clone();

        // register node
        let config = broker_config();
        match register_node(&client_pool, &broker_cache).await {
            Ok(()) => {
                // heartbeat report
                let raw_client_pool = client_pool.clone();
                tokio::spawn(async move {
                    report_heartbeat(&raw_client_pool, &broker_cache, main_stop.clone()).await;
                });

                info!("Node {} has been successfully registered", config.broker_id);
            }
            Err(e) => {
                error!("Node registration failed. Error message:{}", e);
            }
        }
    }
}
